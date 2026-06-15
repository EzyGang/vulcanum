use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

use crate::commands::setup::host::which;
use crate::commands::setup::systemd::run_systemctl;

const DOCKER_READY_ATTEMPTS: u8 = 15;
const DOCKER_READY_DELAY: Duration = Duration::from_secs(1);

#[derive(Debug, Clone, Copy)]
enum DockerAccess {
    Direct,
    Sudo,
}

pub fn install_docker() -> anyhow::Result<()> {
    if which("docker") {
        tracing::debug!("docker already installed");
    } else {
        let status = Command::new("sh")
            .args(["-c", "curl -fsSL https://get.docker.com | sudo -n sh"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map_err(|e| anyhow::anyhow!("failed to run docker install script: {e}"))?;

        if !status.success() {
            anyhow::bail!("docker install script failed");
        }
    }

    run_systemctl("enable --now docker")?;
    wait_for_docker_daemon()?;

    Ok(())
}

fn wait_for_docker_daemon() -> anyhow::Result<()> {
    for attempt in 1..=DOCKER_READY_ATTEMPTS {
        if docker_info_status(DockerAccess::Direct)? || docker_info_status(DockerAccess::Sudo)? {
            return Ok(());
        }

        if attempt < DOCKER_READY_ATTEMPTS {
            thread::sleep(DOCKER_READY_DELAY);
        }
    }

    anyhow::bail!("docker daemon did not become ready after starting the docker service")
}

fn docker_info_status(access: DockerAccess) -> anyhow::Result<bool> {
    let status = match access {
        DockerAccess::Direct => Command::new("docker")
            .arg("info")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map_err(|e| anyhow::anyhow!("failed to check docker daemon readiness: {e}"))?,
        DockerAccess::Sudo => Command::new("sudo")
            .args(["-n", "docker", "info"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map_err(|e| {
                anyhow::anyhow!("failed to check docker daemon readiness with sudo: {e}")
            })?,
    };

    Ok(status.success())
}
