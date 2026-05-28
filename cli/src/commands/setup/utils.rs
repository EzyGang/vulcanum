use std::process::Command;

pub fn which(binary: &str) -> bool {
    Command::new("which")
        .arg(binary)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

pub fn run_systemctl(args: &str) -> anyhow::Result<()> {
    let status = Command::new("sudo")
        .arg("systemctl")
        .args(args.split_whitespace())
        .status()
        .map_err(|e| anyhow::anyhow!("failed to run systemctl: {e}"))?;

    if !status.success() {
        anyhow::bail!("systemctl {} failed", args);
    }
    Ok(())
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
