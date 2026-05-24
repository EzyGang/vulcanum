use super::utils::{run_shell, run_systemctl, which};

pub fn install_docker() -> anyhow::Result<()> {
    if which("docker") {
        tracing::debug!("docker already installed");
        return Ok(());
    }

    run_shell("curl -fsSL https://get.docker.com | sh")?;
    run_systemctl("enable --now docker")?;
    Ok(())
}
