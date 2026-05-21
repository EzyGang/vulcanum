use std::process::Command;

use super::utils::which;

const AGENT_IMAGE: &str = "ghcr.io/vulcanum/agent:latest";

pub fn pull_agent_image() -> anyhow::Result<()> {
    if is_image_pulled() {
        tracing::info!("agent image '{AGENT_IMAGE}' is already pulled");
        return Ok(());
    }

    if !which("docker") {
        anyhow::bail!("docker is not installed — run `vulcanum worker setup` to install dependencies");
    }

    tracing::info!("pulling agent image '{AGENT_IMAGE}'...");

    let status = Command::new("docker")
        .args(["pull", AGENT_IMAGE])
        .status()
        .map_err(|e| anyhow::anyhow!("failed to run docker pull: {e}"))?;

    if !status.success() {
        anyhow::bail!("docker pull '{AGENT_IMAGE}' failed");
    }

    tracing::info!("agent image pulled successfully");
    Ok(())
}

fn is_image_pulled() -> bool {
    Command::new("docker")
        .args(["images", "-q", AGENT_IMAGE])
        .output()
        .map(|o| !String::from_utf8_lossy(&o.stdout).trim().is_empty())
        .unwrap_or(false)
}
