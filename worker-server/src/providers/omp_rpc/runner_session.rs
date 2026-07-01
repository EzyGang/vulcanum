use std::future::Future;
use std::io::ErrorKind;
use std::pin::Pin;
use std::time::Duration;

use chrono::{DateTime, Utc};
use serde_json::Value;
use tokio::time::timeout;
use vulcanum_shared::runtime::agent::RunningSession;
use vulcanum_shared::runtime::errors::HarnessError;
use vulcanum_shared::runtime::types::{AgentEvent, SessionExport, SessionStatus};

use crate::providers::omp_rpc::session::OmpRpcRunningSession;

impl RunningSession for OmpRpcRunningSession {
    fn status(&self) -> SessionStatus {
        self.status.clone()
    }

    fn session_id(&self) -> Option<&str> {
        match self.session_id.is_empty() {
            true => None,
            false => Some(&self.session_id),
        }
    }

    fn agent_session_path(&self) -> Option<&str> {
        match self.session_path.is_empty() {
            true => None,
            false => Some(&self.session_path),
        }
    }

    fn agent_pid(&self) -> Option<u32> {
        self.child.id()
    }

    fn poll_event(&mut self) -> Pin<Box<dyn Future<Output = Option<AgentEvent>> + Send + '_>> {
        Box::pin(async move {
            loop {
                let frame = self.next_frame().await?;
                if let Some(event) = self.map_frame(frame) {
                    return Some(event);
                }
            }
        })
    }

    fn cancel(&mut self) -> Pin<Box<dyn Future<Output = Result<(), HarnessError>> + Send + '_>> {
        Box::pin(async move {
            let _ = self
                .send_command(serde_json::json!({"id": "abort-1", "type": "abort"}))
                .await;
            let _ = self.child.kill().await;
            self.status = SessionStatus::Cancelled;
            Ok(())
        })
    }

    fn export(
        &self,
    ) -> Pin<Box<dyn Future<Output = Result<SessionExport, HarnessError>> + Send + '_>> {
        Box::pin(async move {
            Ok(SessionExport {
                exit_code: match self.status {
                    SessionStatus::Completed => 0,
                    SessionStatus::Running | SessionStatus::Failed | SessionStatus::Cancelled => 1,
                },
                tokens_used: self.tokens.total,
                input_tokens: self.tokens.input,
                output_tokens: self.tokens.output,
                cache_read_tokens: self.tokens.cache_read,
                cache_write_tokens: self.tokens.cache_write,
                duration_ms: duration_ms(self.started_at),
                model_used: self.model_used.clone(),
                failure_payload: self.failure_payload.clone(),
            })
        })
    }

    fn export_messages(
        &self,
    ) -> Pin<Box<dyn Future<Output = Result<Option<Value>, HarnessError>> + Send + '_>> {
        Box::pin(async move {
            if self.session_path.is_empty() {
                return Ok(None);
            }
            let contents = match tokio::fs::read_to_string(&self.session_path).await {
                Ok(contents) => contents,
                Err(error) if error.kind() == ErrorKind::NotFound => return Ok(None),
                Err(error) => {
                    return Err(HarnessError::Crash(format!(
                        "failed to read OMP session JSONL at {}: {error}",
                        self.session_path
                    )));
                }
            };
            let mut entries = Vec::new();
            for line in contents.lines() {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }
                match serde_json::from_str::<Value>(trimmed) {
                    Ok(value) => entries.push(value),
                    Err(_) => entries.push(Value::String(trimmed.to_owned())),
                }
            }
            Ok(Some(serde_json::json!({
                "backend": "omp_rpc",
                "session_path": self.session_path,
                "entries": entries,
            })))
        })
    }

    fn wait(
        &mut self,
    ) -> Pin<Box<dyn Future<Output = Result<SessionExport, HarnessError>> + Send + '_>> {
        Box::pin(async move {
            let max_duration = Duration::from_secs(self.max_duration_secs);
            let result = timeout(max_duration, async {
                while !self.status.is_terminal() {
                    if self.poll_event().await.is_none() {
                        self.status = SessionStatus::Failed;
                        self.failure_payload = Some(serde_json::json!({
                            "reason": "omp_rpc_stream_closed",
                        }));
                    }
                }
            })
            .await;

            if result.is_err() {
                self.status = SessionStatus::Failed;
                self.failure_payload = Some(serde_json::json!({
                    "reason": "omp_rpc_timeout",
                    "max_duration_secs": self.max_duration_secs,
                }));
                let _ = self.child.kill().await;
            }

            self.export().await
        })
    }

    fn continue_with(
        &mut self,
        prompt: &str,
    ) -> Pin<Box<dyn Future<Output = Result<(), HarnessError>> + Send + '_>> {
        let prompt = prompt.to_owned();
        Box::pin(async move {
            self.status = SessionStatus::Running;
            if let Err(error) = self
                .send_command(serde_json::json!({
                    "id": "prompt-continue",
                    "type": "prompt",
                    "message": prompt,
                }))
                .await
            {
                self.status = SessionStatus::Failed;
                return Err(error);
            }
            if let Err(error) = self.wait_for_response("prompt-continue", "prompt").await {
                self.status = SessionStatus::Failed;
                return Err(error);
            }
            Ok(())
        })
    }
}

fn duration_ms(started_at: DateTime<Utc>) -> u64 {
    Utc::now()
        .signed_duration_since(started_at)
        .num_milliseconds()
        .max(0) as u64
}
