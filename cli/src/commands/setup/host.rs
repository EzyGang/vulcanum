use std::path::PathBuf;
use std::process::Command;

use anyhow::Context;
use vulcanum_shared::constants::MAX_WORKER_CAPACITY;
use vulcanum_shared::validate;

const MIN_WORKER_CAPACITY: i32 = 1;
const KB_PER_GB: u64 = 1024 * 1024;

#[cfg(target_os = "linux")]
#[must_use]
pub fn which(binary: &str) -> bool {
    which_path(binary).is_some()
}

#[must_use]
pub(crate) fn which_path(binary: &str) -> Option<PathBuf> {
    std::env::var_os("PATH").and_then(|path| {
        std::env::split_paths(&path).find_map(|dir| {
            let candidate = dir.join(binary);
            match candidate.is_file() {
                true => Some(candidate),
                false => None,
            }
        })
    })
}

#[cfg(target_os = "macos")]
pub(crate) fn macos_user() -> anyhow::Result<String> {
    let user_name = std::env::var("SUDO_USER")
        .or_else(|_| std::env::var("USER"))
        .map_err(|_| anyhow::anyhow!("failed to resolve current macOS user"))?;
    if user_name == "root" {
        anyhow::bail!("failed to resolve the macOS login user from the sudo environment");
    }
    Ok(user_name)
}

/// Ensures that sudo can run non-interactively during setup.
pub fn has_sudo_access() -> anyhow::Result<()> {
    has_sudo_access_with(|action| {
        let mut command = Command::new("sudo");
        match action {
            SudoAction::Check => {
                command.args(["-n", "true"]);
            }
            SudoAction::Authorize => {
                command.arg("-v");
            }
        }

        command
            .status()
            .map(|status| status.success())
            .map_err(|e| anyhow::anyhow!("failed to authorize administrator access: {e}"))
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SudoAction {
    Check,
    Authorize,
}

pub(crate) fn has_sudo_access_with<F>(mut run: F) -> anyhow::Result<()>
where
    F: FnMut(SudoAction) -> anyhow::Result<bool>,
{
    if run(SudoAction::Check)? {
        return Ok(());
    }

    eprintln!("  Administrator access is required to configure the worker service.");
    if !run(SudoAction::Authorize)? {
        anyhow::bail!("administrator authorization was denied");
    }
    if !run(SudoAction::Check)? {
        anyhow::bail!("sudo authorization succeeded, but non-interactive sudo is unavailable");
    }

    Ok(())
}

pub fn worker_server_path() -> anyhow::Result<String> {
    let exe = std::env::current_exe().context("failed to get current exe")?;
    let dir = exe
        .parent()
        .ok_or_else(|| anyhow::anyhow!("failed to get exe directory"))?;
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
        .map(|s: &str| s.to_owned())
        .ok_or_else(|| anyhow::anyhow!("worker-server path is not valid UTF-8"))
}

pub fn calculate_worker_capacity() -> i32 {
    match probe_host_resources() {
        Some(resources) => capacity_from_resources(resources.cpu_count, resources.total_ram_kb),
        None => MIN_WORKER_CAPACITY,
    }
}

#[must_use]
pub(crate) fn capacity_from_resources(cpu_count: u32, total_ram_kb: u64) -> i32 {
    let ram_gb = total_ram_kb / KB_PER_GB;
    let cpu_jobs = (cpu_count / 2).min(i32::MAX as u32) as i32;
    let mem_jobs = (ram_gb / 4).min(i32::MAX as u64) as i32;

    cpu_jobs
        .min(mem_jobs)
        .clamp(MIN_WORKER_CAPACITY, MAX_WORKER_CAPACITY)
}

/// Checks whether KVM is available and accessible on this machine.
pub fn is_kvm_available() -> bool {
    validate::is_kvm_available()
}

#[derive(Debug, Clone, Copy)]
struct HostResources {
    cpu_count: u32,
    total_ram_kb: u64,
}

fn probe_host_resources() -> Option<HostResources> {
    let cpu_count = sys_info::cpu_num().ok()?;
    let mem_info = sys_info::mem_info().ok()?;

    Some(HostResources {
        cpu_count,
        total_ram_kb: mem_info.total,
    })
}
