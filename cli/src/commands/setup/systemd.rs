use std::process::Command;

use crate::commands::setup::host::worker_server_path;

const UNIT_NAME: &str = "vulcanum-worker";
const UNIT_PATH: &str = "/etc/systemd/system/vulcanum-worker.service";

pub(crate) fn run_systemctl(args: &str) -> anyhow::Result<()> {
    let status = Command::new("sudo")
        .arg("-n")
        .arg("systemctl")
        .args(args.split_whitespace())
        .status()
        .map_err(|e| anyhow::anyhow!("failed to run systemctl: {e}"))?;

    if !status.success() {
        anyhow::bail!("systemctl {} failed", args);
    }
    Ok(())
}

pub(crate) fn configure_worker_service() -> anyhow::Result<()> {
    let binary_path = worker_server_path()?;
    tracing::debug!("binding systemd to binary at: {binary_path}");

    let unit_content = format!(
        "[Unit]\n\
         Description=Vulcanum Worker Daemon\n\
         After=network-online.target\n\
         Wants=network-online.target\n\
         \n\
         [Service]\n\
         Type=simple\n\
         ExecStart={binary_path}\n\
         Restart=on-failure\n\
         RestartSec=10\n\
         Environment=PATH=/usr/local/bin:/usr/bin:/bin:/usr/local/sbin:/usr/sbin:/sbin\n\
         \n\
         [Install]\n\
         WantedBy=multi-user.target\n"
    );

    let tmp_path = std::env::temp_dir().join("vulcanum-worker.service");
    std::fs::write(&tmp_path, unit_content)
        .map_err(|e| anyhow::anyhow!("failed to write temporary systemd unit: {e}"))?;

    let status = Command::new("sudo")
        .args(["-n", "install", "-m", "0644"])
        .arg(&tmp_path)
        .arg(UNIT_PATH)
        .status()
        .map_err(|e| anyhow::anyhow!("failed to install systemd unit: {e}"))?;
    if !status.success() {
        anyhow::bail!("failed to install systemd unit");
    }

    run_systemctl("daemon-reload")?;

    Ok(())
}

#[must_use]
pub(crate) fn is_worker_service_installed() -> bool {
    Command::new("systemctl")
        .args(["list-unit-files", UNIT_NAME])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

pub(crate) fn enable_and_restart_worker_service() -> anyhow::Result<()> {
    run_systemctl(&format!("enable {UNIT_NAME}"))?;
    run_systemctl(&format!("restart {UNIT_NAME}"))?;
    Ok(())
}

pub(crate) fn remove_worker_service_best_effort() {
    if is_worker_service_installed() {
        if let Err(err) = run_systemctl(&format!("stop {UNIT_NAME}")) {
            tracing::warn!(error = %err, "failed to stop worker service before uninstall");
        }

        if let Err(err) = run_systemctl(&format!("disable {UNIT_NAME}")) {
            tracing::warn!(error = %err, "failed to disable worker service before uninstall");
        }
    }

    let status = Command::new("sudo")
        .args(["-n", "rm", "-f", UNIT_PATH])
        .status();
    match status {
        Ok(status) if status.success() => (),
        Ok(_) => tracing::warn!(
            path = UNIT_PATH,
            "failed to remove worker service unit file"
        ),
        Err(err) => {
            tracing::warn!(error = %err, path = UNIT_PATH, "failed to remove worker service unit file")
        }
    }

    if let Err(err) = run_systemctl("daemon-reload") {
        tracing::warn!(error = %err, "failed to reload systemd after uninstall");
    }
}
