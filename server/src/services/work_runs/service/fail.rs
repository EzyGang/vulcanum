use uuid::Uuid;

use crate::services::work_runs::errors::WorkRunsError;
use crate::services::work_runs::model::{WorkRun, WorkRunStatus};
use crate::services::work_runs::service::WorkRunsService;

impl WorkRunsService {
    pub async fn fail_run(&self, id: Uuid, team_id: Uuid) -> Result<WorkRun, WorkRunsError> {
        let run = self.work_runs_repo.find_by_id(&self.db, id).await?;
        if run.team_id != team_id {
            return Err(WorkRunsError::NotFound);
        }

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
}
