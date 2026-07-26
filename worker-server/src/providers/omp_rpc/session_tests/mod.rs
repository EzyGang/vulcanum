mod exports;
mod protocol;
mod startup;

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
