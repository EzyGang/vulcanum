use uuid::Uuid;

use crate::services::project_configs::model::JobConfigFields;
use crate::services::work_runs::errors::WorkRunsError;
use crate::services::work_runs::model::{WorkRun, WorkRunListItem, WorkRunStatus};
use crate::services::work_runs::service::WorkRunsService;
use vulcanum_shared::api_types::JobResponse;

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

        let cfg = match config {
            Ok(ref c) => c.job_fields(),
            Err(_) => {
                tracing::warn!(
                    project_config_id = %run.project_config_id,
                    work_run_id = %id,
                    "project config not found for work run"
                );
                JobConfigFields::default()
            }
        };

        let (kaneo_instance, kaneo_api_key) = match cfg.provider_id {
            Some(pid) => match self.providers_repo.find_by_id(&self.db, pid).await {
                Ok(provider) => (provider.instance_url, provider.api_key),
                Err(_) => (String::new(), String::new()),
            },
            None => (String::new(), String::new()),
        };

        Ok(JobResponse {
            prompt_text: run.prompt_text,
            repo_url: run.repo_url,
            agents_md: run.agents_md,
            opencode_config: cfg.opencode_config,
            external_task_ref: run.external_task_ref,
            kaneo_instance,
            kaneo_api_key,
            kaneo_project_id: cfg.kaneo_project_id,
            kaneo_workspace_id: cfg.kaneo_workspace_id,
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

    pub async fn fail_run(&self, id: Uuid) -> Result<WorkRun, WorkRunsError> {
        let run = self.work_runs_repo.find_by_id(&self.db, id).await?;

        match run.status {
            WorkRunStatus::Running | WorkRunStatus::Dispatched => (),
            _ => return Err(WorkRunsError::InvalidStatusTransition),
        }

        let mut tx = self.db.begin().await.map_err(WorkRunsError::Database)?;

        let updated = self
            .work_runs_repo
            .force_fail(&mut *tx, id)
            .await?
            .ok_or(WorkRunsError::NotFound)?;

        if let Some(worker_id) = updated.worker_id {
            if let Err(e) = self
                .workers_repo
                .decrement_active_jobs(&mut *tx, worker_id)
                .await
            {
                tracing::warn!(
                    error = %e,
                    worker_id = %worker_id,
                    "failed to decrement active_jobs on force fail"
                );
            }
        }

        tx.commit().await.map_err(WorkRunsError::Database)?;

        self.clear_cancel_flag(id).await;

        Ok(updated)
    }

    pub async fn cancel_run(&self, id: Uuid) -> Result<(), WorkRunsError> {
        let run = self.work_runs_repo.find_by_id(&self.db, id).await?;

        match run.status {
            WorkRunStatus::Running | WorkRunStatus::Dispatched => (),
            _ => return Err(WorkRunsError::InvalidStatusTransition),
        }

        self.cancel_store
            .request_cancel(id)
            .await
            .map_err(WorkRunsError::Dispatch)?;

        Ok(())
    }

    pub async fn bulk_delete_runs(&self, ids: &[Uuid]) -> Result<u64, WorkRunsError> {
        let mut tx = self.db.begin().await.map_err(WorkRunsError::Database)?;
        let mut deleted = 0u64;

        for id in ids {
            match self.work_runs_repo.find_by_id(&mut *tx, *id).await {
                Ok(run) => {
                    if matches!(run.status, WorkRunStatus::Running) {
                        tracing::warn!(work_run_id = %id, "skipping running run in bulk delete");
                        continue;
                    }

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
                                    "failed to decrement active_jobs on bulk delete"
                                );
                            }
                        }
                    }

                    self.work_runs_repo.delete(&mut *tx, *id).await?;

                    deleted += 1;
                }
                Err(e) => {
                    tracing::warn!(work_run_id = %id, error = %e, "skipping not found run in bulk delete");
                }
            }
        }

        tx.commit().await.map_err(WorkRunsError::Database)?;

        Ok(deleted)
    }

    pub(crate) async fn clear_cancel_flag(&self, work_run_id: Uuid) {
        if let Err(e) = self.cancel_store.take_cancel(work_run_id).await {
            tracing::warn!(
                error = %e,
                work_run_id = %work_run_id,
                "failed to clear cancel flag on terminal status"
            );
        }
    }
}
