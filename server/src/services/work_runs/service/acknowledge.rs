use uuid::Uuid;

use crate::models::work_runs::errors::WorkRunsError;
use crate::models::work_runs::model::{WorkRun, WorkRunType};
use crate::services::work_runs::service::lifecycle_labels::LifecycleLabelState;
use crate::services::work_runs::service::WorkRunsService;

impl WorkRunsService {
    pub async fn ack_job(&self, id: Uuid, worker_id: Uuid) -> Result<WorkRun, WorkRunsError> {
        let run = self
            .work_runs_repo
            .acknowledge(&self.db, id, worker_id)
            .await?;
        let state = match run.work_type {
            WorkRunType::Implementation => LifecycleLabelState::ImplementationRunning,
            WorkRunType::PullRequestReview => LifecycleLabelState::ReviewRunning,
        };
        self.set_lifecycle_label_for_run(&run, state).await;

        Ok(run)
    }
}
