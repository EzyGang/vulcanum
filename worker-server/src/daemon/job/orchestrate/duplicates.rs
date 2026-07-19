use std::sync::Arc;

use tokio::sync::RwLock;
use uuid::Uuid;

use vulcanum_shared::api::error::{is_fatal_api_error, ApiError};
use vulcanum_shared::client::ApiClient;
use vulcanum_shared::state::worker::WorkerState;

use crate::daemon::auth::with_retry_on_401;
use crate::daemon::job::execution::submit::resubmit_stored_result;
use crate::state::journal::{Journal, JournalEntry};

pub(super) async fn reconcile_terminal_duplicate(
    client: &Arc<ApiClient>,
    worker_state: &Arc<RwLock<WorkerState>>,
    journal: &Arc<Journal>,
    job_id: Uuid,
    entry: &JournalEntry,
) -> Result<(), String> {
    tracing::warn!(
        work_run_id = %job_id,
        local_status = ?entry.status,
        workdir = %entry.workdir,
        "duplicate dispatch matches terminal local journal state, resubmitting stored result"
    );

    match with_retry_on_401(client, worker_state, |token| {
        let client = client.clone();
        async move { client.ack_job(job_id, &token).await }
    })
    .await
    {
        Ok(()) => (),
        Err(e) => {
            if is_fatal_api_error(&e) {
                return Err(format!("ack failed permanently: {e:#} — run `vulcanum worker setup --instance <instance> --code <code>` to reconnect"));
            }
            match e.downcast_ref::<ApiError>() {
                Some(api_err) if api_err.status == 404 => {
                    tracing::info!(work_run_id = %job_id, "job was deleted or cancelled, skipping duplicate reconciliation");
                    return Ok(());
                }
                Some(api_err) if api_err.status == 409 => {
                    tracing::warn!(
                        work_run_id = %job_id,
                        error = %e,
                        "duplicate reconciliation ack was rejected, attempting result resubmit anyway"
                    );
                }
                _ => {
                    tracing::warn!(work_run_id = %job_id, error = %e, "ack failed during duplicate reconciliation");
                    return Ok(());
                }
            }
        }
    }

    resubmit_stored_result(client, worker_state, journal, entry).await;
    Ok(())
}
