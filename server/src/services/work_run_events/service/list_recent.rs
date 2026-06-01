use uuid::Uuid;

use crate::services::work_run_events::errors::WorkRunEventsError;
use crate::services::work_run_events::model::WorkRunEvent;
use crate::services::work_run_events::service::WorkRunEventsService;

const RECENT_LIMIT: i64 = 20;

impl WorkRunEventsService {
    /// Returns the most recent N events in ascending order.
    /// Used by the frontend to render the last few events when expanding a run row.
    pub async fn list_recent(
        &self,
        work_run_id: Uuid,
    ) -> Result<Vec<WorkRunEvent>, WorkRunEventsError> {
        self.repo
            .find_last_n(&self.db, work_run_id, RECENT_LIMIT)
            .await
    }
}
