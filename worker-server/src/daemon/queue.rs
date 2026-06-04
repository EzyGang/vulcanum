use std::sync::Arc;

use super::job::handle_job;
use super::DaemonState;

pub(super) async fn try_drain_queue(state: &DaemonState) {
    loop {
        let semaphore = Arc::clone(&state.semaphore);
        let Ok(permit) = semaphore.try_acquire_owned() else {
            break;
        };

        let job_id = {
            let mut queue = state.pending_queue.lock().await;
            queue.pop_front()
        };

        let Some(job_id) = job_id else {
            break;
        };

        let client = Arc::clone(&state.client);
        let worker_state = Arc::clone(&state.worker_state);
        let journal = Arc::clone(&state.journal);
        let shutdown_tx = state.shutdown_tx.clone();
        let config = state.config.clone();

        tokio::spawn(async move {
            let _permit = permit;
            if let Err(msg) = handle_job(client, worker_state, journal, job_id, &config).await {
                let _ = shutdown_tx.send(Some(msg));
            }
        });
    }
}
