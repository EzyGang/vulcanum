use std::process::{Command, Stdio};

use crate::commands::setup::host::which;
use crate::commands::setup::systemd::run_systemctl;

pub fn install_docker() -> anyhow::Result<()> {
    if which("docker") {
        tracing::debug!("docker already installed");
        return Ok(());
    }

    let status = Command::new("sh")
        .args(["-c", "curl -fsSL https://get.docker.com | sh"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_err(|e| anyhow::anyhow!("failed to run docker install script: {e}"))?;

    if !status.success() {
        anyhow::bail!("docker install script failed");
    }

    run_systemctl("enable --now docker")?;
    Ok(())
}
