use std::process::{Command, Stdio};

use super::utils::which;
use crate::harness::kata::DEFAULT_KATA_IMAGE;

pub fn pull_agent_image() -> anyhow::Result<()> {
    if !which("docker") {
        anyhow::bail!(
            "docker is not installed — run `vulcanum worker setup` to install dependencies"
        );
    }

    let status = Command::new("docker")
        .args(["pull", DEFAULT_KATA_IMAGE])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_err(|e| anyhow::anyhow!("failed to run docker pull: {e}"))?;

    if !status.success() {
        anyhow::bail!("docker pull '{DEFAULT_KATA_IMAGE}' failed");
    }

    Ok(())
}
