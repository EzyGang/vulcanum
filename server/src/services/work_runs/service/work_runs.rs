use uuid::Uuid;

use crate::services::integrations::client::IntegrationClient;
use crate::services::work_runs::errors::WorkRunsError;
use crate::services::work_runs::model::{WorkRun, WorkRunListItem, WorkRunStatus};
use crate::services::work_runs::repository::work_runs::SetResultParams;
use crate::services::work_runs::service::WorkRunsService;
use vulcanum_shared::api_types::{JobResponse, SubmitResultRequest};

impl WorkRunsService {
    pub async fn poll(&self, worker_id: Uuid) -> Result<Option<Uuid>, WorkRunsError> {
        if let Err(e) = self
            .workers_repo
            .update_last_seen(&self.db, worker_id, chrono::Utc::now())
            .await
        {
            tracing::warn!(error = %e, worker_id = %worker_id, "failed to update last_seen");
        }

        let dispatched_id = self.dispatch_store.take_dispatched(worker_id).await?;

        Ok(dispatched_id)
    }

    pub async fn get_job(&self, id: Uuid) -> Result<JobResponse, WorkRunsError> {
        let run = self.work_runs_repo.find_by_id(&self.db, id).await?;

        let config = self
            .project_configs_repo
            .find_by_id(&self.db, run.project_config_id)
            .await;

        let (kaneo_project_id, kaneo_workspace_id, provider_id) = match config {
            Ok(ref c) => (
                c.kaneo_project_id.clone(),
                c.kaneo_workspace_id.clone(),
                c.provider_id,
            ),
            Err(_) => {
                tracing::warn!(
                    project_config_id = %run.project_config_id,
                    work_run_id = %id,
                    "project config not found for work run"
                );
                (String::new(), String::new(), None)
            }
        };

        let kaneo_instance;
        let kaneo_api_key;

        match provider_id {
            Some(pid) => match self.providers_repo.find_by_id(&self.db, pid).await {
                Ok(provider) => {
                    kaneo_instance = provider.instance_url;
                    kaneo_api_key = provider.api_key;
                }
                Err(_) => {
                    kaneo_instance = String::new();
                    kaneo_api_key = String::new();
                }
            },
            None => {
                kaneo_instance = String::new();
                kaneo_api_key = String::new();
            }
        }

        Ok(JobResponse {
            prompt_text: run.prompt_text,
            repo_url: run.repo_url,
            agents_md: run.agents_md,
            external_task_ref: run.external_task_ref,
            kaneo_instance,
            kaneo_api_key,
            kaneo_project_id,
            kaneo_workspace_id,
        })
    }

    pub async fn ack_job(&self, id: Uuid, worker_id: Uuid) -> Result<WorkRun, WorkRunsError> {
        self.work_runs_repo
            .acknowledge(&self.db, id, worker_id)
            .await
    }

