#[cfg(target_os = "macos")]
use crate::commands::setup::launchd;
#[cfg(target_os = "linux")]
use crate::commands::setup::systemd;

#[must_use]
pub(crate) const fn worker_service_label() -> &'static str {
    #[cfg(target_os = "macos")]
    {
        "launchd service"
    }
    #[cfg(target_os = "linux")]
    {
        "systemd service"
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    {
        "worker service"
    }
}

#[cfg(target_os = "macos")]
pub(crate) fn configure_worker_service() -> anyhow::Result<()> {
    launchd::configure_worker_service()
}

#[cfg(target_os = "linux")]
pub(crate) fn configure_worker_service() -> anyhow::Result<()> {
    systemd::configure_worker_service()
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
pub(crate) fn configure_worker_service() -> anyhow::Result<()> {
    anyhow::bail!("worker service installation is only supported on Linux and macOS");
}

#[cfg(target_os = "macos")]
#[must_use]
pub(crate) fn is_worker_service_installed() -> bool {
    launchd::is_worker_service_installed()
}

#[cfg(target_os = "linux")]
#[must_use]
pub(crate) fn is_worker_service_installed() -> bool {
    systemd::is_worker_service_installed()
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
#[must_use]
pub(crate) fn is_worker_service_installed() -> bool {
    false
}

#[cfg(target_os = "macos")]
pub(crate) fn enable_and_restart_worker_service() -> anyhow::Result<()> {
    launchd::enable_and_restart_worker_service()
}

#[cfg(target_os = "linux")]
pub(crate) fn enable_and_restart_worker_service() -> anyhow::Result<()> {
    systemd::enable_and_restart_worker_service()
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
pub(crate) fn enable_and_restart_worker_service() -> anyhow::Result<()> {
    anyhow::bail!("worker service installation is only supported on Linux and macOS");
}

pub(crate) fn remove_worker_service_best_effort() {
    #[cfg(target_os = "macos")]
    {
        launchd::remove_worker_service_best_effort();
    }
    #[cfg(target_os = "linux")]
    {
        systemd::remove_worker_service_best_effort();
    }
}

#[cfg(target_os = "linux")]
pub(crate) fn enable_and_start_docker_service() -> anyhow::Result<()> {
    systemd::run_systemctl(&["enable", "--now", "docker"])
}

#[cfg(target_os = "linux")]
pub(crate) fn restart_docker_service() -> anyhow::Result<()> {
    systemd::run_systemctl(&["restart", "docker"])
}
