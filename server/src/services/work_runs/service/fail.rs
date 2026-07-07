use uuid::Uuid;

use crate::models::work_runs::errors::WorkRunsError;
use crate::models::work_runs::model::{WorkRun, WorkRunStatus};
use crate::services::work_runs::service::lifecycle_labels::LifecycleLabelState;
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
            self.workers_repo
                .decrement_active_jobs(&mut *tx, worker_id)
                .await?;
        }

        tx.commit().await.map_err(WorkRunsError::Database)?;

        self.clear_cancel_flag(id).await;
        self.set_lifecycle_label_for_run(&updated, LifecycleLabelState::NeedsAttention)
            .await;

        Ok(updated)
    }
}
