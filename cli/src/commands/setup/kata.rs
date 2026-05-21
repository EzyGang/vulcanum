use super::utils::{run_shell, which};

pub fn install_kata() -> anyhow::Result<()> {
    if which("kata-runtime") {
        tracing::info!("kata-runtime is already installed");
        return Ok(());
    }

    tracing::info!("installing Kata Containers...");

    run_shell("curl -fsSL https://raw.githubusercontent.com/kata-containers/kata-containers/main/utils/kata-manager/kata-manager.sh | bash -s install-packages")?;

    tracing::info!("Kata Containers installed successfully");
    Ok(())
}
