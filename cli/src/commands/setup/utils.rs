use std::process::Command;

use serde_json::Value;

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

pub fn docker_runtime_registered(runtime: &str) -> bool {
    Command::new("docker")
        .args(["info", "--format", "{{json .Runtimes}}"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|output| output.contains(runtime))
        .unwrap_or(false)
}

pub fn read_daemon_json() -> anyhow::Result<Value> {
    let raw = std::fs::read_to_string("/etc/docker/daemon.json").or_else(|_| {
        Command::new("sudo")
            .args(["cat", "/etc/docker/daemon.json"])
            .output()
            .ok()
            .filter(|o| o.status.success())
            .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
            .ok_or_else(|| anyhow::anyhow!("daemon.json not readable"))
    })?;
    serde_json::from_str(&raw).map_err(|e| anyhow::anyhow!("malformed daemon.json: {e}"))
}

pub fn write_daemon_json(config: &Value) -> anyhow::Result<()> {
    let new_content = serde_json::to_string_pretty(config)
        .map_err(|e| anyhow::anyhow!("failed to serialize daemon.json: {e}"))?;

    let tmp_path = std::env::temp_dir().join("vulcanum-docker-daemon.json");
    std::fs::write(&tmp_path, new_content)
        .map_err(|e| anyhow::anyhow!("failed to write temp daemon.json: {e}"))?;

    let mv_script = format!(
        "mkdir -p /etc/docker && mv {} /etc/docker/daemon.json && chmod 644 /etc/docker/daemon.json",
        tmp_path.display()
    );

    let status = Command::new("sudo")
        .args(["sh", "-c", &mv_script])
        .status()
        .map_err(|e| anyhow::anyhow!("failed to move daemon.json into place: {e}"))?;

    if !status.success() {
        anyhow::bail!("failed to install /etc/docker/daemon.json");
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
