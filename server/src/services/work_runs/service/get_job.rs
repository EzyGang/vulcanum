use std::time::Instant;

use uuid::Uuid;
use vulcanum_shared::api_types::JobResponse;

use super::TaskCache;
use crate::models::project_configs::model::EffectiveProjectSettings;
use crate::models::provider_configs::model::IntegrationProvider;
use crate::models::work_runs::errors::WorkRunsError;
use crate::models::work_runs::model::WorkRunType;
use crate::services::model_providers::renderer::ModelSelection;
use crate::services::poller::prompts::{ENVIRONMENT_INSTRUCTION, GITHUB_INSTRUCTION};
use crate::services::poller::service::repo_layout;
use crate::services::poller::template::{self, TemplateVars};
use crate::services::providers::client::IntegrationClient;
use crate::services::work_runs::service::WorkRunsService;

impl WorkRunsService {
    pub async fn get_job(&self, id: Uuid, worker_id: Uuid) -> Result<JobResponse, WorkRunsError> {
        let run = self.work_runs_repo.find_by_id(&self.db, id).await?;
        if run.worker_id != Some(worker_id) {
            return Err(WorkRunsError::NotOwned);
        }

        let config = self.project_configs.find_by_id(run.project_config_id).await;

        let settings = match &config {
            Ok(c) => {
                let s = self.project_configs.effective_settings(c).await?;
                s
            }
            Err(_) => {
                tracing::warn!(
                    project_config_id = %run.project_config_id,
                    work_run_id = %id,
                    "project config not found for work run"
                );
                EffectiveProjectSettings::empty_for_team(run.team_id)
            }
        };

        let cfg = match &config {
            Ok(c) => c.job_fields(settings.clone()),
            Err(_) => {
                tracing::warn!(
                    project_config_id = %run.project_config_id,
                    work_run_id = %id,
                    "project config not found for work run (job fields)"
                );
                crate::models::project_configs::model::JobConfigFields::empty_for_team(run.team_id)
            }
        };

        let repos = self.github_repos_for_work_run(&run).await?;
        let pr_urls = self.work_runs_repo.list_pr_urls(&self.db, id).await?;

        let repo_urls_str = repos
            .iter()
            .map(|r| r.url.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        let repo_names_str = repos
            .iter()
            .map(|r| r.full_name.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        let repo_layout_str = repo_layout(
            &repos
                .iter()
                .map(|r| r.full_name.clone())
                .collect::<Vec<_>>(),
        );

        // Fetch the integration provider once — used for task reconstruction and API credentials
        let provider = match cfg.provider_id {
            Some(pid) => self
                .providers_repo
                .find_by_id(&self.db, pid, cfg.team_id)
                .await
                .ok(),
            None => None,
        };

        // Reconstruct prompt_text from template + task data.
        // Check a short-lived in-memory cache before hitting the provider API.
        let cache_key = (
            cfg.provider_id.unwrap_or_default(),
            run.external_task_ref.clone(),
        );
        let (task_title, task_body) = {
            let mut cache = self.task_cache.lock().await;
            if let Some((title, body, ts)) = cache.get(&cache_key) {
                if ts.elapsed() < super::TASK_CACHE_TTL {
                    (title.clone(), body.clone())
                } else {
                    cache.remove(&cache_key);
                    drop(cache);
                    fetch_and_cache_task(
                        &self.task_cache,
                        cache_key,
                        &provider,
                        &run.external_task_ref,
                        id,
                    )
                    .await
                }
            } else {
                drop(cache);
                fetch_and_cache_task(
                    &self.task_cache,
                    cache_key,
                    &provider,
                    &run.external_task_ref,
                    id,
                )
                .await
            }
        };

        let review_target_pr_url = run.review_target_pr_url.as_deref().unwrap_or("");
        let is_review = matches!(run.work_type, WorkRunType::PullRequestReview);

        let prompt_template = if is_review {
            &settings.review_prompt_template
        } else {
            &settings.prompt_template
        };

        let mut prompt_text = template::render_template(
            prompt_template,
            &TemplateVars {
                task_title: &task_title,
                task_body: &task_body,
                repo_url: repos.first().map(|r| r.url.as_str()).unwrap_or(""),
                repo_urls: &repo_urls_str,
                repo_names: &repo_names_str,
                repo_layout: &repo_layout_str,
                review_target_pr_url,
            },
        );

        if !is_review {
            prompt_text.push_str(ENVIRONMENT_INSTRUCTION);
            if !repos.is_empty() {
                prompt_text.push_str(GITHUB_INSTRUCTION);
            }
        }

        let (provider_instance_url, provider_api_key) = match &provider {
            Some(p) => (p.instance_url.clone(), p.api_key.clone()),
            None => (String::new(), String::new()),
        };

        let github_token = self
            .mint_github_token_for_repos(id, cfg.team_id, &repos)
            .await?;

        let rendered = self
            .model_providers
            .render_agent_config_for_team(
                cfg.team_id,
                cfg.agent_backend,
                ModelSelection {
                    primary_provider_key: cfg.primary_model_provider_key.as_deref(),
                    primary_model_id: cfg.primary_model_id.as_deref(),
                    small_provider_key: cfg.small_model_provider_key.as_deref(),
                    small_model_id: cfg.small_model_id.as_deref(),
                },
            )
            .await?;

        Ok(JobResponse {
            work_type: shared_work_type(run.work_type),
            prompt_text,
            repos,
            agents_md: settings.agents_md,
            agent_backend: cfg.agent_backend,
            agent_config: rendered.agent_config,
            model_provider_env: rendered.env,
            external_task_ref: run.external_task_ref,
            provider_instance_url,
            provider_api_key,
            external_project_id: cfg.external_project_id,
            external_workspace_id: cfg.external_workspace_id,
            max_turns: match run.work_type {
                WorkRunType::Implementation => cfg.max_turns,
                WorkRunType::PullRequestReview => cfg.review_max_turns,
            },
            github_token: github_token.github_token,
            github_token_expires_at: github_token.github_token_expires_at,
            pr_urls,
            review_target_pr_url: run.review_target_pr_url,
            review_target_repo_full_name: run.review_target_repo_full_name,
        })
    }
}

/// Fetches task data from the provider API and caches the result.
async fn fetch_and_cache_task(
    cache: &TaskCache,
    cache_key: (Uuid, String),
    provider: &Option<IntegrationProvider>,
    task_ref: &str,
    work_run_id: Uuid,
) -> (String, String) {
    let result = match provider {
        Some(p) => {
            let client = IntegrationClient::from_provider(p);
            match client.fetch_task_by_id(task_ref).await {
                Ok(Some(task)) => (task.title, task.description.unwrap_or_default()),
                Ok(None) | Err(_) => {
                    tracing::warn!(
                        work_run_id = %work_run_id,
                        task_ref = %task_ref,
                        "failed to fetch task data from provider for prompt reconstruction"
                    );
                    (String::new(), String::new())
                }
            }
        }
        None => (String::new(), String::new()),
    };
    if !result.0.is_empty() || !result.1.is_empty() {
        cache.lock().await.insert(
            cache_key,
            (result.0.clone(), result.1.clone(), Instant::now()),
        );
    }
    result
}

#[must_use]
fn shared_work_type(work_type: WorkRunType) -> vulcanum_shared::api_types::WorkRunType {
    match work_type {
        WorkRunType::Implementation => vulcanum_shared::api_types::WorkRunType::Implementation,
        WorkRunType::PullRequestReview => {
            vulcanum_shared::api_types::WorkRunType::PullRequestReview
        }
    }
}
