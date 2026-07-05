use std::path::PathBuf;
use std::process::Command;

use anyhow::Context;
use vulcanum_shared::constants::MAX_WORKER_CAPACITY;

const MIN_WORKER_CAPACITY: i32 = 1;
const KB_PER_GB: u64 = 1024 * 1024;

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

/// Verifies that the current user has passwordless sudo access.
pub fn has_sudo_access() -> anyhow::Result<()> {
    let status = Command::new("sudo")
        .args(["-n", "true"])
        .status()
        .map_err(|e| anyhow::anyhow!("failed to check sudo access: {e}"))?;

    if !status.success() {
        anyhow::bail!(
            "passwordless sudo is required. Ensure the user can run 'sudo -n true' without a password prompt."
        );
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
    let kvm_path = std::path::PathBuf::from("/dev/kvm");
    if !kvm_path.exists() {
        return false;
    }

    match std::fs::metadata(&kvm_path) {
        #[cfg(unix)]
        Ok(meta) => {
            use std::os::unix::fs::MetadataExt;
            let mode = meta.mode() & 0o777;
            mode & 0o666 != 0
        }
        #[cfg(not(unix))]
        Ok(_) => false,
        Err(_) => false,
    }
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
