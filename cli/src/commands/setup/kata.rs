use std::process::{Command, Stdio};

use super::utils::which;

pub(super) const KATA_MANAGER_URL: &str =
    "https://raw.githubusercontent.com/kata-containers/kata-containers/main/utils/kata-manager.sh";

pub fn install_kata() -> anyhow::Result<()> {
    if which("kata-runtime") {
        tracing::debug!("kata-runtime already installed");
        return Ok(());
    }

    let status = Command::new("sh")
        .args([
            "-c",
            &format!("curl -fsSL {KATA_MANAGER_URL} | sudo bash -s -- -D"),
        ])
        .current_dir("/")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_err(|e| anyhow::anyhow!("failed to run kata-manager: {e}"))?;

    if !status.success() && !which("kata-runtime") {
        anyhow::bail!("kata-manager.sh -D failed");
    }

    Ok(())
}
