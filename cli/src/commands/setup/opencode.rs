use crate::harness::validate::validate_environment;
use crate::harness::validate::Severity;

pub fn verify_or_install_opencode() -> anyhow::Result<()> {
    let issues = validate_environment();
    let opencode_ok = issues
        .iter()
        .filter(|i| i.severity == Severity::Critical)
        .all(|i| !i.message.contains("opencode"));

    if opencode_ok {
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

    let issues = validate_environment();
    let still_missing = issues
        .iter()
        .any(|i| i.severity == Severity::Critical && i.message.contains("opencode"));

    if still_missing {
        anyhow::bail!("OpenCode installation succeeded but the binary is not in PATH — ensure ~/.local/bin is on PATH and re-run setup");
    }

    tracing::info!("OpenCode installed successfully");
    Ok(())
}
