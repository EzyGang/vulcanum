use std::collections::VecDeque;
use std::time::Duration;

use chrono::{DateTime, Utc};
use serde_json::Value;
use tokio::io::AsyncWriteExt;
use tokio::process::{Child, ChildStdin};
use tokio::sync::mpsc;
use tokio::time::timeout;
use vulcanum_shared::runtime::errors::HarnessError;
use vulcanum_shared::runtime::types::{AgentEvent, IsolatedEnvironment, SessionStatus};

use crate::providers::omp_rpc::process::ProcessOutputBuffer;
use crate::providers::omp_rpc::{VULCANUM_OMP_MODEL_ENV, VULCANUM_OMP_PROVIDER_ENV};

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

    pub(crate) fn set_configured_model(&mut self, env: &IsolatedEnvironment) {
        if let Some(model) = configured_model(env) {
            self.model_used
                .get_or_insert_with(|| model_used_name(env, model));
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

    pub(crate) async fn refresh_state(
        &mut self,
        env: &IsolatedEnvironment,
    ) -> Result<(), HarnessError> {
        self.send_command(serde_json::json!({"id": "state-1", "type": "get_state"}))
            .await?;
        let response = self.wait_for_response("state-1", "get_state").await?;
        let data = response.get("data").unwrap_or(&Value::Null);
        if let Some(session_id) = state_string(data, "sessionId") {
            self.session_id = session_id.to_owned();
        }
        if let Some(session_path) = state_string(data, "sessionFile") {
            self.session_path = host_session_path(env, session_path);
        }
        if let Some(model) = state_string(data, "model") {
            self.model_used = Some(model_used_name(env, model));
        }
        self.set_configured_model(env);
        Ok(())
    }

    pub(crate) async fn refresh_usage_stats(&mut self) -> Result<(), HarnessError> {
        self.send_command(serde_json::json!({"id": "stats-1", "type": "get_session_stats"}))
            .await?;
        let response = self
            .wait_for_response("stats-1", "get_session_stats")
            .await?;
        let data = response.get("data").unwrap_or(&Value::Null);
        self.update_usage(data);
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
                self.update_usage(frame.get("data").unwrap_or(&Value::Null));
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

    fn update_usage(&mut self, data: &Value) {
        let Some(tokens) = data.get("tokens") else {
            return;
        };

        self.tokens.input = token_count(tokens, "input").unwrap_or(self.tokens.input);
        self.tokens.output = token_count(tokens, "output").unwrap_or(self.tokens.output);
        self.tokens.cache_read = token_count(tokens, "cacheRead").unwrap_or(self.tokens.cache_read);
        self.tokens.cache_write =
            token_count(tokens, "cacheWrite").unwrap_or(self.tokens.cache_write);
        self.tokens.total = token_count(tokens, "total").unwrap_or(
            self.tokens.input
                + self.tokens.output
                + self.tokens.cache_read
                + self.tokens.cache_write,
        );
    }
}

fn event(event_type: &str, payload: Value) -> AgentEvent {
    AgentEvent {
        event_type: event_type.to_owned(),
        payload,
        timestamp: Utc::now(),
    }
}

fn state_string<'a>(data: &'a Value, field: &str) -> Option<&'a str> {
    match data.get(field).and_then(Value::as_str) {
        Some(value) if !value.is_empty() => Some(value),
        _ => None,
    }
}

fn configured_model(env: &IsolatedEnvironment) -> Option<&str> {
    env.env_vars
        .get(VULCANUM_OMP_MODEL_ENV)
        .map(String::as_str)
        .filter(|model| !model.is_empty())
}

fn configured_provider(env: &IsolatedEnvironment) -> Option<&str> {
    env.env_vars
        .get(VULCANUM_OMP_PROVIDER_ENV)
        .map(String::as_str)
        .filter(|provider| !provider.is_empty())
}

fn model_used_name(env: &IsolatedEnvironment, model: &str) -> String {
    if model.contains('/') {
        return model.to_owned();
    }

    match configured_provider(env) {
        Some(provider) => format!("{provider}/{model}"),
        None => model.to_owned(),
    }
}

pub(super) fn host_session_path(env: &IsolatedEnvironment, session_path: &str) -> String {
    if env.container_name.is_none() {
        return session_path.to_owned();
    }

    let Some(relative_path) = session_path.strip_prefix("/workdir") else {
        return session_path.to_owned();
    };
    let mut host_path = env.workdir.clone();
    for component in relative_path.trim_start_matches('/').split('/') {
        if !component.is_empty() {
            host_path.push(component);
        }
    }
    host_path.to_string_lossy().to_string()
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

fn token_count(tokens: &Value, field: &str) -> Option<u64> {
    tokens.get(field).and_then(Value::as_u64)
}
