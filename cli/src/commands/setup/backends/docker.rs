#[cfg(target_os = "macos")]
mod macos;
#[cfg(test)]
mod macos_tests;

use std::path::PathBuf;
use std::process::{Command, Stdio};
#[cfg(target_os = "linux")]
use std::thread;
#[cfg(target_os = "linux")]
use std::time::Duration;

use crate::commands::setup::host::which_path;
#[cfg(target_os = "linux")]
use crate::commands::setup::service;

#[cfg(any(test, target_os = "macos"))]
const DOCKER_APP: &str = "/Applications/Docker.app";

#[cfg(target_os = "linux")]
const DOCKER_READY_ATTEMPTS: u8 = 15;
#[cfg(target_os = "linux")]
const DOCKER_READY_DELAY: Duration = Duration::from_secs(1);

#[derive(Debug, Clone, Copy)]
pub(crate) enum DockerAccess {
    Direct,
    Sudo,
}

#[cfg(any(test, target_os = "macos"))]
fn docker_desktop_launch_command(user_name: &str) -> Command {
    let mut command = Command::new("sudo");
    command.args(["-u", user_name, "open"]).arg(DOCKER_APP);
    command
}

#[cfg(target_os = "macos")]
pub fn install_docker() -> anyhow::Result<()> {
    macos::install_docker()
}

#[cfg(target_os = "linux")]
pub fn install_docker() -> anyhow::Result<()> {
    install_linux_docker()
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
pub fn install_docker() -> anyhow::Result<()> {
    anyhow::bail!("Docker auto-installation is only supported on Linux and macOS");
}

#[must_use]
#[cfg(target_os = "macos")]
pub(crate) fn docker_binary_path() -> Option<PathBuf> {
    which_path("docker").or_else(macos::docker_cli_path)
}

#[cfg(not(target_os = "macos"))]
pub(crate) fn docker_binary_path() -> Option<PathBuf> {
    which_path("docker")
}

pub(crate) fn docker_command() -> Command {
    Command::new(docker_binary_path().unwrap_or_else(|| PathBuf::from("docker")))
}

pub(crate) fn docker_info_status(access: DockerAccess) -> anyhow::Result<bool> {
    let status = match access {
        DockerAccess::Direct => docker_command()
            .arg("info")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map_err(|e| anyhow::anyhow!("failed to check docker daemon readiness: {e}"))?,
        DockerAccess::Sudo => {
            let docker = docker_binary_path().unwrap_or_else(|| PathBuf::from("docker"));
            Command::new("sudo")
                .arg("-n")
                .arg(docker)
                .arg("info")
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .map_err(|e| {
                    anyhow::anyhow!("failed to check docker daemon readiness with sudo: {e}")
                })?
        }
    };

    Ok(status.success())
}

#[cfg(target_os = "linux")]
pub(crate) fn wait_for_docker_daemon() -> anyhow::Result<()> {
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

#[cfg(target_os = "linux")]
fn install_linux_docker() -> anyhow::Result<()> {
    if docker_binary_path().is_some() {
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

    service::enable_and_start_docker_service()?;
    wait_for_docker_daemon()?;

    Ok(())
}
