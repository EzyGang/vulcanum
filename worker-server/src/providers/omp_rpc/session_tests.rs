use std::collections::HashMap;
use std::error::Error;
use std::path::PathBuf;
use std::process::Stdio;
use std::time::Duration;

use serde_json::Value;
use tokio::process::Command;
use tokio::sync::mpsc;
use tokio::time::timeout;
use vulcanum_shared::runtime::agent::RunningSession;
use vulcanum_shared::runtime::types::{IsolatedEnvironment, ResourceLimits};

use crate::providers::omp_rpc::process::ProcessOutputBuffer;
use crate::providers::omp_rpc::session::{host_session_path, OmpRpcRunningSession};
use crate::providers::omp_rpc::{VULCANUM_OMP_MODEL_ENV, VULCANUM_OMP_PROVIDER_ENV};

#[tokio::test]
async fn wait_ready_preserves_startup_frames_until_ready() -> Result<(), Box<dyn Error>> {
    let stderr = ProcessOutputBuffer::default();
    let (mut session, tx) = test_session(stderr).await?;
    tx.send(serde_json::json!({"type": "extension_ui_request"}))
        .await?;
    tx.send(serde_json::json!({"type": "ready"})).await?;

    session.wait_ready().await?;

    assert_eq!(session.pending.len(), 1);
    assert_eq!(
        session
            .pending
            .front()
            .and_then(|frame| frame.get("type"))
            .and_then(Value::as_str),
        Some("extension_ui_request")
    );
    Ok(())
}

#[tokio::test]
async fn wait_for_response_keeps_pending_events_and_reads_new_frames() -> Result<(), Box<dyn Error>>
{
    let stderr = ProcessOutputBuffer::default();
    let (mut session, tx) = test_session(stderr).await?;
    tx.send(serde_json::json!({"type": "message_update"}))
        .await?;
    tx.send(serde_json::json!({
        "type": "response",
        "id": "state-1",
        "command": "get_state",
        "success": true,
        "data": {"sessionId": "abc"}
    }))
    .await?;

    let response = timeout(
        Duration::from_secs(1),
        session.wait_for_response("state-1", "get_state"),
    )
    .await??;

    assert_eq!(
        response
            .get("data")
            .and_then(|data| data.get("sessionId"))
            .and_then(Value::as_str),
        Some("abc")
    );
    assert_eq!(session.pending.len(), 1);
    Ok(())
}

#[tokio::test]
async fn wait_for_response_times_out_when_command_never_answers() -> Result<(), Box<dyn Error>> {
    let stderr = ProcessOutputBuffer::default();
    let (mut session, _tx) = test_session(stderr).await?;

    let result = session
        .wait_for_response_with_timeout("missing", "get_state", Duration::from_millis(1))
        .await;

    assert!(result.is_err());
    Ok(())
}

#[tokio::test]
async fn wait_for_response_timeout_keeps_unmatched_frames_pending() -> Result<(), Box<dyn Error>> {
    let stderr = ProcessOutputBuffer::default();
    let (mut session, tx) = test_session(stderr).await?;
    tx.send(serde_json::json!({"type": "message_update", "id": "event-1"}))
        .await?;

    let result = session
        .wait_for_response_with_timeout("missing", "get_state", Duration::from_millis(1))
        .await;
    let error = result.err().ok_or("expected command timeout")?.to_string();

    assert!(error.contains("omp rpc command get_state response missing timed out"));
    assert_eq!(session.pending.len(), 1);
    assert_eq!(
        session
            .pending
            .front()
            .and_then(|frame| frame.get("type"))
            .and_then(Value::as_str),
        Some("message_update")
    );
    Ok(())
}

#[tokio::test]
async fn agent_end_reads_documented_session_stats_tokens() -> Result<(), Box<dyn Error>> {
    let stderr = ProcessOutputBuffer::default();
    let (mut session, _tx) = test_session(stderr).await?;

    session.map_frame(serde_json::json!({
        "type": "agent_end",
        "data": {
            "sessionFile": "C:\\Users\\Galtozzy\\.omp\\agent\\sessions\\session.jsonl",
            "sessionId": "019f2343-4e74-7000-abab-2b162f80b3dd",
            "userMessages": 0,
            "assistantMessages": 0,
            "toolCalls": 0,
            "toolResults": 0,
            "totalMessages": 0,
            "tokens": {
                "input": 11,
                "output": 7,
                "reasoning": 0,
                "cacheRead": 5,
                "cacheWrite": 3,
                "total": 26
            },
            "cost": 0,
            "premiumRequests": 0
        }
    }));

    assert_eq!(session.tokens.input, 11);
    assert_eq!(session.tokens.output, 7);
    assert_eq!(session.tokens.cache_read, 5);
    assert_eq!(session.tokens.cache_write, 3);
    assert_eq!(session.tokens.total, 26);
    Ok(())
}

#[tokio::test]
async fn configured_model_sets_export_model_used() -> Result<(), Box<dyn Error>> {
    let stderr = ProcessOutputBuffer::default();
    let (mut session, _tx) = test_session(stderr).await?;
    let mut env = docker_env();
    env.env_vars
        .insert(VULCANUM_OMP_MODEL_ENV.to_owned(), "gpt-5-codex".to_owned());
    env.env_vars.insert(
        VULCANUM_OMP_PROVIDER_ENV.to_owned(),
        "openai-codex".to_owned(),
    );

    session.set_configured_model(&env);
    let export = session.export().await?;

    assert_eq!(
        export.model_used,
        Some("openai-codex/gpt-5-codex".to_owned())
    );
    Ok(())
}

