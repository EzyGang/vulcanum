use std::process::Command;

pub(crate) trait ServiceRestarter: Send + Sync {
    fn restart(&self) -> anyhow::Result<()>;
}

#[derive(Debug, Default)]
pub(crate) struct PlatformServiceRestarter;

impl ServiceRestarter for PlatformServiceRestarter {
    fn restart(&self) -> anyhow::Result<()> {
        #[cfg(target_os = "linux")]
        let status = Command::new("sudo")
            .args([
                "-n",
                "systemctl",
                "--no-block",
                "restart",
                "vulcanum-worker",
            ])
            .status()
            .map_err(|error| anyhow::anyhow!("failed to request systemd restart: {error}"))?;

        #[cfg(target_os = "macos")]
        let status = Command::new("sudo")
            .args([
                "-n",
                "launchctl",
                "kickstart",
                "-k",
                "system/com.vulcanum.worker",
            ])
            .status()
            .map_err(|error| anyhow::anyhow!("failed to request launchd restart: {error}"))?;

        #[cfg(not(any(target_os = "linux", target_os = "macos")))]
        anyhow::bail!("worker service restart is only supported on Linux and macOS");

        #[cfg(any(target_os = "linux", target_os = "macos"))]
        if !status.success() {
            anyhow::bail!("worker service restart command failed with {status}");
        }

        Ok(())
    }
}
