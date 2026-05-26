use super::utils::run_systemctl;

const UNIT_NAME: &str = "vulcanum-worker";
const UNIT_PATH: &str = "/etc/systemd/system/vulcanum-worker.service";

pub fn configure_systemd() -> anyhow::Result<()> {
    if is_unit_active() {
        tracing::debug!("systemd unit '{UNIT_NAME}' already active");
        return Ok(());
    }

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

pub fn enable_and_start_service() -> anyhow::Result<()> {
    if is_unit_active() {
        tracing::debug!("systemd unit '{UNIT_NAME}' already active");
        return Ok(());
    }

    run_systemctl(&format!("enable --now {UNIT_NAME}"))?;

    Ok(())
}

fn is_unit_active() -> bool {
    std::process::Command::new("systemctl")
        .args(["is-active", "--quiet", UNIT_NAME])
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn current_exe_path() -> anyhow::Result<String> {
    std::env::current_exe()
        .map_err(|e| anyhow::anyhow!("cannot determine current binary path: {e}"))?
        .to_str()
        .map(|s| s.to_owned())
        .ok_or_else(|| anyhow::anyhow!("binary path is not valid UTF-8"))
}