#[tokio::test]
async fn configured_openai_model_sets_provider_prefixed_export_model_used(
) -> Result<(), Box<dyn Error>> {
    let stderr = ProcessOutputBuffer::default();
    let (mut session, _tx) = test_session(stderr).await?;
    let mut env = docker_env();
    env.env_vars
        .insert(VULCANUM_OMP_MODEL_ENV.to_owned(), "gpt-5.5".to_owned());
    env.env_vars
        .insert(VULCANUM_OMP_PROVIDER_ENV.to_owned(), "openai".to_owned());

    session.set_configured_model(&env);
    let export = session.export().await?;

    assert_eq!(export.model_used, Some("openai/gpt-5.5".to_owned()));
    Ok(())
}

#[tokio::test]
async fn reported_model_sets_provider_prefixed_model_used() -> Result<(), Box<dyn Error>> {
    let stderr = ProcessOutputBuffer::default();
    let (mut session, tx) = test_session(stderr).await?;
    let mut env = docker_env();
    env.env_vars.insert(
        VULCANUM_OMP_PROVIDER_ENV.to_owned(),
        "openai-codex".to_owned(),
    );
    env.env_vars.insert(
        VULCANUM_OMP_MODEL_ENV.to_owned(),
        "configured-fallback".to_owned(),
    );

    tx.send(serde_json::json!({
        "id": "state-1",
        "type": "response",
        "command": "get_state",
        "success": true,
        "data": {
            "sessionId": "abc",
            "model": "gpt-5-codex"
        }
    }))
    .await?;

    session.refresh_state(&env).await?;
    let export = session.export().await?;

    assert_eq!(
        export.model_used,
        Some("openai-codex/gpt-5-codex".to_owned())
    );
    Ok(())
}

#[test]
fn host_session_path_maps_container_session_file_to_host_path() {
    let env = docker_env();
    let expected_path = env
        .workdir
        .join("home")
        .join(".omp")
        .join("sessions")
        .join("session.jsonl")
        .to_string_lossy()
        .into_owned();

    assert_eq!(
        host_session_path(&env, "/workdir/home/.omp/sessions/session.jsonl"),
        expected_path
    );
}

#[tokio::test]
async fn export_messages_ignores_missing_session_file() -> Result<(), Box<dyn Error>> {
    let stderr = ProcessOutputBuffer::default();
    let (mut session, _tx) = test_session(stderr).await?;
    session.session_path = std::env::temp_dir()
        .join("vulcanum-missing-session.jsonl")
        .to_string_lossy()
        .into_owned();

    let export = session.export_messages().await?;

    assert_eq!(export, None);
    Ok(())
}

#[tokio::test]
async fn wait_ready_reports_stderr_when_rpc_exits_before_ready() -> Result<(), Box<dyn Error>> {
    let stderr = ProcessOutputBuffer::default();
    stderr.push_line("startup config missing".to_owned());
    let (mut session, tx) = test_session(stderr).await?;
    drop(tx);

    let error = session
        .wait_ready()
        .await
        .err()
        .ok_or("expected startup error")?;
    let message = error.to_string();

    assert!(message.contains("omp rpc exited before ready"));
    assert!(message.contains("stderr: startup config missing"));
    Ok(())
}

#[tokio::test]
async fn wait_ready_redacts_sensitive_stderr_when_rpc_exits() -> Result<(), Box<dyn Error>> {
    let stderr = ProcessOutputBuffer::default();
    stderr.push_line("auth token missing".to_owned());
    let (mut session, tx) = test_session(stderr).await?;
    drop(tx);

    let error = session
        .wait_ready()
        .await
        .err()
        .ok_or("expected startup error")?;
    let message = error.to_string();

    assert!(message.contains("stderr: [redacted provider output]"));
    assert!(!message.contains("auth token missing"));
    Ok(())
}

async fn test_session(
    stderr: ProcessOutputBuffer,
) -> Result<(OmpRpcRunningSession, mpsc::Sender<Value>), Box<dyn Error>> {
    let mut child = Command::new("rustc")
        .arg("--version")
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;
    let stdin = child.stdin.take().ok_or("rustc stdin was not piped")?;
    let (tx, rx) = mpsc::channel(8);
    let session = OmpRpcRunningSession::new(child, stdin, rx, stderr, 60);
    Ok((session, tx))
}

fn docker_env() -> IsolatedEnvironment {
    let mut env_vars = HashMap::new();
    env_vars.insert(
        "PI_SESSION_DIR".to_owned(),
        "/workdir/home/.omp/sessions".to_owned(),
    );

    IsolatedEnvironment {
        workdir: PathBuf::from("/tmp/vulcanum-work-test"),
        workspace_dir: PathBuf::from("/tmp/vulcanum-work-test/workspace"),
        repos: Vec::new(),
        container_name: Some("vulcanum-test".to_owned()),
        secrets: HashMap::new(),
        env_vars,
        runtime: None,
        image: Some("ghcr.io/ezygang/vulcanum/agent:latest".to_owned()),
        server_host_port: None,
        limits: ResourceLimits::default(),
    }
}
