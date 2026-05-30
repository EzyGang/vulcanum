use std::process::{Command, Stdio};

use serde_json::{Map, Value};

use super::utils::{
    docker_runtime_registered, read_daemon_json, run_systemctl, which, write_daemon_json,
};

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

pub fn configure_docker_for_kata() -> anyhow::Result<()> {
    let kata_path =
        kata_runtime_path().ok_or_else(|| anyhow::anyhow!("kata-runtime not found in PATH"))?;

    if docker_runtime_registered("kata-runtime") {
        tracing::info!("Kata runtime already active in Docker — skipping configuration");
        return Ok(());
    }

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
        tracing::debug!(
            "kata-runtime in daemon.json but not picked up by Docker, restarting daemon"
        );
        run_systemctl("restart docker")?;
        return Ok(());
    }

    let mut runtime_entry = Map::new();
    runtime_entry.insert("path".to_owned(), Value::String(kata_path));
    runtimes.insert("kata-runtime".to_owned(), Value::Object(runtime_entry));

    write_daemon_json(&config)?;
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
