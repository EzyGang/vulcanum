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
use crate::providers::omp_rpc::{
    VULCANUM_OMP_MODEL_ENV, VULCANUM_OMP_PROVIDER_ENV, VULCANUM_OMP_SMOL_ENV,
};

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
    append_omp_model_args(&mut command, env);
    if let Some(path) = resume_path {
        command.arg("--resume").arg(path);
    }
    command.current_dir(&env.workspace_dir);
    for (key, value) in &env.env_vars {
        if !is_omp_launch_metadata(key) {
            command.env(key, value);
        }
    }
    remove_omp_launch_metadata(&mut command);
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
        if !is_omp_launch_metadata(key) {
            command.arg("-e").arg(format!("{key}={value}"));
        }
    }
    command.arg(image);
    let session_dir = env
        .env_vars
        .get("PI_SESSION_DIR")
        .map(String::as_str)
        .unwrap_or("/workdir/home/.omp/sessions");
    command.args(["omp", "--mode", "rpc", OMP_APPROVAL_FLAG, "--session-dir"]);
    command.arg(session_dir);
    append_omp_model_args(&mut command, env);
    if let Some(path) = resume_path {
        command
            .arg("--resume")
            .arg(workspace::container_path(&env.workdir, "/workdir", path));
    }
    Ok(command)
}

fn append_omp_model_args(command: &mut Command, env: &IsolatedEnvironment) {
    if let Some(provider) = omp_launch_value(env, VULCANUM_OMP_PROVIDER_ENV) {
        command.arg("--provider").arg(provider);
    }
    if let Some(model) = omp_launch_value(env, VULCANUM_OMP_MODEL_ENV) {
        command.arg("--model").arg(model);
    }
    if let Some(smol) = omp_launch_value(env, VULCANUM_OMP_SMOL_ENV) {
        command.arg("--smol").arg(smol);
    }
}

fn omp_launch_value<'a>(env: &'a IsolatedEnvironment, key: &str) -> Option<&'a str> {
    env.env_vars
        .get(key)
        .map(String::as_str)
        .filter(|value| !value.is_empty())
}

fn remove_omp_launch_metadata(command: &mut Command) {
    command.env_remove(VULCANUM_OMP_PROVIDER_ENV);
    command.env_remove(VULCANUM_OMP_MODEL_ENV);
    command.env_remove(VULCANUM_OMP_SMOL_ENV);
}

fn is_omp_launch_metadata(key: &str) -> bool {
    matches!(
        key,
        VULCANUM_OMP_PROVIDER_ENV | VULCANUM_OMP_MODEL_ENV | VULCANUM_OMP_SMOL_ENV
    )
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
