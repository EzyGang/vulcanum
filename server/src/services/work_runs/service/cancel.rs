use uuid::Uuid;

use crate::models::work_runs::errors::WorkRunsError;
use crate::models::work_runs::model::WorkRunStatus;
use crate::services::work_runs::service::WorkRunsService;

impl WorkRunsService {
    pub async fn cancel_run(&self, id: Uuid, team_id: Uuid) -> Result<(), WorkRunsError> {
        let run = self.work_runs_repo.find_by_id(&self.db, id).await?;
        if run.team_id != team_id {
            return Err(WorkRunsError::NotFound);
        }

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
}
