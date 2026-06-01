use uuid::Uuid;

use crate::services::work_run_events::errors::WorkRunEventsError;
use crate::services::work_run_events::repository::work_run_events::InsertEventParams;
use crate::services::work_run_events::service::WorkRunEventsService;
use crate::services::work_runs::errors::WorkRunsError;
use crate::services::work_runs::model::WorkRunStatus;

#[derive(Debug)]
pub struct AppendResult {
    pub accepted: u64,
    pub next_expected_sequence: i64,
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

        if !matches!(
            run.status,
            WorkRunStatus::Running | WorkRunStatus::Dispatched
        ) {
            return Err(WorkRunEventsError::NotFound);
        }

        let params: Vec<InsertEventParams> = events
            .into_iter()
            .map(|e| InsertEventParams {
                sequence: e.sequence as i64,
                event_type: e.event_type,
                payload: e.payload,
            })
            .collect();

        let result = self
            .repo
            .insert_batch(&self.db, work_run_id, &params)
            .await?;

        let should_cancel = self
            .cancel_store
            .is_cancel_requested(work_run_id)
            .await
            .unwrap_or(false);

        Ok(AppendResult {
            accepted: result.accepted,
            next_expected_sequence: result.next_expected_sequence,
            should_cancel,
        })
    }
}

fn map_work_runs_error(e: WorkRunsError) -> WorkRunEventsError {
    match e {
        WorkRunsError::NotFound => WorkRunEventsError::NotFound,
        WorkRunsError::Database(e) => WorkRunEventsError::Database(e),
        WorkRunsError::AlreadyClaimed
        | WorkRunsError::InvalidStatusTransition
        | WorkRunsError::NotOwned
        | WorkRunsError::DeleteRunning
        | WorkRunsError::Dispatch(_) => {
            WorkRunEventsError::Database(sqlx::Error::Protocol(e.to_string()))
        }
    }
}
