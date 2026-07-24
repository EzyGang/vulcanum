use std::collections::HashSet;
use std::sync::Arc;

use tokio::sync::Mutex;
use uuid::Uuid;

use super::job::handle_job;
use super::DaemonState;

#[derive(Default)]
pub(super) struct JobTracker {
    active: Mutex<HashSet<Uuid>>,
}

impl JobTracker {
    pub(super) async fn reserve(&self, job_id: Uuid) -> bool {
        self.active.lock().await.insert(job_id)
    }

    pub(super) async fn release(&self, job_id: Uuid) {
        self.active.lock().await.remove(&job_id);
    }
}

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
        let job_tracker = Arc::clone(&state.job_tracker);

        tokio::spawn(async move {
            let _permit = permit;
            let result = handle_job(client, worker_state, journal, job_id, &config).await;
            job_tracker.release(job_id).await;
            if let Err(msg) = result {
                let _ = shutdown_tx.send(Some(msg));
            }
        });
    }
}
