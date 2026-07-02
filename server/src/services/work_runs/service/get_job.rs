use uuid::Uuid;
use vulcanum_shared::api_types::JobResponse;

use crate::models::project_configs::model::JobConfigFields;
use crate::models::work_runs::errors::WorkRunsError;
use crate::models::work_runs::model::WorkRunType;
use crate::services::model_providers::renderer::ModelSelection;
use crate::services::poller::prompts::{ENVIRONMENT_INSTRUCTION, GITHUB_INSTRUCTION};
use crate::services::poller::service::repo_layout;
use crate::services::poller::template::{render_template, TemplateVars};
use crate::services::providers::client::IntegrationClient;
use crate::services::work_runs::service::WorkRunsService;
use crate::util::github::github_repo_full_name_from_url;

impl WorkRunsService {
    pub async fn get_job(&self, id: Uuid, worker_id: Uuid) -> Result<JobResponse, WorkRunsError> {
        let run = self.work_runs_repo.find_by_id(&self.db, id).await?;
        if run.worker_id != Some(worker_id) {
            return Err(WorkRunsError::NotOwned);
        }

        let config = self.project_configs.find_by_id(run.project_config_id).await;

        let cfg = match &config {
            Ok(c) => {
                let settings = self.project_configs.effective_settings(c).await?;
                c.job_fields(settings)
            }
            Err(_) => {
                tracing::warn!(
                    project_config_id = %run.project_config_id,
                    work_run_id = %id,
                    "project config not found for work run"
                );
                JobConfigFields::empty_for_team(run.team_id)
            }
        };
        let repos = self.github_repos_for_work_run(&run).await?;
        let pr_urls = self.work_runs_repo.list_pr_urls(&self.db, id).await?;

        let (provider_instance_url, provider_api_key) = match cfg.provider_id {
            Some(pid) => match self
                .providers_repo
                .find_by_id(&self.db, pid, cfg.team_id)
                .await
            {
                Ok(provider) => (provider.instance_url, provider.api_key),
                Err(_) => (String::new(), String::new()),
            },
            None => (String::new(), String::new()),
        };

        let prompt_text = self
            .reconstruct_prompt_text(&run, &cfg)
            .await
            .unwrap_or_else(|e| {
                tracing::warn!(work_run_id = %id, error = %e, "failed to reconstruct prompt_text, using empty");
                String::new()
            });

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
            agents_md: cfg.agents_md,
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

    async fn reconstruct_prompt_text(
        &self,
        run: &crate::models::work_runs::model::WorkRun,
        cfg: &JobConfigFields,
    ) -> Result<String, WorkRunsError> {
        let repo_url_strs: Vec<&str> = cfg.repo_urls.iter().map(String::as_str).collect();
        let repo_urls = repo_url_strs.join("\n");
        let repo_names: Vec<String> = cfg
            .repo_urls
            .iter()
            .map(|url| github_repo_full_name_from_url(url))
            .collect();
        let repo_names_str: Vec<&str> = repo_names.iter().map(String::as_str).collect();
        let repo_names_joined = repo_names_str.join("\n");
        let repo_layout = repo_layout(&cfg.repo_urls);

        let (task_title, task_body) = self.fetch_task_data(run, cfg).await.unwrap_or_else(|e| {
            tracing::warn!(
                work_run_id = %run.id,
                external_task_ref = %run.external_task_ref,
                error = %e,
                "failed to fetch task data, using empty defaults"
            );
            (String::new(), String::new())
        });

        let repo_url = cfg.repo_urls.first().cloned().unwrap_or_default();
        let review_target_pr_url = run.review_target_pr_url.as_deref().unwrap_or("");

        let mut prompt_text = render_template(
            &cfg.prompt_template,
            &TemplateVars {
                task_title: &task_title,
                task_body: &task_body,
                repo_url: &repo_url,
                repo_urls: &repo_urls,
                repo_names: &repo_names_joined,
                repo_layout: &repo_layout,
                review_target_pr_url,
            },
        );

        if matches!(run.work_type, WorkRunType::Implementation) {
            prompt_text.push_str(ENVIRONMENT_INSTRUCTION);
            if !cfg.repo_urls.is_empty() {
                prompt_text.push_str(GITHUB_INSTRUCTION);
            }
        }

        Ok(prompt_text)
    }

    async fn fetch_task_data(
        &self,
        run: &crate::models::work_runs::model::WorkRun,
        cfg: &JobConfigFields,
    ) -> Result<(String, String), WorkRunsError> {
        let provider_id = match cfg.provider_id {
            Some(pid) => pid,
            None => return Ok((String::new(), String::new())),
        };

        let provider = self
            .providers_repo
            .find_by_id(&self.db, provider_id, cfg.team_id)
            .await
            .map_err(|e| {
                WorkRunsError::Database(sqlx::Error::Protocol(format!(
                    "failed to look up provider: {e}"
                )))
            })?;

        let client = IntegrationClient::from_provider(&provider);
        let task = client
            .fetch_task(&run.external_task_ref)
            .await
            .map_err(|e| {
                WorkRunsError::Database(sqlx::Error::Protocol(format!("failed to fetch task: {e}")))
            })?;

        Ok((task.title, task.description.unwrap_or_default()))
    }
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
