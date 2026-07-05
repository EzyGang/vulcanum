use std::process::Command;

use serde_json::Value;

use crate::commands::setup::backends::docker::docker_command;

pub fn docker_runtime_registered(runtime: &str) -> bool {
    docker_command()
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
            .args(["-n", "cat", "/etc/docker/daemon.json"])
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
        .args(["-n", "sh", "-c", &mv_script])
        .status()
        .map_err(|e| anyhow::anyhow!("failed to move daemon.json into place: {e}"))?;

    if !status.success() {
        anyhow::bail!("failed to install /etc/docker/daemon.json");
    }
    Ok(())
}
