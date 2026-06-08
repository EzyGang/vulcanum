use std::process::{Command, Stdio};

use vulcanum_shared::constants::DEFAULT_IMAGE;

use crate::commands::setup::host::which;

pub fn pull_agent_image() -> anyhow::Result<()> {
    if !which("docker") {
        anyhow::bail!(
            "docker is not installed — run `vulcanum worker setup` to install dependencies"
        );
    }

    for attempt in 1..=2 {
        let status = Command::new("docker")
            .args(["pull", DEFAULT_IMAGE])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map_err(|e| anyhow::anyhow!("failed to run docker pull: {e}"))?;

        if status.success() {
            return Ok(());
        }

        if attempt < 2 {
            tracing::debug!("docker pull failed on attempt {attempt}, retrying silently");
        }
    }

    anyhow::bail!("docker pull '{DEFAULT_IMAGE}' failed");
}
