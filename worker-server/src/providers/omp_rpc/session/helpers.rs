use chrono::Utc;
use serde_json::Value;

use vulcanum_shared::runtime::errors::HarnessError;
use vulcanum_shared::runtime::types::{AgentEvent, IsolatedEnvironment};

use crate::providers::omp_rpc::{VULCANUM_OMP_MODEL_ENV, VULCANUM_OMP_PROVIDER_ENV};

pub(super) fn event(event_type: &str, payload: Value) -> AgentEvent {
    AgentEvent {
        event_type: event_type.to_owned(),
        payload,
        timestamp: Utc::now(),
    }
}

pub(super) fn state_string<'a>(data: &'a Value, field: &str) -> Option<&'a str> {
    match data.get(field).and_then(Value::as_str) {
        Some(value) if !value.is_empty() => Some(value),
        _ => None,
    }
}

pub(super) fn configured_model(env: &IsolatedEnvironment) -> Option<&str> {
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

pub(super) fn model_used_name(env: &IsolatedEnvironment, model: &str) -> String {
    if model.contains('/') {
        return model.to_owned();
    }

    match configured_provider(env) {
        Some(provider) => format!("{provider}/{model}"),
        None => model.to_owned(),
    }
}

pub(crate) fn host_session_path(env: &IsolatedEnvironment, session_path: &str) -> String {
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

pub(super) fn is_response_for(frame: &Value, id: &str, command: &str) -> bool {
    frame.get("type").and_then(Value::as_str) == Some("response")
        && frame.get("id").and_then(Value::as_str) == Some(id)
        && frame.get("command").and_then(Value::as_str) == Some(command)
}

pub(super) fn response_result(frame: Value) -> Result<Value, HarnessError> {
    if frame.get("success").and_then(Value::as_bool) == Some(true) {
        return Ok(frame);
    }
    let error = frame
        .get("error")
        .and_then(Value::as_str)
        .unwrap_or("omp rpc command failed");
    Err(HarnessError::ServerLaunch(error.to_owned()))
}

pub(super) fn format_startup_failure(reason: &str, status: Option<&str>, stderr: &str) -> String {
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

pub(super) fn token_count(tokens: &Value, field: &str) -> Option<u64> {
    tokens.get(field).and_then(Value::as_u64)
}
