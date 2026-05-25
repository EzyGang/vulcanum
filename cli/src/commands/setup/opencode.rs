use std::process::{Command, Stdio};

use super::utils::which;

pub fn verify_or_install_opencode() -> anyhow::Result<()> {
    if which("opencode") {
        tracing::debug!("opencode already installed");
        return Ok(());
    }

    let status = Command::new("sh")
        .args(["-c", "curl -fsSL https://opencode.ai/install.sh | sh"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_err(|e| anyhow::anyhow!("failed to run OpenCode install script: {e}"))?;

    if !status.success() {
        anyhow::bail!("OpenCode install script failed");
    }

    if !which("opencode") {
        anyhow::bail!("OpenCode installation succeeded but the binary is not in PATH — ensure ~/.local/bin is on PATH and re-run setup");
    }

    Ok(())
}
