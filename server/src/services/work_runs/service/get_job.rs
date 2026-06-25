use uuid::Uuid;

use crate::models::project_configs::model::JobConfigFields;
use crate::models::work_runs::errors::WorkRunsError;
use crate::models::work_runs::model::WorkRunType;
use crate::services::model_providers::renderer::ModelSelection;
use crate::services::work_runs::service::WorkRunsService;
use crate::util::github::github_repo_full_name_from_url;
use vulcanum_shared::api_types::{JobRepo, JobResponse};

impl WorkRunsService {
    pub async fn get_job(&self, id: Uuid, worker_id: Uuid) -> Result<JobResponse, WorkRunsError> {
        let run = self.work_runs_repo.find_by_id(&self.db, id).await?;
        if run.worker_id != Some(worker_id) {
            return Err(WorkRunsError::NotOwned);
        }

        let config = self.project_configs.find_by_id(run.project_config_id).await;

        let cfg = match config {
            Ok(ref c) => {
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
        let mut repos = self.work_runs_repo.list_repos(&self.db, id).await?;
        if repos.is_empty() && !run.repo_url.is_empty() {
            repos.push(JobRepo {
                full_name: github_repo_full_name_from_url(&run.repo_url),
                url: run.repo_url.clone(),
            });
        }
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

        let repo_full_names = repos
            .iter()
            .map(|repo| repo.full_name.clone())
            .collect::<Vec<String>>();
        let github_token = match repo_full_names.is_empty() {
            true => None,
            false => match self
                .github
                .generate_installation_token_for_repos(cfg.team_id, &repo_full_names)
                .await
            {
                Ok(token) => Some(token.token),
                Err(e) => {
                    tracing::error!(
                        work_run_id = %id,
                        error = %e,
                        "failed to mint github installation token"
                    );
                    return Err(e.into());
                }
            },
        };

        let rendered = self
            .model_providers
            .render_opencode_config_for_team(
                cfg.team_id,
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
            generated_opencode_config: rendered.opencode_config,
            model_provider_env: rendered.env,
            opencode_auth_content: rendered.opencode_auth_content,
            external_task_ref: run.external_task_ref,
            provider_instance_url,
            provider_api_key,
            external_project_id: cfg.external_project_id,
            external_workspace_id: cfg.external_workspace_id,
            max_turns: match run.work_type {
                WorkRunType::Implementation => cfg.max_turns,
                WorkRunType::PullRequestReview => cfg.review_max_turns,
            },
            github_token,
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
