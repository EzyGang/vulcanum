use uuid::Uuid;

use crate::services::work_run_events::errors::WorkRunEventsError;
use crate::services::work_run_events::repository::queries::InsertEventParams;
use crate::services::work_run_events::service::{map_work_runs_error, WorkRunEventsService};

#[derive(Debug)]
pub struct AppendResult {
    pub accepted: u64,
    pub should_cancel: bool,
}

impl WorkRunEventsService {
    pub async fn append_events(
        &self,
        work_run_id: Uuid,
        worker_id: Uuid,
        events: Vec<vulcanum_shared::api_types::WireEvent>,
    ) -> Result<AppendResult, WorkRunEventsError> {
        let run = self
            .work_runs_repo
            .find_by_id(&self.db, work_run_id)
            .await
            .map_err(map_work_runs_error)?;

        if run.worker_id != Some(worker_id) {
            return Err(WorkRunEventsError::NotFound);
        }

        let params: Vec<InsertEventParams> = events
            .into_iter()
            .map(|e| InsertEventParams {
                sequence: e.sequence as i64,
                event_type: e.event_type,
                payload: e.payload,
                occurred_at: e.occurred_at,
            })
            .collect();

        let result = self
            .repo
            .insert_batch(&self.db, work_run_id, &params)
            .await?;

        self.work_runs_repo
            .touch_active_run(&self.db, work_run_id, worker_id)
            .await
            .map_err(map_work_runs_error)?;

        let should_cancel = self
            .cancel_store
            .is_cancel_requested(work_run_id)
            .await
            .unwrap_or(false);

        Ok(AppendResult {
            accepted: result.accepted,
            should_cancel,
        })
    }
}
