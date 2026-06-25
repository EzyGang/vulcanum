use crate::models::work_runs::errors::WorkRunsError;
use crate::models::work_runs::model::{WorkRunListItem, WorkRunStatus};
use crate::services::work_runs::service::WorkRunsService;

impl WorkRunsService {
    pub async fn list_all(
        &self,
        team_id: uuid::Uuid,
        status: Option<WorkRunStatus>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<WorkRunListItem>, WorkRunsError> {
        self.work_runs_repo
            .list_all(&self.db, team_id, status, limit, offset)
            .await
    }
}
