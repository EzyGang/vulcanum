use super::utils::run_systemctl;

const UNIT_NAME: &str = "vulcanum-worker";
const UNIT_PATH: &str = "/etc/systemd/system/vulcanum-worker.service";

pub fn configure_systemd() -> anyhow::Result<()> {
    let binary_path = current_exe_path()?;
    tracing::debug!("binding systemd to binary at: {binary_path}");

    let unit_content = format!(
        "[Unit]\n\
         Description=Vulcanum Worker Daemon\n\
         After=network-online.target\n\
         Wants=network-online.target\n\
         \n\
         [Service]\n\
         Type=simple\n\
         ExecStart={binary_path} worker daemon\n\
         Restart=always\n\
         RestartSec=10\n\
         Environment=VULCANUM_HARNESS=kata\n\
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

pub fn enable_and_start_service() -> anyhow::Result<()> {
    run_systemctl(&format!("enable --now {UNIT_NAME}"))?;
    Ok(())
}

fn current_exe_path() -> anyhow::Result<String> {
    std::env::current_exe()
        .map_err(|e| anyhow::anyhow!("cannot determine current binary path: {e}"))?
        .to_str()
        .map(|s| s.to_owned())
        .ok_or_else(|| anyhow::anyhow!("binary path is not valid UTF-8"))
}
