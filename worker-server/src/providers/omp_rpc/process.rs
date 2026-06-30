use std::path::Path;
use std::process::Stdio;

use serde_json::Value;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::mpsc;
use vulcanum_shared::runtime::errors::HarnessError;
use vulcanum_shared::runtime::types::IsolatedEnvironment;

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

    command
        .spawn()
        .map_err(|e| HarnessError::ServerLaunch(format!("failed to launch omp rpc: {e}")))
}

fn host_omp_command(env: &IsolatedEnvironment, resume_path: Option<&Path>) -> Command {
    let mut command = Command::new("omp");
    command.arg("--mode").arg("rpc");
    command.arg("--auto-approve");
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

fn docker_omp_command(
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
    command.args([
        "omp",
        "--mode",
        "rpc",
        "--auto-approve",
        "--session-dir",
        "/workdir/home/.omp/sessions",
    ]);
    if let Some(path) = resume_path {
        command.arg("--resume").arg(container_resume_path(path));
    }
    Ok(command)
}

fn container_resume_path(path: &Path) -> String {
    let path_text = path.to_string_lossy();
    match path_text.split_once("/home/.omp/sessions") {
        Some((_, rest)) => format!("/workdir/home/.omp/sessions{rest}"),
        None => path_text.to_string(),
    }
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
