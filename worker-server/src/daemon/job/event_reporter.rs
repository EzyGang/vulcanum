use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use tokio::sync::RwLock;

use vulcanum_shared::api_types::WireEvent;
use vulcanum_shared::client::ApiClient;
use vulcanum_shared::worker_state::WorkerState;

use crate::daemon::auth::with_retry_on_401;

pub(crate) struct EventReporter {
    client: Arc<ApiClient>,
    worker_state: Arc<RwLock<WorkerState>>,
    job_id: uuid::Uuid,
    sequence: AtomicU64,
}

impl EventReporter {
    pub(crate) fn new(
        client: Arc<ApiClient>,
        worker_state: Arc<RwLock<WorkerState>>,
        job_id: uuid::Uuid,
    ) -> Self {
        Self {
            client,
            worker_state,
            job_id,
            sequence: AtomicU64::new(0),
        }
    }

    pub(crate) fn emit(&self, event_type: &str, payload: serde_json::Value) {
        let seq = self.sequence.fetch_add(1, Ordering::Relaxed) + 1;
        let wire = WireEvent {
            sequence: seq,
            event_type: event_type.to_owned(),
            payload,
            occurred_at: chrono::Utc::now(),
        };
        let client = Arc::clone(&self.client);
        let worker_state = Arc::clone(&self.worker_state);
        let job_id = self.job_id;
        let events = vec![wire];
        tokio::spawn(async move {
            match with_retry_on_401(&client, &worker_state, |token| {
                let client = client.clone();
                let events = events.clone();
                async move { client.append_events(job_id, &events, &token).await }
            })
            .await
            {
                Ok(resp) => {
                    if resp.should_cancel {
                        tracing::warn!(
                            work_run_id = %job_id,
                            "server requested cancel via event response"
                        );
                    }
                }
                Err(e) => {
                    tracing::warn!(
                        work_run_id = %job_id,
                        error = %e,
                        "failed to send event to server"
                    );
                }
            }
        });
    }
}
