use super::utils::run_systemctl;

const UNIT_NAME: &str = "vulcanum-worker";
const UNIT_PATH: &str = "/etc/systemd/system/vulcanum-worker.service";

pub fn configure_systemd(harness: &str) -> anyhow::Result<()> {
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
         Environment=VULCANUM_HARNESS={harness}\n\
         \n\
         [Install]\n\
         WantedBy=multi-user.target\n"
    );

    std::fs::write(UNIT_PATH, unit_content)?;

    run_systemctl("daemon-reload")?;

    Ok(())
}

pub fn is_unit_installed() -> bool {
    std::process::Command::new("systemctl")
        .args(["list-unit-files", UNIT_NAME])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

pub fn enable_and_restart_service() -> anyhow::Result<()> {
    run_systemctl(&format!("enable {UNIT_NAME}"))?;
    run_systemctl(&format!("restart {UNIT_NAME}"))?;
    Ok(())
}

fn worker_server_path() -> anyhow::Result<String> {
    let exe = std::env::current_exe()
        .map_err(|e| anyhow::anyhow!("cannot determine current binary path: {e}"))?;
    let dir = exe
        .parent()
        .ok_or_else(|| anyhow::anyhow!("cannot determine binary directory"))?;
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
        .map(|s| s.to_owned())
        .ok_or_else(|| anyhow::anyhow!("worker-server path is not valid UTF-8"))
}
