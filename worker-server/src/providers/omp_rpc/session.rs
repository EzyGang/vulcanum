use std::collections::VecDeque;
use std::time::Duration;

use chrono::{DateTime, Utc};
use serde_json::Value;
use tokio::io::AsyncWriteExt;
use tokio::process::{Child, ChildStdin};
use tokio::sync::mpsc;
use tokio::time::timeout;
use vulcanum_shared::runtime::errors::HarnessError;
use vulcanum_shared::runtime::types::{AgentEvent, SessionStatus};

use crate::providers::omp_rpc::process::ProcessOutputBuffer;

const STARTUP_TIMEOUT_SECS: u64 = 180;

pub(crate) struct OmpRpcRunningSession {
    pub(super) child: Child,
    pub(super) stdin: ChildStdin,
    pub(super) frames: mpsc::Receiver<Value>,
    pub(super) pending: VecDeque<Value>,
    pub(super) stderr: ProcessOutputBuffer,
    pub(super) session_id: String,
    pub(super) session_path: String,
    pub(super) status: SessionStatus,
    pub(super) started_at: DateTime<Utc>,
    pub(super) failure_payload: Option<Value>,
    pub(super) tokens: OmpTokenUsage,
    pub(super) model_used: Option<String>,
    pub(super) max_duration_secs: u64,
}

#[derive(Default, Debug, Clone, Copy)]
pub(super) struct OmpTokenUsage {
    pub(super) total: u64,
    pub(super) input: u64,
    pub(super) output: u64,
    pub(super) cache_read: u64,
    pub(super) cache_write: u64,
}

impl OmpRpcRunningSession {
    pub(crate) fn new(
        child: Child,
        stdin: ChildStdin,
        frames: mpsc::Receiver<Value>,
        stderr: ProcessOutputBuffer,
        max_duration_secs: u64,
    ) -> Self {
        Self {
            child,
            stdin,
            frames,
            pending: VecDeque::new(),
            stderr,
            session_id: String::new(),
            session_path: String::new(),
            status: SessionStatus::Running,
            started_at: Utc::now(),
            failure_payload: None,
            tokens: OmpTokenUsage::default(),
            model_used: None,
            max_duration_secs,
        }
    }

    pub(crate) async fn wait_ready(&mut self) -> Result<(), HarnessError> {
        let result = timeout(Duration::from_secs(STARTUP_TIMEOUT_SECS), async {
            loop {
                match self.frames.recv().await {
                    Some(frame) => match frame.get("type").and_then(Value::as_str) {
                        Some("ready") => return Ok(()),
                        _ => self.pending.push_back(frame),
                    },
                    None => return Err(()),
                }
            }
        })
        .await;

        match result {
            Ok(Ok(())) => Ok(()),
            Ok(Err(())) => Err(self.startup_error("omp rpc exited before ready")),
            Err(_) => Err(self.startup_error("omp rpc did not become ready")),
        }
    }

    pub(crate) async fn refresh_state(&mut self) -> Result<(), HarnessError> {
        self.send_command(serde_json::json!({"id": "state-1", "type": "get_state"}))
            .await?;
        let response = self.wait_for_response("state-1", "get_state").await?;
        let data = response.get("data").unwrap_or(&Value::Null);
        if let Some(session_id) = required_state_string(data, "sessionId") {
            self.session_id = session_id.to_owned();
        }
        if let Some(session_path) = required_state_string(data, "sessionFile") {
            self.session_path = session_path.to_owned();
        }
        if let Some(model) = required_state_string(data, "model") {
            self.model_used = Some(model.to_owned());
        }
        Ok(())
    }

    pub(crate) async fn wait_for_response(
        &mut self,
        id: &str,
        command: &str,
    ) -> Result<Value, HarnessError> {
        if let Some(frame) = self.pop_pending_response(id, command) {
            return response_result(frame);
        }

        loop {
            let frame =
                self.frames.recv().await.ok_or_else(|| {
                    HarnessError::ServerLaunch("omp rpc stream closed".to_owned())
                })?;
            if is_response_for(&frame, id, command) {
                return response_result(frame);
            }
            self.pending.push_back(frame);
        }
    }

    pub(crate) async fn send_command(&mut self, command: Value) -> Result<(), HarnessError> {
        let mut line = serde_json::to_vec(&command)
            .map_err(|e| HarnessError::Crash(format!("failed to encode OMP RPC command: {e}")))?;
        line.push(b'\n');
        self.stdin
            .write_all(&line)
            .await
            .map_err(|e| HarnessError::Crash(format!("failed to write OMP RPC command: {e}")))?;
        self.stdin
            .flush()
            .await
            .map_err(|e| HarnessError::Crash(format!("failed to flush OMP RPC command: {e}")))?;
        Ok(())
    }

