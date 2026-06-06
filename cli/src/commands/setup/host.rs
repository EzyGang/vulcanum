use std::process::Command;

use anyhow::Context;

pub fn which(binary: &str) -> bool {
    Command::new("which")
        .arg(binary)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Verifies that the current user has passwordless sudo access.
pub fn has_sudo_access() -> anyhow::Result<()> {
    let status = Command::new("sudo")
        .args(["-n", "true"])
        .status()
        .map_err(|e| anyhow::anyhow!("failed to check sudo access: {e}"))?;

    if !status.success() {
        anyhow::bail!(
            "passwordless sudo is required. Ensure the user can run 'sudo -n true' without a password prompt."
        );
    }

    Ok(())
}

pub fn worker_server_path() -> anyhow::Result<String> {
    let exe = std::env::current_exe().context("failed to get current exe")?;
    let dir = exe
        .parent()
        .ok_or_else(|| anyhow::anyhow!("failed to get exe directory"))?;
    let name = if cfg!(windows) {
        "vulcanum-server.exe"
    } else {
        "vulcanum-server"
    };
    let path = dir.join(name);
    if !path.exists() {
        anyhow::bail!("worker-server binary not found at {}", path.display());
    }
    path.to_str()
        .map(|s: &str| s.to_owned())
        .ok_or_else(|| anyhow::anyhow!("worker-server path is not valid UTF-8"))
}

/// Checks whether KVM is available and accessible on this machine.
pub fn is_kvm_available() -> bool {
    let kvm_path = std::path::PathBuf::from("/dev/kvm");
    if !kvm_path.exists() {
        return false;
    }

    match std::fs::metadata(&kvm_path) {
        #[cfg(unix)]
        Ok(meta) => {
            use std::os::unix::fs::MetadataExt;
            let mode = meta.mode() & 0o777;
            mode & 0o666 != 0
        }
        #[cfg(not(unix))]
        Ok(_) => false,
        Err(_) => false,
    }
}
