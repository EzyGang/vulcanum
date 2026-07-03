use uuid::Uuid;
use vulcanum_shared::api_types::JobResponse;

use crate::models::project_configs::model::JobConfigFields;
use crate::models::work_runs::errors::WorkRunsError;
use crate::models::work_runs::model::WorkRunType;
use crate::services::model_providers::renderer::ModelSelection;
use crate::services::poller::template::{render_template, TemplateVars};
use crate::services::providers::client::IntegrationClient;
use crate::services::work_runs::service::WorkRunsService;

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

        let (provider_instance_url, provider_api_key, task_title, task_body) = match cfg.provider_id
        {
            Some(pid) => match self
                .providers_repo
                .find_by_id(&self.db, pid, cfg.team_id)
                .await
            {
                Ok(provider) => {
                    let instance_url = provider.instance_url.clone();
                    let api_key = provider.api_key.clone();
                    let client = IntegrationClient::from_provider(&provider);
                    let (title, body) = match client.fetch_board(&cfg.external_project_id).await {
                        Ok(board) => {
                            let task = board
                                .columns
                                .iter()
                                .flat_map(|col| col.tasks.iter())
                                .find(|t| t.id == run.external_task_ref);
                            match task {
                                Some(t) => {
                                    (t.title.clone(), t.description.clone().unwrap_or_default())
                                }
                                None => (String::new(), String::new()),
                            }
                        }
                        Err(e) => {
                            tracing::warn!(
                                error = %e,
                                external_project_id = %cfg.external_project_id,
                                "failed to fetch board for task details"
                            );
                            (String::new(), String::new())
                        }
                    };
                    (instance_url, api_key, title, body)
                }
                Err(_) => (String::new(), String::new(), String::new(), String::new()),
            },
            None => (String::new(), String::new(), String::new(), String::new()),
        };

        let repo_url = repos.first().map(|r| r.url.as_str()).unwrap_or("");
        let repo_urls = repos
            .iter()
            .map(|r| r.url.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        let repo_names = repos
            .iter()
            .map(|r| r.full_name.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        let repo_layout = crate::services::poller::service::repo_layout(
            &repos
                .iter()
                .map(|r| r.full_name.clone())
                .collect::<Vec<_>>(),
        );

        let template_vars = TemplateVars {
            task_title: &task_title,
            task_body: &task_body,
            repo_url,
            repo_urls: &repo_urls,
            repo_names: &repo_names,
            repo_layout: &repo_layout,
            review_target_pr_url: run.review_target_pr_url.as_deref().unwrap_or(""),
        };
        let prompt_text = render_template(&cfg.prompt_template, &template_vars);

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
