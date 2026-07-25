use std::fs::File;
use std::path::{Path, PathBuf};

use anyhow::Context;
use uuid::Uuid;

use crate::update::activation::{CLI_BINARY, WORKER_BINARY};
use crate::update::VERSION_FILE;

const RESTORE_TEMP_PREFIX: &str = ".vulcanum-restore-";

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

pub(super) fn backup_version(
    source: &Path,
    destination: &Path,
    fallback: &str,
) -> anyhow::Result<()> {
    match source.is_file() {
        true => backup(source, destination),
        false => {
            std::fs::write(destination, fallback).with_context(|| {
                format!(
                    "failed to write fallback version to {}",
                    destination.display()
                )
            })?;
            sync_file(destination)
        }
    }
}

pub(super) fn restore_pair(rollback_dir: &Path, install_dir: &Path) -> anyhow::Result<()> {
    remove_abandoned_restore_files(install_dir)?;
    let mut errors: Vec<String> = Vec::new();
    if let Err(error) = restore(
        &rollback_dir.join(CLI_BINARY),
        &install_dir.join(CLI_BINARY),
    ) {
        errors.push(error.to_string());
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

    if errors.is_empty() {
        if let Err(error) = sync_dir(install_dir) {
            errors.push(error.to_string());
        }
    }
    if errors.is_empty() {
        if let Err(error) = restore(
            &rollback_dir.join(WORKER_BINARY),
            &install_dir.join(WORKER_BINARY),
        ) {
            errors.push(error.to_string());
        }
    }
    if !errors.is_empty() {
        anyhow::bail!(errors.join("; "));
    }
    sync_dir(install_dir)
}

fn restore(source: &Path, destination: &Path) -> anyhow::Result<()> {
    let parent = destination.parent().with_context(|| {
        format!(
            "restore destination {} has no parent directory",
            destination.display()
        )
    })?;
    let temporary: PathBuf = parent.join(format!("{RESTORE_TEMP_PREFIX}{}", Uuid::new_v4()));
    let result: anyhow::Result<()> = (|| {
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
    })();

    match result {
        Ok(()) => Ok(()),
        Err(error) => match std::fs::remove_file(&temporary) {
            Ok(()) => Err(error),
            Err(cleanup_error) if cleanup_error.kind() == std::io::ErrorKind::NotFound => {
                Err(error)
            }
            Err(cleanup_error) => Err(anyhow::anyhow!(
                "failed to remove restore temporary file {}: {cleanup_error}; \
                 original restore error: {error:#}",
                temporary.display()
            )),
        },
    }
}

fn remove_abandoned_restore_files(install_dir: &Path) -> anyhow::Result<()> {
    let entries = std::fs::read_dir(install_dir).with_context(|| {
        format!(
            "failed to inspect install directory {} for restore temporary files",
            install_dir.display()
        )
    })?;
    for entry in entries {
        let entry = entry.with_context(|| {
            format!(
                "failed to inspect restore temporary files in {}",
                install_dir.display()
            )
        })?;
        let file_name = entry.file_name();
        if !file_name.to_string_lossy().starts_with(RESTORE_TEMP_PREFIX)
            || !entry.file_type()?.is_file()
        {
            continue;
        }
        std::fs::remove_file(entry.path()).with_context(|| {
            format!(
                "failed to remove abandoned restore temporary file {}",
                entry.path().display()
            )
        })?;
    }
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
