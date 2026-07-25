use std::fs::File;
use std::path::{Path, PathBuf};

use anyhow::Context;
use uuid::Uuid;

use crate::update::activation::{CLI_BINARY, WORKER_BINARY};
use crate::update::VERSION_FILE;

pub(super) fn ensure_pair_exists(cli_path: &Path, worker_path: &Path) -> anyhow::Result<()> {
    if !cli_path.is_file() || !worker_path.is_file() {
        anyhow::bail!(
            "installed vulcanum and vulcanum-server binaries must be in the same directory"
        );
    }
    Ok(())
}

pub(super) fn backup(source: &Path, destination: &Path) -> anyhow::Result<()> {
    std::fs::copy(source, destination).with_context(|| {
        format!(
            "failed to back up {} to {}",
            source.display(),
            destination.display()
        )
    })?;
    sync_file(destination)
}

pub(super) fn restore_pair(rollback_dir: &Path, install_dir: &Path) -> anyhow::Result<()> {
    let mut errors: Vec<String> = Vec::new();
    for name in [CLI_BINARY, WORKER_BINARY] {
        if let Err(error) = restore(&rollback_dir.join(name), &install_dir.join(name)) {
            errors.push(error.to_string());
        }
    }

    let backup_version = rollback_dir.join(VERSION_FILE);
    let installed_version = install_dir.join(VERSION_FILE);
    let version_result = match backup_version.is_file() {
        true => restore(&backup_version, &installed_version),
        false => std::fs::remove_file(&installed_version)
            .or_else(|error| match error.kind() {
                std::io::ErrorKind::NotFound => Ok(()),
                _ => Err(error),
            })
            .map_err(Into::into),
    };
    if let Err(error) = version_result {
        errors.push(error.to_string());
    }

    if !errors.is_empty() {
        anyhow::bail!(errors.join("; "));
    }
    sync_dir(install_dir)
}

fn restore(source: &Path, destination: &Path) -> anyhow::Result<()> {
    let temporary: PathBuf = destination.with_extension(format!("restore-{}", Uuid::new_v4()));
    std::fs::copy(source, &temporary).with_context(|| {
        format!(
            "failed to copy rollback file {} to {}",
            source.display(),
            temporary.display()
        )
    })?;
    sync_file(&temporary)?;
    std::fs::rename(&temporary, destination).with_context(|| {
        format!(
            "failed to restore {} from {}",
            destination.display(),
            source.display()
        )
    })?;
    Ok(())
}

pub(super) fn sync_file(path: &Path) -> anyhow::Result<()> {
    File::open(path)
        .and_then(|file| file.sync_all())
        .with_context(|| format!("failed to sync {}", path.display()))
}

#[cfg(unix)]
pub(super) fn sync_dir(path: &Path) -> anyhow::Result<()> {
    File::open(path)
        .and_then(|directory| directory.sync_all())
        .with_context(|| format!("failed to sync directory {}", path.display()))
}

#[cfg(not(unix))]
pub(super) fn sync_dir(_path: &Path) -> anyhow::Result<()> {
    Ok(())
}
