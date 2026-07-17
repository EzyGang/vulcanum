use tokio::time::sleep;

use vulcanum_shared::api::error::is_fatal_api_error;

use super::auth::{ensure_token_valid, with_retry_on_401};
use super::queue::try_drain_queue;
use super::{DaemonState, TickOutcome};

pub(super) async fn tick(state: &DaemonState, refresh_buffer_secs: i64) -> TickOutcome {
    if let Err(e) =
        ensure_token_valid(&state.client, &state.worker_state, refresh_buffer_secs).await
    {
        if is_fatal_api_error(&e) {
            return TickOutcome::Fatal(format!(
                "token refresh failed permanently: {e:#} — run `vulcanum worker setup --instance <instance> --code <code>` to reconnect"
            ));
        }
        tracing::warn!("token refresh failed: {e:#} — if this persists, try `vulcanum worker setup --instance <instance> --code <code>`");
        return TickOutcome::Transient(e.to_string());
    }

    try_drain_queue(state).await;

    tracing::info!("polling server for jobs");

    match with_retry_on_401(&state.client, &state.worker_state, |token| {
        let client = state.client.clone();
        async move { client.poll(&token).await }
    })
    .await
    {
        Ok(Some(job_id)) => {
            {
                let mut queue = state.pending_queue.lock().await;
                queue.push_back(job_id);
            }
            try_drain_queue(state).await;
            TickOutcome::Success
        }
        Ok(None) => {
            let interval = state.config.poll_interval_secs;
            tracing::info!("no jobs available, sleeping {interval}s");
            sleep(std::time::Duration::from_secs(interval)).await;
            TickOutcome::Success
        }
        Err(e) => {
            if is_fatal_api_error(&e) {
                return TickOutcome::Fatal(format!(
                    "poll failed permanently: {e:#} — run `vulcanum worker setup --instance <instance> --code <code>` to reconnect"
                ));
            }
            TickOutcome::Transient(e.to_string())
        }
    }
}
