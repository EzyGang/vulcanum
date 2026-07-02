use std::collections::VecDeque;
use std::path::Path;
use std::process::Stdio;
use std::sync::{Arc, Mutex};

use serde_json::Value;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, ChildStderr, Command};
use tokio::sync::mpsc;
use vulcanum_shared::runtime::errors::HarnessError;
use vulcanum_shared::runtime::types::IsolatedEnvironment;

use crate::isolation::workspace;

const STDERR_LINE_LIMIT: usize = 40;
const OMP_APPROVAL_FLAG: &str = "--yolo";

#[derive(Clone, Default)]
pub(crate) struct ProcessOutputBuffer {
    lines: Arc<Mutex<VecDeque<String>>>,
}

impl ProcessOutputBuffer {
    pub(crate) fn push_line(&self, line: String) {
        let mut lines = match self.lines.lock() {
            Ok(lines) => lines,
            Err(poisoned) => poisoned.into_inner(),
        };
        if lines.len() == STDERR_LINE_LIMIT {
            lines.pop_front();
        }
        lines.push_back(line);
    }

    pub(crate) fn tail(&self) -> String {
        let lines = match self.lines.lock() {
            Ok(lines) => lines,
            Err(poisoned) => poisoned.into_inner(),
        };
        lines.iter().cloned().collect::<Vec<String>>().join("\n")
    }
}

pub(crate) async fn launch_omp(
    env: &IsolatedEnvironment,
    resume_path: Option<&Path>,
) -> Result<Child, HarnessError> {
    let mut command = match env.container_name.as_deref() {
        Some(container_name) => docker_omp_command(env, container_name, resume_path)?,
        None => host_omp_command(env, resume_path),
    };

    command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    command.kill_on_drop(true);

    command
        .spawn()
        .map_err(|e| HarnessError::ServerLaunch(format!("failed to launch omp rpc: {e}")))
}

pub(super) fn host_omp_command(env: &IsolatedEnvironment, resume_path: Option<&Path>) -> Command {
    let mut command = Command::new("omp");
    command.arg("--mode").arg("rpc");
    command.arg(OMP_APPROVAL_FLAG);
    if let Some(session_dir) = env.env_vars.get("PI_SESSION_DIR") {
        command.arg("--session-dir").arg(session_dir);
    }
    if let Some(path) = resume_path {
        command.arg("--resume").arg(path);
    }
    command.current_dir(&env.workspace_dir);
    command.envs(&env.env_vars);
    command
}

pub(super) fn docker_omp_command(
    env: &IsolatedEnvironment,
    container_name: &str,
    resume_path: Option<&Path>,
) -> Result<Command, HarnessError> {
    let image = env
        .image
        .as_ref()
        .ok_or_else(|| HarnessError::ServerLaunch("missing container image".to_owned()))?;
    let mut command = Command::new("docker");
    command.args(["run", "--rm", "-i", "--name", container_name]);
    if let Some(runtime) = env.runtime {
        command.args(["--runtime", runtime]);
    }
    command
        .arg("-v")
        .arg(format!("{}:/workdir", env.workdir.display()));
    command.args(["--workdir", "/workdir/workspace"]);
    for (key, value) in &env.env_vars {
        command.arg("-e").arg(format!("{key}={value}"));
    }
    command.arg(image);
    let session_dir = env
        .env_vars
        .get("PI_SESSION_DIR")
        .map(String::as_str)
        .unwrap_or("/workdir/home/.omp/sessions");
    command.args(["omp", "--mode", "rpc", OMP_APPROVAL_FLAG, "--session-dir"]);
    command.arg(session_dir);
    if let Some(path) = resume_path {
        command
            .arg("--resume")
            .arg(workspace::container_path(&env.workdir, "/workdir", path));
    }
    Ok(command)
}

pub(crate) async fn read_stdout_frames(
    stdout: tokio::process::ChildStdout,
    tx: mpsc::Sender<Value>,
) {
    let mut lines = BufReader::new(stdout).lines();
    while let Ok(Some(line)) = lines.next_line().await {
        match serde_json::from_str::<Value>(&line) {
            Ok(frame) => {
                if tx.send(frame).await.is_err() {
                    return;
                }
            }
            Err(error) => {
                tracing::warn!(line = line, error = %error, "failed to parse OMP RPC frame");
            }
        }
    }
}

pub(crate) async fn read_stderr_tail(stderr: ChildStderr, buffer: ProcessOutputBuffer) {
    let mut lines = BufReader::new(stderr).lines();
    while let Ok(Some(line)) = lines.next_line().await {
        buffer.push_line(line);
    }
}
