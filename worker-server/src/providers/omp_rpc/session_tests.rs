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
    stderr.push_line("auth token missing".to_owned());
    let (mut session, tx) = test_session(stderr).await?;
    drop(tx);

    let error = session
        .wait_ready()
        .await
        .err()
        .ok_or("expected startup error")?;
    let message = error.to_string();

    assert!(message.contains("omp rpc exited before ready"));
    assert!(message.contains("stderr: auth token missing"));
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
