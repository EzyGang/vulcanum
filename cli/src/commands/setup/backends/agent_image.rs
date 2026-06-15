use std::process::{Command, Output};
use std::thread;
use std::time::Duration;

use vulcanum_shared::constants::DEFAULT_IMAGE;

use crate::commands::setup::host::which;

const PULL_ATTEMPTS: u8 = 3;
const PULL_RETRY_DELAY: Duration = Duration::from_secs(2);

#[derive(Debug, Clone, Copy)]
enum DockerAccess {
    Direct,
    Sudo,
}

pub fn pull_agent_image() -> anyhow::Result<()> {
    if !which("docker") {
        anyhow::bail!(
            "docker is not installed — run `vulcanum worker setup` to install dependencies"
        );
    }

    let access = docker_access()?;

    for attempt in 1..=PULL_ATTEMPTS {
        let output = docker_pull_command(access)
            .output()
            .map_err(|e| anyhow::anyhow!("failed to run docker pull: {e}"))?;

        if output.status.success() {
            return Ok(());
        }

        let message = docker_failure_message(&output);

        if attempt < PULL_ATTEMPTS {
            tracing::debug!(
                attempt,
                error = %message,
                "docker pull failed; retrying"
            );
            thread::sleep(PULL_RETRY_DELAY);
            continue;
        }

        anyhow::bail!("docker pull '{DEFAULT_IMAGE}' failed: {message}");
    }

    anyhow::bail!("docker pull '{DEFAULT_IMAGE}' failed");
}

fn docker_access() -> anyhow::Result<DockerAccess> {
    if docker_info_status(DockerAccess::Direct)? {
        return Ok(DockerAccess::Direct);
    }

    if docker_info_status(DockerAccess::Sudo)? {
        return Ok(DockerAccess::Sudo);
    }

    anyhow::bail!("docker daemon is not reachable by the current user or passwordless sudo")
}

fn docker_info_status(access: DockerAccess) -> anyhow::Result<bool> {
    let status = match access {
        DockerAccess::Direct => {
            Command::new("docker")
                .arg("info")
                .output()
                .map_err(|e| anyhow::anyhow!("failed to check docker daemon readiness: {e}"))?
                .status
        }
        DockerAccess::Sudo => {
            Command::new("sudo")
                .args(["-n", "docker", "info"])
                .output()
                .map_err(|e| {
                    anyhow::anyhow!("failed to check docker daemon readiness with sudo: {e}")
                })?
                .status
        }
    };

    Ok(status.success())
}

fn docker_pull_command(access: DockerAccess) -> Command {
    let mut command = match access {
        DockerAccess::Direct => Command::new("docker"),
        DockerAccess::Sudo => {
            let mut command = Command::new("sudo");
            command.arg("-n").arg("docker");
            command
        }
    };

    command.args(["pull", "--quiet", DEFAULT_IMAGE]);
    command
}

fn docker_failure_message(output: &Output) -> String {
    let stderr_raw = String::from_utf8_lossy(&output.stderr);
    let stderr = stderr_raw.trim();
    if !stderr.is_empty() {
        return stderr.to_owned();
    }

    let stdout_raw = String::from_utf8_lossy(&output.stdout);
    let stdout = stdout_raw.trim();
    if !stdout.is_empty() {
        return stdout.to_owned();
    }

    match output.status.code() {
        Some(code) => format!("exit status {code}"),
        None => "process terminated before reporting an exit status".to_owned(),
    }
}
