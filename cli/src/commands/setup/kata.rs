use std::process::{Command, Stdio};

use serde_json::{Map, Value};

use super::utils::{run_systemctl, which};

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

/// Configure Docker to recognise the kata-runtime.
///
/// If `/etc/docker/daemon.json` already lists `kata-runtime` under
/// `runtimes` the file is left untouched (idempotent).
pub fn configure_docker_for_kata() -> anyhow::Result<()> {
    let kata_path =
        kata_runtime_path().ok_or_else(|| anyhow::anyhow!("kata-runtime not found in PATH"))?;

    let existing = read_daemon_json();
    let mut config = existing.unwrap_or_else(|_| Value::Object(Map::new()));

    let runtimes = config
        .as_object_mut()
        .ok_or_else(|| anyhow::anyhow!("docker daemon.json root is not an object"))?
        .entry("runtimes")
        .or_insert_with(|| Value::Object(Map::new()))
        .as_object_mut()
        .ok_or_else(|| anyhow::anyhow!("docker daemon.json runtimes is not an object"))?;

    if runtimes.contains_key("kata-runtime") {
        tracing::debug!("kata-runtime already registered in docker daemon.json");
        return Ok(());
    }

    let mut runtime_entry = Map::new();
    runtime_entry.insert("path".to_owned(), Value::String(kata_path));
    runtimes.insert("kata-runtime".to_owned(), Value::Object(runtime_entry));

    let new_content = serde_json::to_string_pretty(&config)
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

    run_systemctl("restart docker")?;
    Ok(())
}

fn kata_runtime_path() -> Option<String> {
    Command::new("which")
        .arg("kata-runtime")
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_owned())
}

fn read_daemon_json() -> Result<Value, anyhow::Error> {
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
