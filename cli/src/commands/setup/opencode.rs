use super::utils::which;

pub fn verify_or_install_opencode() -> anyhow::Result<()> {
    if which("opencode") {
        tracing::info!("OpenCode is already installed");
        return Ok(());
    }

    tracing::info!("installing OpenCode...");

    let status = std::process::Command::new("sh")
        .args(["-c", "curl -fsSL https://opencode.ai/install.sh | sh"])
        .status()
        .map_err(|e| anyhow::anyhow!("failed to run OpenCode install script: {e}"))?;

    if !status.success() {
        anyhow::bail!("OpenCode install script failed");
    }

    if !which("opencode") {
        anyhow::bail!("OpenCode installation succeeded but the binary is not in PATH — ensure ~/.local/bin is on PATH and re-run setup");
    }

    tracing::info!("OpenCode installed successfully");
    Ok(())
}
