use uuid::Uuid;
use vulcanum_shared::api_types::{AgentBackend, JobResponse, WorkerCapabilities};

use crate::models::project_configs::model::JobConfigFields;
use crate::models::work_runs::errors::WorkRunsError;
use crate::models::work_runs::model::WorkRunType;
use crate::services::model_providers::renderer::ModelSelection;
use crate::services::work_runs::service::WorkRunsService;

impl WorkRunsService {
    pub async fn get_job(&self, id: Uuid, worker_id: Uuid) -> Result<JobResponse, WorkRunsError> {
        let run = self.work_runs_repo.find_by_id(&self.db, id).await?;
        if run.worker_id != Some(worker_id) {
            return Err(WorkRunsError::NotOwned);
        }

        let worker = self.workers_repo.find_by_id(&self.db, worker_id).await?;
        let agent_backend = worker_agent_backend(&worker.capabilities);
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

        let github_token = self
            .mint_github_token_for_repos(id, cfg.team_id, &repos)
            .await?;

        let rendered = self
            .model_providers
            .render_agent_config_for_team(
                cfg.team_id,
                agent_backend,
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
            prompt_text: run.prompt_text,
            repos,
            agents_md: run.agents_md,
            agent_backend,
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
fn worker_agent_backend(capabilities: &serde_json::Value) -> AgentBackend {
    serde_json::from_value::<WorkerCapabilities>(capabilities.clone())
        .ok()
        .and_then(|capabilities| capabilities.agent_backends.first().copied())
        .unwrap_or_default()
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
