use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

use chrono::Utc;
use tokio::time::timeout;
use uuid::Uuid;

use vulcanum_shared::client::ApiClient;
use vulcanum_shared::runtime::agent::RunningSession;
use vulcanum_shared::runtime::errors::HarnessError;
use vulcanum_shared::runtime::types::{AgentEvent, SessionExport, SessionStatus};

use crate::runtime::client::events;
use crate::runtime::client::session;
use crate::runtime::export;
use crate::runtime::mapping;
use crate::runtime::runner::OpenCodeRunningSession;

const STALL_TIMEOUT_SECS: u64 = 300;

impl RunningSession for OpenCodeRunningSession {
    fn status(&self) -> SessionStatus {
        self.status.clone()
    }

    fn session_id(&self) -> Option<&str> {
        Some(&self.session_id)
    }

    fn set_event_reporter(&mut self, client: Arc<ApiClient>, token: String, job_id: Uuid) {
        self.api_client = Some(client);
        self.access_token = Some(token);
        self.job_id = Some(job_id);
    }

    fn poll_event(&mut self) -> Pin<Box<dyn Future<Output = Option<AgentEvent>> + Send + '_>> {
        Box::pin(async move {
            let elapsed = (Utc::now() - self.started_at).num_seconds() as u64;
            if elapsed >= self.max_duration_secs {
                self.status = SessionStatus::Failed;
                return Some(AgentEvent {
                    event_type: "session.failed".to_owned(),
                    payload: serde_json::json!({"reason": "max_duration_exceeded"}),
                    timestamp: Utc::now(),
                });
            }

            let stream = match self.event_stream.as_mut() {
                Some(s) => s,
                None => return None,
            };

            let result =
                timeout(Duration::from_secs(STALL_TIMEOUT_SECS), stream.next_event()).await;

            match result {
                Ok(Some(sse)) => {
                    tracing::debug!(
                        session_id = %self.session_id,
                        event_type = %sse.event_type,
                        "sse event received"
                    );
                    let mapped = mapping::map_event(&sse);
                    let last = mapped.last().cloned();
                    for event in &mapped {
                        match event.event_type.as_str() {
                            "session.completed" => self.status = SessionStatus::Completed,
                            "session.failed" => self.status = SessionStatus::Failed,
                            _ => (),
                        }
                        if super::HIGH_LEVEL_EVENT_TYPES.contains(&event.event_type.as_str()) {
                            self.send_event(&event.event_type, event.payload.clone());
                        }
                    }
                    last
                }
                Ok(None) => {
                    tracing::info!(
                        session_id = %self.session_id,
                        "event stream ended, session failed"
                    );
                    self.status = SessionStatus::Failed;
                    Some(AgentEvent {
                        event_type: "session.failed".to_owned(),
                        payload: serde_json::json!({"reason": "stream_ended"}),
                        timestamp: Utc::now(),
                    })
                }
                Err(_) => {
                    tracing::warn!(
                        session_id = %self.session_id,
                        stall_timeout_secs = STALL_TIMEOUT_SECS,
                        "session stalled, no events received"
                    );
                    self.status = SessionStatus::Failed;
                    Some(AgentEvent {
                        event_type: "stall.detected".to_owned(),
                        payload: serde_json::json!({"timeout_secs": STALL_TIMEOUT_SECS}),
                        timestamp: Utc::now(),
                    })
                }
            }
        })
    }

    fn cancel(&mut self) -> Pin<Box<dyn Future<Output = Result<(), HarnessError>> + Send + '_>> {
        let session_id = self.session_id.clone();
        let client = self.client.clone();
        Box::pin(async move {
            crate::runtime::client::session::abort_session(&client, &session_id).await?;
            self.status = SessionStatus::Cancelled;
            self.kill_server().await;
            Ok(())
        })
    }

    fn export(
        &self,
    ) -> Pin<Box<dyn Future<Output = Result<SessionExport, HarnessError>> + Send + '_>> {
        let is_container = self.is_container;
        let container_name = self.container_name.clone();
        let session_id = self.session_id.clone();
        let current_status = self.status.clone();
        let started_at = self.started_at;

        Box::pin(async move {
            let raw = if is_container {
                let name = container_name.as_deref().ok_or_else(|| {
                    HarnessError::OutputParse("container name missing for export".to_owned())
                })?;
                let output = tokio::process::Command::new("docker")
                    .args(["exec", name, "opencode", "export", &session_id])
                    .stdout(std::process::Stdio::piped())
                    .stderr(std::process::Stdio::null())
                    .output()
                    .await
                    .map_err(|e| {
                        HarnessError::OutputParse(format!("docker exec export failed: {e}"))
                    })?;
                String::from_utf8_lossy(&output.stdout).to_string()
            } else {
                let output = tokio::process::Command::new("opencode")
                    .args(["export", &session_id])
                    .stdout(std::process::Stdio::piped())
                    .stderr(std::process::Stdio::null())
                    .output()
                    .await
                    .map_err(|e| {
                        HarnessError::OutputParse(format!("opencode export failed: {e}"))
                    })?;
                String::from_utf8_lossy(&output.stdout).to_string()
            };

            let session_export = export::parse_export(&raw)?;
            let elapsed_ms = (Utc::now() - started_at).num_milliseconds() as u64;
            let exit_code = match current_status {
                SessionStatus::Failed => 1,
                SessionStatus::Cancelled => 2,
                _ => session_export.exit_code,
            };

            Ok(SessionExport {
                duration_ms: elapsed_ms,
                exit_code,
                ..session_export
            })
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
            session::send_message_async(&client, &session_id, &prompt).await?;

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
