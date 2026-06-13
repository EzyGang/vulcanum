use uuid::Uuid;

use crate::services::model_providers::renderer::{render_opencode_config, ModelSelection};
use crate::services::project_configs::model::JobConfigFields;
use crate::services::work_runs::errors::WorkRunsError;
use crate::services::work_runs::service::WorkRunsService;
use vulcanum_shared::api_types::JobResponse;

impl WorkRunsService {
    pub async fn get_job(&self, id: Uuid, worker_id: Uuid) -> Result<JobResponse, WorkRunsError> {
        let run = self.work_runs_repo.find_by_id(&self.db, id).await?;
        if run.worker_id != Some(worker_id) {
            return Err(WorkRunsError::NotOwned);
        }

        let config = self
            .project_configs_repo
            .find_by_id(&self.db, run.project_config_id)
            .await;

        let cfg = match config {
            Ok(ref c) => c.job_fields(),
            Err(_) => {
                tracing::warn!(
                    project_config_id = %run.project_config_id,
                    work_run_id = %id,
                    "project config not found for work run"
                );
                JobConfigFields::empty_for_team(run.team_id)
            }
        };

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

        let github_token = match cfg.repo_url.is_empty() {
            true => None,
            false => match self
                .github
                .generate_installation_token(cfg.team_id, &cfg.repo_url)
                .await
            {
                Ok(token) => Some(token.token),
                Err(e) => {
                    tracing::error!(
                        work_run_id = %id,
                        repo_url = %cfg.repo_url,
                        error = %e,
                        "failed to mint github installation token"
                    );
                    return Err(e.into());
                }
            },
        };

        let connected_model_providers = self
            .model_providers_repo
            .list_all(&self.db, cfg.team_id)
            .await
            .map_err(WorkRunsError::ModelProvider)?;
        let rendered = render_opencode_config(
            &connected_model_providers,
            ModelSelection {
                primary_provider_key: cfg.primary_model_provider_key.as_deref(),
                primary_model_id: cfg.primary_model_id.as_deref(),
                small_provider_key: cfg.small_model_provider_key.as_deref(),
                small_model_id: cfg.small_model_id.as_deref(),
            },
        );

        Ok(JobResponse {
            prompt_text: run.prompt_text,
            repo_url: run.repo_url,
            agents_md: run.agents_md,
            opencode_config: cfg.opencode_config,
            generated_opencode_config: rendered.opencode_config,
            model_provider_env: rendered.env,
            external_task_ref: run.external_task_ref,
            provider_instance_url,
            provider_api_key,
            external_project_id: cfg.external_project_id,
            external_workspace_id: cfg.external_workspace_id,
            max_turns: cfg.max_turns,
            github_token,
        })
    }
}
