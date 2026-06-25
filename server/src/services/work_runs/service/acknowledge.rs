use uuid::Uuid;

use crate::models::work_runs::errors::WorkRunsError;
use crate::models::work_runs::model::WorkRun;
use crate::services::work_runs::service::WorkRunsService;

impl WorkRunsService {
    pub async fn ack_job(&self, id: Uuid, worker_id: Uuid) -> Result<WorkRun, WorkRunsError> {
        self.work_runs_repo
            .acknowledge(&self.db, id, worker_id)
            .await
    }
}
