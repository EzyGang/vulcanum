#[cfg(any(target_os = "linux", target_os = "macos"))]
use std::time::Duration;

#[cfg(any(target_os = "linux", target_os = "macos"))]
use anyhow::Context;
#[cfg(any(target_os = "linux", target_os = "macos"))]
use tokio::process::Command;

#[cfg(any(target_os = "linux", target_os = "macos"))]
const SERVICE_RESTART_TIMEOUT: Duration = Duration::from_secs(30);

pub(crate) trait ServiceRestarter: Send + Sync {
    async fn restart(&self) -> anyhow::Result<()>;
}

#[derive(Debug, Default)]
pub(crate) struct PlatformServiceRestarter;

impl ServiceRestarter for PlatformServiceRestarter {
    async fn restart(&self) -> anyhow::Result<()> {
        #[cfg(not(any(target_os = "linux", target_os = "macos")))]
        anyhow::bail!("worker service restart is only supported on Linux and macOS");

        #[cfg(any(target_os = "linux", target_os = "macos"))]
        {
            #[cfg(target_os = "linux")]
            let arguments = [
                "-n",
                "systemctl",
                "--no-block",
                "restart",
                "vulcanum-worker",
            ];
            #[cfg(target_os = "macos")]
            let arguments = [
                "-n",
                "launchctl",
                "kickstart",
                "-k",
                "system/com.vulcanum.worker",
            ];

            let mut command = Command::new("sudo");
            command.args(arguments).kill_on_drop(true);
            let status = tokio::time::timeout(SERVICE_RESTART_TIMEOUT, command.status())
                .await
                .context("worker service restart request timed out")?
                .context("failed to request worker service restart")?;
            if !status.success() {
                anyhow::bail!("worker service restart command failed with {status}");
            }
            Ok(())
        }
    }
}
