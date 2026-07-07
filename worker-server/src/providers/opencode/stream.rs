use chrono::Utc;

use vulcanum_shared::runtime::types::{AgentEvent, SessionStatus};

use super::api;
use super::events;
use super::runner::OpenCodeRunningSession;

impl OpenCodeRunningSession {
    pub(super) async fn reconcile_interrupted_stream(
        &mut self,
        reason: &str,
    ) -> Option<AgentEvent> {
        let status_map = match api::get_session_status(&self.client).await {
            Ok(status_map) => status_map,
            Err(e) => {
                tracing::warn!(
                    session_id = %self.session_id,
                    reason,
                    error = %e,
                    "failed to check opencode session status after event stream interruption"
                );
                return Some(self.failure_event(serde_json::json!({
                    "reason": reason,
                    "status_check_error": e.to_string(),
                })));
            }
        };

        match status_map.get(&self.session_id) {
            Some(api::OpenCodeSessionStatus::Idle) => {
                tracing::info!(
                    session_id = %self.session_id,
                    reason,
                    "opencode session is idle after event stream interruption"
                );
                self.status = SessionStatus::Completed;
                Some(AgentEvent {
                    event_type: "session.completed".to_owned(),
                    payload: serde_json::json!({"reason": reason, "status": "idle"}),
                    timestamp: Utc::now(),
                })
            }
            Some(api::OpenCodeSessionStatus::Busy) if reason == "stall_detected" => {
                tracing::info!(
                    session_id = %self.session_id,
                    reason,
                    "opencode session is still busy after event stall"
                );
                self.status = SessionStatus::Running;
                Some(AgentEvent {
                    event_type: "session.still_busy".to_owned(),
                    payload: serde_json::json!({"reason": reason, "status": "busy"}),
                    timestamp: Utc::now(),
                })
            }
            Some(api::OpenCodeSessionStatus::Busy) => self.reconnect_stream(reason, None).await,
            Some(api::OpenCodeSessionStatus::Retry {
                attempt,
                message,
                next,
            }) => {
                self.reconnect_stream(
                    reason,
                    Some(serde_json::json!({
                        "attempt": attempt,
                        "message": message,
                        "next": next,
                    })),
                )
                .await
            }
            None => Some(self.failure_event(serde_json::json!({
                "reason": reason,
                "status_check_error": "session_missing",
            }))),
        }
    }

    async fn reconnect_stream(
        &mut self,
        reason: &str,
        retry: Option<serde_json::Value>,
    ) -> Option<AgentEvent> {
        match events::connect_events(&self.client).await {
            Ok(stream) => {
                tracing::info!(
                    session_id = %self.session_id,
                    reason,
                    "reconnected opencode event stream"
                );
                self.event_stream = Some(stream);
                self.status = SessionStatus::Running;
                Some(AgentEvent {
                    event_type: "session.stream_reconnected".to_owned(),
                    payload: serde_json::json!({"reason": reason, "retry": retry}),
                    timestamp: Utc::now(),
                })
            }
            Err(e) => {
                tracing::warn!(
                    session_id = %self.session_id,
                    reason,
                    error = %e,
                    "failed to reconnect opencode event stream"
                );
                Some(self.failure_event(serde_json::json!({
                    "reason": reason,
                    "stream_reconnect_error": e.to_string(),
                })))
            }
        }
    }

    pub(super) fn failure_event(&mut self, payload: serde_json::Value) -> AgentEvent {
        self.status = SessionStatus::Failed;
        self.failure_payload = Some(payload.clone());
        AgentEvent {
            event_type: "session.failed".to_owned(),
            payload,
            timestamp: Utc::now(),
        }
    }
}
