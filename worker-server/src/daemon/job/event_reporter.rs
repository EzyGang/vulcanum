use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use vulcanum_shared::api_types::WireEvent;
use vulcanum_shared::client::ApiClient;

pub(crate) struct EventReporter {
    client: Arc<ApiClient>,
    access_token: String,
    job_id: uuid::Uuid,
    sequence: AtomicU64,
}

impl EventReporter {
    pub(crate) fn new(client: Arc<ApiClient>, access_token: String, job_id: uuid::Uuid) -> Self {
        Self {
            client,
            access_token,
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
        let token = self.access_token.clone();
        let job_id = self.job_id;
        let events = vec![wire];
        tokio::spawn(async move {
            match client.append_events(job_id, &events, &token).await {
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
