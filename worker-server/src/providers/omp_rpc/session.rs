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

const STARTUP_TIMEOUT_SECS: u64 = 30;

pub(crate) struct OmpRpcRunningSession {
    pub(super) child: Child,
    pub(super) stdin: ChildStdin,
    pub(super) frames: mpsc::Receiver<Value>,
    pub(super) pending: VecDeque<Value>,
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
        max_duration_secs: u64,
    ) -> Self {
        Self {
            child,
            stdin,
            frames,
            pending: VecDeque::new(),
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
        let frame = timeout(Duration::from_secs(STARTUP_TIMEOUT_SECS), self.next_frame())
            .await
            .map_err(|_| HarnessError::ServerLaunch("omp rpc did not become ready".to_owned()))?
            .ok_or_else(|| HarnessError::ServerLaunch("omp rpc exited before ready".to_owned()))?;
        match frame.get("type").and_then(Value::as_str) {
            Some("ready") => Ok(()),
            _ => Err(HarnessError::ServerLaunch(format!(
                "unexpected omp startup frame: {frame}"
            ))),
        }
    }

    pub(crate) async fn refresh_state(&mut self) -> Result<(), HarnessError> {
        self.send_command(serde_json::json!({"id": "state-1", "type": "get_state"}))
            .await?;
        let response = self.wait_for_response("state-1", "get_state").await?;
        let data = response.get("data").unwrap_or(&Value::Null);
        self.session_id = data
            .get("sessionId")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned();
        self.session_path = data
            .get("sessionFile")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned();
        self.model_used = data.get("model").map(Value::to_string);
        Ok(())
    }

    pub(crate) async fn wait_for_response(
        &mut self,
        id: &str,
        command: &str,
    ) -> Result<Value, HarnessError> {
        loop {
            let frame = self
                .next_frame()
                .await
                .ok_or_else(|| HarnessError::ServerLaunch("omp rpc stream closed".to_owned()))?;
            let is_response = frame.get("type").and_then(Value::as_str) == Some("response")
                && frame.get("id").and_then(Value::as_str) == Some(id)
                && frame.get("command").and_then(Value::as_str) == Some(command);
            if !is_response {
                self.pending.push_back(frame);
                continue;
            }
            if frame.get("success").and_then(Value::as_bool) == Some(true) {
                return Ok(frame);
            }
            let error = frame
                .get("error")
                .and_then(Value::as_str)
                .unwrap_or("omp rpc command failed");
            return Err(HarnessError::ServerLaunch(error.to_owned()));
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

fn get_u64_path(value: &Value, path: &[&str]) -> Option<u64> {
    let mut current = value;
    for key in path {
        current = current.get(key)?;
    }
    current.as_u64()
}
