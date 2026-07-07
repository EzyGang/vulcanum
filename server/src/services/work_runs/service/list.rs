use crate::models::work_runs::errors::WorkRunsError;
use crate::models::work_runs::model::{WorkRunListItem, WorkRunStatus};
use crate::services::work_runs::service::WorkRunsService;

impl WorkRunsService {
    pub async fn list_all(
        &self,
        team_id: uuid::Uuid,
        status: Option<WorkRunStatus>,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<WorkRunListItem>, WorkRunsError> {
        let limit = normalize_limit(limit)?;
        let offset = normalize_offset(offset)?;
        self.work_runs_repo
            .list_all(&self.db, team_id, status, limit, offset)
            .await
    }
}

fn normalize_limit(limit: Option<i64>) -> Result<i64, WorkRunsError> {
    let limit = limit.unwrap_or(50);
    if limit < 1 {
        return Err(WorkRunsError::InvalidPagination(
            "limit must be greater than zero".to_owned(),
        ));
    }

    Ok(limit.min(100))
}

fn normalize_offset(offset: Option<i64>) -> Result<i64, WorkRunsError> {
    let offset = offset.unwrap_or(0);
    if offset < 0 {
        return Err(WorkRunsError::InvalidPagination(
            "offset must be greater than or equal to zero".to_owned(),
        ));
    }

    Ok(offset)
}