    pub async fn list_all(
        &self,
        status: Option<WorkRunStatus>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<WorkRunListItem>, WorkRunsError> {
        self.work_runs_repo
            .list_all(&self.db, status, limit, offset)
            .await
    }

    pub async fn delete_run(&self, id: Uuid) -> Result<(), WorkRunsError> {
        let run = self.work_runs_repo.find_by_id(&self.db, id).await?;

        if matches!(run.status, WorkRunStatus::Running) {
            return Err(WorkRunsError::DeleteRunning);
        }

        let mut tx = self.db.begin().await.map_err(WorkRunsError::Database)?;

        if let Some(worker_id) = run.worker_id {
            if matches!(run.status, WorkRunStatus::Dispatched) {
                if let Err(e) = self
                    .workers_repo
                    .decrement_active_jobs(&mut *tx, worker_id)
                    .await
                {
                    tracing::warn!(
                        error = %e,
                        worker_id = %worker_id,
                        work_run_id = %id,
                        "failed to decrement active_jobs on run deletion"
                    );
                }
            }
        }

        let delete_r = self.work_runs_repo.delete(&mut *tx, id).await;
        if let Err(e) = delete_r {
            let _ = tx.rollback().await;
            return Err(e);
        }

        tx.commit().await.map_err(WorkRunsError::Database)
    }

    pub async fn submit_result(
        &self,
        id: Uuid,
        worker_id: Uuid,
        params: SubmitResultRequest,
    ) -> Result<WorkRun, WorkRunsError> {
        let status = if params.exit_code == 0 {
            WorkRunStatus::Completed
        } else {
            WorkRunStatus::Failed
        };

        let run = self.work_runs_repo.find_by_id(&self.db, id).await?;

        if !matches!(run.status, WorkRunStatus::Running) {
            return Err(WorkRunsError::InvalidStatusTransition);
        }

        if run.worker_id != Some(worker_id) {
            return Err(WorkRunsError::NotOwned);
        }

        let mut tx = self.db.begin().await.map_err(WorkRunsError::Database)?;

        let updated = self
            .work_runs_repo
            .set_result(
                &mut *tx,
                id,
                SetResultParams {
                    pr_url: &params.pr_url,
                    exit_code: params.exit_code,
                    tokens_used: params.tokens_used,
                    duration_ms: params.duration_ms,
                    status,
                },
            )
            .await?;

        if let Err(e) = self
            .workers_repo
            .decrement_active_jobs(&mut *tx, worker_id)
            .await
        {
            tracing::warn!(
                error = %e,
                worker_id = %worker_id,
                "failed to decrement active_jobs"
            );
        }

        tx.commit().await.map_err(WorkRunsError::Database)?;

        tracing::info!(
            worker_id = %worker_id,
            work_run_id = %id,
            tokens_used = params.tokens_used,
            duration_ms = params.duration_ms,
            exit_code = params.exit_code,
            has_pr_url = !params.pr_url.is_empty(),
            "work_run completed by worker",
        );

        self.sync_kaneo_on_result(&run, &params, status).await;

        Ok(updated)
    }

    async fn sync_kaneo_on_result(
        &self,
        run: &WorkRun,
        params: &SubmitResultRequest,
        status: WorkRunStatus,
    ) {
        let project_config = match self
            .project_configs_repo
            .find_by_id(&self.db, run.project_config_id)
            .await
        {
            Ok(c) => c,
            Err(e) => {
                tracing::warn!(
                    project_config_id = %run.project_config_id,
                    work_run_id = %run.id,
                    error = %e,
                    "failed to look up project config",
                );
                return;
            }
        };

        let provider_id = match project_config.provider_id {
            Some(pid) => pid,
            None => {
                tracing::warn!(
                    project_config_id = %run.project_config_id,
                    work_run_id = %run.id,
                    "no provider configured for project config",
                );
                return;
            }
        };

        let provider = match self.providers_repo.find_by_id(&self.db, provider_id).await {
            Ok(p) => p,
            Err(e) => {
                tracing::warn!(
                    provider_id = %provider_id,
                    work_run_id = %run.id,
                    error = %e,
                    "failed to look up provider",
                );
                return;
            }
        };

        let client = IntegrationClient::new_kaneo(provider.instance_url, provider.api_key);

        let new_column = match status {
            WorkRunStatus::Completed => &project_config.target_column,
            _ => &project_config.pickup_column,
        };

        if let Err(e) = client
            .update_task_status(&run.external_task_ref, new_column)
            .await
        {
            tracing::warn!(
                task_ref = %run.external_task_ref,
                column = %new_column,
                error = %e,
                "failed to update kaneo task status",
            );
        }

        if let Err(e) = client
            .add_comment(&run.external_task_ref, &format!("PR: {}", params.pr_url))
            .await
        {
            tracing::warn!(
                task_ref = %run.external_task_ref,
                error = %e,
                "failed to add kaneo comment",
            );
        }
    }
}
