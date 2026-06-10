use uuid::Uuid;

use crate::services::work_run_events::errors::WorkRunEventsError;
use crate::services::work_run_events::model::WorkRunEvent;
use crate::services::work_run_events::service::WorkRunEventsService;

const RECENT_LIMIT: i64 = 20;

impl WorkRunEventsService {
    /// Returns the most recent events in ascending chronological order.
    /// Used by the frontend to render the event timeline when expanding a run row.
    pub async fn list_recent(
        &self,
        work_run_id: Uuid,
        team_id: Uuid,
    ) -> Result<Vec<WorkRunEvent>, WorkRunEventsError> {
        let run = self
            .work_runs_repo
            .find_by_id(&self.db, work_run_id)
            .await
            .map_err(|_| WorkRunEventsError::NotFound)?;
        if run.team_id != team_id {
            return Err(WorkRunEventsError::NotFound);
        }

        self.repo
            .find_last_n(&self.db, work_run_id, RECENT_LIMIT)
            .await
    }
}
