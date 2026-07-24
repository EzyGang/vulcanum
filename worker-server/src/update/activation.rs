use std::path::{Path, PathBuf};

use anyhow::Context;
use uuid::Uuid;

use crate::update::VERSION_FILE;

const CLI_BINARY: &str = "vulcanum";
const WORKER_BINARY: &str = "vulcanum-server";
const ROLLBACK_DIR: &str = ".vulcanum-rollback";

pub(super) fn activate_pair(
    staging_dir: &Path,
    install_dir: &Path,
    current_version: &str,
) -> anyhow::Result<PathBuf> {
    activate_pair_with(staging_dir, install_dir, current_version, replace_file)
}

pub(super) fn activate_pair_with<F>(
    staging_dir: &Path,
    install_dir: &Path,
    current_version: &str,
    mut replace: F,
) -> anyhow::Result<PathBuf>
where
    F: FnMut(&Path, &Path) -> std::io::Result<()>,
{
    let installed_cli = install_dir.join(CLI_BINARY);
    let installed_worker = install_dir.join(WORKER_BINARY);
    let installed_version = install_dir.join(VERSION_FILE);
    ensure_pair_exists(&installed_cli, &installed_worker)?;

    let rollback_dir = create_rollback_dir(install_dir, current_version)?;
    backup_file(&installed_cli, &rollback_dir.join(CLI_BINARY))?;
    backup_file(&installed_worker, &rollback_dir.join(WORKER_BINARY))?;
    if installed_version.is_file() {
        backup_file(&installed_version, &rollback_dir.join(VERSION_FILE))?;
    }

    let replacements = [
        (staging_dir.join(CLI_BINARY), installed_cli),
        (staging_dir.join(WORKER_BINARY), installed_worker),
        (staging_dir.join(VERSION_FILE), installed_version),
    ];

    for (source, destination) in &replacements {
        if let Err(update_error) = replace(source, destination) {
            let rollback_result = restore_pair(&rollback_dir, install_dir);
            return match rollback_result {
                Ok(()) => Err(anyhow::anyhow!(
                    "failed to activate {}: {update_error}; restored the previous binary pair",
                    destination.display()
                )),
                Err(rollback_error) => Err(anyhow::anyhow!(
                    "failed to activate {}: {update_error}; rollback also failed: {rollback_error:#}",
                    destination.display()
                )),
            };
        }
    }

    Ok(rollback_dir)
}

fn ensure_pair_exists(cli_path: &Path, worker_path: &Path) -> anyhow::Result<()> {
    if !cli_path.is_file() || !worker_path.is_file() {
        anyhow::bail!(
            "installed vulcanum and vulcanum-server binaries must be in the same directory"
        );
    }
    Ok(())
}

fn create_rollback_dir(install_dir: &Path, current_version: &str) -> anyhow::Result<PathBuf> {
    let safe_version: String = current_version
        .chars()
        .map(|character| {
            match character.is_ascii_alphanumeric() || matches!(character, '.' | '-') {
                true => character,
                false => '_',
            }
        })
        .collect();
    let rollback_dir = install_dir
        .join(ROLLBACK_DIR)
        .join(format!("{safe_version}-{}", Uuid::new_v4()));
    std::fs::create_dir_all(&rollback_dir).with_context(|| {
        format!(
            "failed to create rollback directory {}",
            rollback_dir.display()
        )
    })?;
    Ok(rollback_dir)
}

fn backup_file(source: &Path, destination: &Path) -> anyhow::Result<()> {
    std::fs::copy(source, destination).with_context(|| {
        format!(
            "failed to back up {} to {}",
            source.display(),
            destination.display()
        )
    })?;
    Ok(())
}

fn replace_file(source: &Path, destination: &Path) -> std::io::Result<()> {
    std::fs::rename(source, destination)
}

fn restore_pair(rollback_dir: &Path, install_dir: &Path) -> anyhow::Result<()> {
    let mut errors: Vec<String> = Vec::new();
    for name in [CLI_BINARY, WORKER_BINARY] {
        if let Err(error) = restore_file(&rollback_dir.join(name), &install_dir.join(name)) {
            errors.push(error.to_string());
        }
    }

    let backup_version = rollback_dir.join(VERSION_FILE);
    let installed_version = install_dir.join(VERSION_FILE);
    let version_result = match backup_version.is_file() {
        true => restore_file(&backup_version, &installed_version),
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
        return Ok(());
    }
    anyhow::bail!(errors.join("; "))
}

fn restore_file(source: &Path, destination: &Path) -> anyhow::Result<()> {
    let temporary = destination.with_extension(format!("restore-{}", Uuid::new_v4()));
    std::fs::copy(source, &temporary).with_context(|| {
        format!(
            "failed to copy rollback file {} to {}",
            source.display(),
            temporary.display()
        )
    })?;
    std::fs::rename(&temporary, destination).with_context(|| {
        format!(
            "failed to restore {} from {}",
            destination.display(),
            source.display()
        )
    })?;
    Ok(())
}
