use uuid::Uuid;

use crate::models::work_runs::errors::WorkRunsError;
use crate::models::work_runs::model::WorkRunStatus;
use crate::services::work_runs::service::WorkRunsService;

impl WorkRunsService {
    pub async fn delete_run(&self, id: Uuid, team_id: Uuid) -> Result<(), WorkRunsError> {
        let run = self.work_runs_repo.find_by_id(&self.db, id).await?;
        if run.team_id != team_id {
            return Err(WorkRunsError::NotFound);
        }

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

    pub async fn bulk_delete_runs(
        &self,
        ids: &[Uuid],
        team_id: Uuid,
    ) -> Result<u64, WorkRunsError> {
        let mut tx = self.db.begin().await.map_err(WorkRunsError::Database)?;
        let mut deleted = 0u64;

        for id in ids {
            match self.work_runs_repo.find_by_id(&mut *tx, *id).await {
                Ok(run) => {
                    if run.team_id != team_id {
                        tracing::warn!(work_run_id = %id, "skipping run outside team in bulk delete");
                        continue;
                    }

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
}