    fn startup_error(&mut self, reason: &str) -> HarnessError {
        let status = match self.child.try_wait() {
            Ok(Some(status)) => Some(status),
            Ok(None) => None,
            Err(error) => {
                return HarnessError::ServerLaunch(format!(
                    "{reason}; failed to inspect omp rpc process: {error}"
                ));
            }
        };
        let status_text = status.as_ref().map(|status| status.to_string());
        let stderr = self.stderr.tail();
        HarnessError::ServerLaunch(format_startup_failure(
            reason,
            status_text.as_deref(),
            &stderr,
        ))
    }

    fn pop_pending_response(&mut self, id: &str, command: &str) -> Option<Value> {
        let index = self
            .pending
            .iter()
            .position(|frame| is_response_for(frame, id, command))?;
        self.pending.remove(index)
    }

    pub(super) async fn next_frame(&mut self) -> Option<Value> {
        match self.pending.pop_front() {
            Some(frame) => Some(frame),
            None => self.frames.recv().await,
        }
    }

    pub(super) fn map_frame(&mut self, frame: Value) -> Option<AgentEvent> {
        let frame_type = frame
            .get("type")
            .and_then(Value::as_str)
            .unwrap_or("unknown");
        match frame_type {
            "agent_end" => {
                self.status = SessionStatus::Completed;
                self.update_usage(&frame);
                Some(event("session.completed", frame))
            }
            "tool_execution_start" => Some(event("tool.called", frame)),
            "tool_execution_update" => Some(event("tool.progress", frame)),
            "tool_execution_end" => Some(event("tool.completed", frame)),
            "response" if frame.get("success").and_then(Value::as_bool) == Some(false) => {
                self.status = SessionStatus::Failed;
                self.failure_payload = Some(frame.clone());
                Some(event("session.failed", frame))
            }
            "message_update"
            | "available_commands_update"
            | "prompt_result"
            | "turn_start"
            | "message_start"
            | "message_end" => None,
            other => Some(event(&format!("omp.{other}"), frame)),
        }
    }

    fn update_usage(&mut self, frame: &Value) {
        let Some(telemetry) = frame.get("telemetry") else {
            return;
        };
        let usage = telemetry.get("usage").unwrap_or(telemetry);
        self.tokens.input = get_u64_path(usage, &["inputTokens"])
            .or_else(|| get_u64_path(usage, &["input_tokens"]))
            .unwrap_or(self.tokens.input);
        self.tokens.output = get_u64_path(usage, &["outputTokens"])
            .or_else(|| get_u64_path(usage, &["output_tokens"]))
            .unwrap_or(self.tokens.output);
        self.tokens.cache_read = get_u64_path(usage, &["cacheReadTokens"])
            .or_else(|| get_u64_path(usage, &["cache_read_tokens"]))
            .unwrap_or(self.tokens.cache_read);
        self.tokens.cache_write = get_u64_path(usage, &["cacheWriteTokens"])
            .or_else(|| get_u64_path(usage, &["cache_write_tokens"]))
            .unwrap_or(self.tokens.cache_write);
        self.tokens.total = get_u64_path(usage, &["totalTokens"])
            .or_else(|| get_u64_path(usage, &["tokensUsed"]))
            .unwrap_or(self.tokens.input + self.tokens.output);
    }
}

fn event(event_type: &str, payload: Value) -> AgentEvent {
    AgentEvent {
        event_type: event_type.to_owned(),
        payload,
        timestamp: Utc::now(),
    }
}

fn required_state_string<'a>(data: &'a Value, field: &str) -> Option<&'a str> {
    match data.get(field).and_then(Value::as_str) {
        Some(value) if !value.is_empty() => Some(value),
        _ => {
            tracing::warn!(field, response = %data, "OMP get_state response missing expected field");
            None
        }
    }
}

fn is_response_for(frame: &Value, id: &str, command: &str) -> bool {
    frame.get("type").and_then(Value::as_str) == Some("response")
        && frame.get("id").and_then(Value::as_str) == Some(id)
        && frame.get("command").and_then(Value::as_str) == Some(command)
}

fn response_result(frame: Value) -> Result<Value, HarnessError> {
    if frame.get("success").and_then(Value::as_bool) == Some(true) {
        return Ok(frame);
    }
    let error = frame
        .get("error")
        .and_then(Value::as_str)
        .unwrap_or("omp rpc command failed");
    Err(HarnessError::ServerLaunch(error.to_owned()))
}

fn format_startup_failure(reason: &str, status: Option<&str>, stderr: &str) -> String {
    let mut message = reason.to_owned();
    if let Some(status) = status {
        message.push_str("; process ");
        message.push_str(status);
    }
    let stderr = stderr.trim();
    if !stderr.is_empty() {
        message.push_str("; stderr: ");
        message.push_str(stderr);
    }
    message
}

fn get_u64_path(value: &Value, path: &[&str]) -> Option<u64> {
    let mut current = value;
    for key in path {
        current = current.get(key)?;
    }
    current.as_u64()
}
