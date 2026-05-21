use super::utils::{run_shell, run_systemctl, which};

pub fn install_docker() -> anyhow::Result<()> {
    if which("docker") {
        tracing::info!("Docker is already installed");
        return Ok(());
    }

    tracing::info!("installing Docker...");

    run_shell("curl -fsSL https://get.docker.com | sh")?;
    run_systemctl("enable --now docker")?;

    tracing::info!("Docker installed successfully");
    Ok(())
}
