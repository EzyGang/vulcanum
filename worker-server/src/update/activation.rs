use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::{Component, Path, PathBuf};

use anyhow::Context;
use uuid::Uuid;

use crate::update::VERSION_FILE;

const CLI_BINARY: &str = "vulcanum";
const WORKER_BINARY: &str = "vulcanum-server";
const ROLLBACK_DIR: &str = ".vulcanum-rollback";
const STATE_FILE: &str = ".vulcanum-update-state";

pub(super) fn recover_interrupted_activation(
    install_dir: &Path,
) -> anyhow::Result<Option<PathBuf>> {
    let state_path = install_dir.join(STATE_FILE);
    let state = match std::fs::read_to_string(&state_path) {
        Ok(state) => state,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => {
            return Err(error)
                .with_context(|| format!("failed to read update state {}", state_path.display()));
        }
    };
    let rollback_name = valid_rollback_name(state.trim())?;
    let rollback_dir = install_dir.join(ROLLBACK_DIR).join(rollback_name);
    restore_pair(&rollback_dir, install_dir)?;
    commit_state(install_dir)?;
    Ok(Some(rollback_dir))
}

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
    let _ = recover_interrupted_activation(install_dir)?;
    let installed_cli = install_dir.join(CLI_BINARY);
    let installed_worker = install_dir.join(WORKER_BINARY);
    let installed_version = install_dir.join(VERSION_FILE);
    ensure_pair_exists(&installed_cli, &installed_worker)?;

    let staged_cli = staging_dir.join(CLI_BINARY);
    let staged_worker = staging_dir.join(WORKER_BINARY);
    let staged_version = staging_dir.join(VERSION_FILE);
    for path in [&staged_cli, &staged_worker, &staged_version] {
        sync_file(path)?;
    }
    sync_dir(staging_dir)?;

    let rollback_dir = create_rollback_dir(install_dir, current_version)?;
    backup_file(&installed_cli, &rollback_dir.join(CLI_BINARY))?;
    backup_file(&installed_worker, &rollback_dir.join(WORKER_BINARY))?;
    if installed_version.is_file() {
        backup_file(&installed_version, &rollback_dir.join(VERSION_FILE))?;
    }
    sync_dir(&rollback_dir)?;
    write_state(install_dir, &rollback_dir)?;

    let replacements = [
        (staged_cli, installed_cli),
        (staged_worker, installed_worker),
        (staged_version, installed_version),
    ];

    for (source, destination) in &replacements {
        if let Err(update_error) = replace(source, destination) {
            let rollback_result = recover_interrupted_activation(install_dir);
            return match rollback_result {
                Ok(_) => Err(anyhow::anyhow!(
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

    sync_dir(install_dir)?;
    commit_state(install_dir)?;
    Ok(rollback_dir)
}

pub(super) fn rollback_pair(rollback_dir: &Path, install_dir: &Path) -> anyhow::Result<()> {
    write_state(install_dir, rollback_dir)?;
    recover_interrupted_activation(install_dir).map(|_| ())
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
    let rollback_root = install_dir.join(ROLLBACK_DIR);
    let rollback_dir = rollback_root.join(format!("{safe_version}-{}", Uuid::new_v4()));
    std::fs::create_dir_all(&rollback_dir).with_context(|| {
        format!(
            "failed to create rollback directory {}",
            rollback_dir.display()
        )
    })?;
    sync_dir(&rollback_dir)?;
    sync_dir(&rollback_root)?;
    sync_dir(install_dir)?;
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
    sync_file(destination)
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

    if !errors.is_empty() {
        anyhow::bail!(errors.join("; "));
    }
    sync_dir(install_dir)
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

fn write_state(install_dir: &Path, rollback_dir: &Path) -> anyhow::Result<()> {
    let rollback_name = rollback_dir
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| anyhow::anyhow!("rollback directory has no valid name"))?;
    let rollback_name = valid_rollback_name(rollback_name)?;
    let state_path = install_dir.join(STATE_FILE);
    let temporary = install_dir.join(format!("{STATE_FILE}.{}", Uuid::new_v4()));
    let result = (|| -> anyhow::Result<()> {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)
            .with_context(|| format!("failed to create update state {}", temporary.display()))?;
        writeln!(file, "{}", rollback_name.to_string_lossy())
            .context("failed to write update state")?;
        file.sync_all().context("failed to sync update state")?;
        std::fs::rename(&temporary, &state_path)
            .with_context(|| format!("failed to publish update state {}", state_path.display()))?;
        sync_dir(install_dir)
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&temporary);
    }
    result
}

fn commit_state(install_dir: &Path) -> anyhow::Result<()> {
    let state_path = install_dir.join(STATE_FILE);
    std::fs::remove_file(&state_path)
        .with_context(|| format!("failed to remove update state {}", state_path.display()))?;
    sync_dir(install_dir)
}

fn valid_rollback_name(value: &str) -> anyhow::Result<&Path> {
    let path = Path::new(value);
    let mut components = path.components();
    match (components.next(), components.next()) {
        (Some(Component::Normal(_)), None) => Ok(path),
        _ => anyhow::bail!("update state contains an invalid rollback directory"),
    }
}

fn sync_file(path: &Path) -> anyhow::Result<()> {
    File::open(path)
        .and_then(|file| file.sync_all())
        .with_context(|| format!("failed to sync {}", path.display()))
}

#[cfg(unix)]
fn sync_dir(path: &Path) -> anyhow::Result<()> {
    File::open(path)
        .and_then(|directory| directory.sync_all())
        .with_context(|| format!("failed to sync directory {}", path.display()))
}

#[cfg(not(unix))]
fn sync_dir(_path: &Path) -> anyhow::Result<()> {
    Ok(())
}
