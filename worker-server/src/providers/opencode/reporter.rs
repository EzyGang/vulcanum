use std::sync::Arc;

use vulcanum_shared::api_types::WireEvent;
use vulcanum_shared::client::ApiClient;

use super::runner::OpenCodeRunningSession;

impl OpenCodeRunningSession {
    pub(crate) fn send_event(
        &mut self,
        event_type: &str,
        payload: serde_json::Value,
        occurred_at: chrono::DateTime<chrono::Utc>,
    ) {
        let Some(client) = self.api_client.as_ref() else {
            return;
        };
        let Some(token) = self.access_token.as_ref() else {
            return;
        };
        let Some(job_id) = self.job_id else {
            return;
        };

        self.event_sequence += 1;
        let wire = WireEvent {
            sequence: self.event_sequence,
            event_type: event_type.to_owned(),
            payload,
            occurred_at,
        };

        let c: Arc<ApiClient> = Arc::clone(client);
        let t: String = token.clone();
        let jid = job_id;
        let events = vec![wire];
        tokio::spawn(async move {
            match c.append_events(jid, &events, &t).await {
                Ok(resp) => {
                    if resp.should_cancel {
                        tracing::warn!(
                            work_run_id = %jid,
                            "server requested cancel via event response"
                        );
                    }
                }
                Err(e) => {
                    tracing::warn!(
                        work_run_id = %jid,
                        error = %e,
                        "failed to send event to server"
                    );
                }
            }
        });
    }
}
