use std::future::Future;
use std::pin::Pin;
use std::time::Duration;

use chrono::Utc;
use tokio::time::timeout;

use vulcanum_shared::runtime::agent::RunningSession;
use vulcanum_shared::runtime::errors::HarnessError;
use vulcanum_shared::runtime::types::{AgentEvent, SessionExport, SessionStatus};

use super::api;
use super::events;
use super::runner::OpenCodeRunningSession;

const STALL_TIMEOUT_SECS: u64 = 300;

impl OpenCodeRunningSession {
    async fn reconcile_interrupted_stream(&mut self, reason: &str) -> Option<AgentEvent> {
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

    fn failure_event(&mut self, payload: serde_json::Value) -> AgentEvent {
        self.status = SessionStatus::Failed;
        self.failure_payload = Some(payload.clone());
        AgentEvent {
            event_type: "session.failed".to_owned(),
            payload,
            timestamp: Utc::now(),
        }
    }
}

impl RunningSession for OpenCodeRunningSession {
    fn status(&self) -> SessionStatus {
        self.status.clone()
    }

    fn session_id(&self) -> Option<&str> {
        Some(&self.session_id)
    }

    fn agent_pid(&self) -> Option<u32> {
        self.host_pid
    }

    fn agent_base_url(&self) -> Option<&str> {
        Some(self.client.base_url())
    }

    fn host_server_info(&self) -> Option<(u32, u16)> {
        match (self.host_pid, self.host_port) {
            (Some(pid), Some(port)) => Some((pid, port)),
            _ => None,
        }
    }

    fn poll_event(&mut self) -> Pin<Box<dyn Future<Output = Option<AgentEvent>> + Send + '_>> {
        Box::pin(async move {
            loop {
                let elapsed = (Utc::now() - self.started_at).num_seconds() as u64;
                if elapsed >= self.max_duration_secs {
                    return Some(
                        self.failure_event(serde_json::json!({"reason": "max_duration_exceeded"})),
                    );
                }

                let result = {
                    let stream = match self.event_stream.as_mut() {
                        Some(s) => s,
                        None => return None,
                    };

                    timeout(Duration::from_secs(STALL_TIMEOUT_SECS), stream.next_event()).await
                };

                match result {
                    Ok(Some(sse)) => {
                        tracing::debug!(
                            session_id = %self.session_id,
                            event_type = %sse.event_type,
                            properties = %sse.properties,
                            "sse event received"
                        );
                        let mapped = super::event_mapper::map_event(&sse);
                        let last = mapped.last().cloned();
                        for event in &mapped {
                            match event.event_type.as_str() {
                                "session.completed" => self.status = SessionStatus::Completed,
                                "session.failed" => {
                                    tracing::warn!(
                                        session_id = %self.session_id,
                                        payload = %event.payload,
                                        "opencode session failed"
                                    );
                                    self.failure_payload = Some(event.payload.clone());
                                    self.status = SessionStatus::Failed;
                                }
                                "turn.failed" => {
                                    tracing::warn!(
                                        session_id = %self.session_id,
                                        payload = %event.payload,
                                        "opencode turn failed"
                                    );
                                }
                                _ => (),
                            }
                        }
                        match last {
                            Some(event) => return Some(event),
                            None => continue,
                        }
                    }
                    Ok(None) => {
                        tracing::warn!(
                            session_id = %self.session_id,
                            "opencode event stream ended before terminal session event"
                        );
                        return self.reconcile_interrupted_stream("stream_ended").await;
                    }
                    Err(_) => {
                        tracing::info!(
                            session_id = %self.session_id,
                            stall_timeout_secs = STALL_TIMEOUT_SECS,
                            "session stalled, no events received"
                        );
                        return self.reconcile_interrupted_stream("stall_detected").await;
                    }
                }
            }
        })
    }

    fn cancel(&mut self) -> Pin<Box<dyn Future<Output = Result<(), HarnessError>> + Send + '_>> {
        let session_id = self.session_id.clone();
        let client = self.client.clone();
        Box::pin(async move {
            api::abort_session(&client, &session_id).await?;
            self.status = SessionStatus::Cancelled;
            self.kill_server().await;
            Ok(())
        })
    }

    fn export(
        &self,
    ) -> Pin<Box<dyn Future<Output = Result<SessionExport, HarnessError>> + Send + '_>> {
        let client = self.client.clone();
        let session_id = self.session_id.clone();
        let current_status = self.status.clone();
        let started_at = self.started_at;
        let failure_payload = self.failure_payload.clone();

        Box::pin(async move {
            let info = api::get_session_info(&client, &session_id).await?;

            let tokens = &info.tokens;
            let input_tokens = tokens.input.unwrap_or(0);
            let output_tokens = tokens.output.unwrap_or(0);
            let cache_read_tokens = tokens.cache.as_ref().and_then(|c| c.read).unwrap_or(0);
            let cache_write_tokens = tokens.cache.as_ref().and_then(|c| c.write).unwrap_or(0);
            let tokens_used = input_tokens + output_tokens + cache_read_tokens + cache_write_tokens;

            let model_used = info
                .model
                .as_ref()
                .and_then(|m| match (&m.provider_id, &m.id) {
                    (Some(p), Some(m_id)) => Some(format!("{p}/{m_id}")),
                    (None, Some(m_id)) => Some(m_id.to_owned()),
                    _ => None,
                });

            let elapsed_ms = (Utc::now() - started_at).num_milliseconds() as u64;
            let exit_code = match current_status {
                SessionStatus::Failed => 1,
                SessionStatus::Cancelled => 2,
                _ => 0,
            };

            Ok(SessionExport {
                exit_code,
                tokens_used,
                input_tokens,
                output_tokens,
                cache_read_tokens,
                cache_write_tokens,
                duration_ms: elapsed_ms,
                model_used,
                failure_payload,
            })
        })
    }

    fn export_messages(
        &self,
    ) -> Pin<Box<dyn Future<Output = Result<Option<serde_json::Value>, HarnessError>> + Send + '_>>
    {
        let client = self.client.clone();
        let session_id = self.session_id.clone();
        Box::pin(async move {
            let messages = api::get_session_messages(&client, &session_id, None).await?;
            Ok(Some(messages))
        })
    }

    fn continue_with(
        &mut self,
        prompt: &str,
    ) -> Pin<Box<dyn Future<Output = Result<(), HarnessError>> + Send + '_>> {
        let client = self.client.clone();
        let session_id = self.session_id.clone();
        let prompt = prompt.to_owned();

        Box::pin(async move {
            api::send_message_async(&client, &session_id, &prompt).await?;

            self.event_stream = None;
            let stream = events::connect_events(&client).await?;
            self.event_stream = Some(stream);
            self.status = SessionStatus::Running;

            Ok(())
        })
    }

    fn wait(
        &mut self,
    ) -> Pin<Box<dyn Future<Output = Result<SessionExport, HarnessError>> + Send + '_>> {
        Box::pin(async move {
            loop {
                if self.status.is_terminal() {
                    break;
                }
                match self.poll_event().await {
                    Some(_) => continue,
                    None => {
                        self.status = SessionStatus::Failed;
                        break;
                    }
                }
            }
            self.export().await
        })
    }
}
