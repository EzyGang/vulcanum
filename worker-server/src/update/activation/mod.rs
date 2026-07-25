mod files;
mod state;

use std::path::{Path, PathBuf};

use anyhow::Context;
use uuid::Uuid;

use crate::update::activation::state::{Phase, Transaction};
use crate::update::VERSION_FILE;

const CLI_BINARY: &str = "vulcanum";
const WORKER_BINARY: &str = "vulcanum-server";
const ROLLBACK_DIR: &str = ".vulcanum-rollback";

#[derive(Debug, Eq, PartialEq)]
pub(super) enum StartupActivation {
    Clean,
    Recovered(PathBuf),
    Pending(PathBuf),
}

pub(super) fn prepare_startup(install_dir: &Path) -> anyhow::Result<StartupActivation> {
    let Some(transaction) = state::read(install_dir)? else {
        return Ok(StartupActivation::Clean);
    };
    if transaction.phase == Phase::PendingRestart {
        state::write(install_dir, &transaction.rollback_dir, Phase::Verifying)?;
        return Ok(StartupActivation::Pending(transaction.rollback_dir));
    }

    let rollback_dir = recover_transaction(install_dir, transaction)?;
    Ok(StartupActivation::Recovered(rollback_dir))
}

pub(super) fn confirm_pending_activation(
    install_dir: &Path,
    expected_rollback_dir: &Path,
) -> anyhow::Result<()> {
    let transaction = state::read(install_dir)?
        .ok_or_else(|| anyhow::anyhow!("pending update transaction disappeared"))?;
    ensure_expected_transaction(&transaction, expected_rollback_dir)?;
    if transaction.phase != Phase::Verifying {
        anyhow::bail!("pending update is not awaiting startup verification");
    }
    state::remove(install_dir)
}

pub(super) fn rollback_pending_activation(install_dir: &Path) -> anyhow::Result<Option<PathBuf>> {
    let Some(transaction) = state::read(install_dir)? else {
        return Ok(None);
    };
    state::write(install_dir, &transaction.rollback_dir, Phase::Activating)?;
    recover_transaction(install_dir, transaction).map(Some)
}

pub(super) fn has_transaction(install_dir: &Path) -> anyhow::Result<bool> {
    state::read(install_dir).map(|transaction| transaction.is_some())
}

pub(super) fn recover_interrupted_activation(
    install_dir: &Path,
) -> anyhow::Result<Option<PathBuf>> {
    let Some(transaction) = state::read(install_dir)? else {
        return Ok(None);
    };
    recover_transaction(install_dir, transaction).map(Some)
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
    files::ensure_pair_exists(&installed_cli, &installed_worker)?;

    let staged_cli = staging_dir.join(CLI_BINARY);
    let staged_worker = staging_dir.join(WORKER_BINARY);
    let staged_version = staging_dir.join(VERSION_FILE);
    for path in [&staged_cli, &staged_worker, &staged_version] {
        files::sync_file(path)?;
    }
    files::sync_dir(staging_dir)?;

    let rollback_dir = create_rollback_dir(install_dir, current_version)?;
    files::backup(&installed_cli, &rollback_dir.join(CLI_BINARY))?;
    files::backup(&installed_worker, &rollback_dir.join(WORKER_BINARY))?;
    if installed_version.is_file() {
        files::backup(&installed_version, &rollback_dir.join(VERSION_FILE))?;
    }
    files::sync_dir(&rollback_dir)?;
    state::write(install_dir, &rollback_dir, Phase::Activating)?;

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

    files::sync_dir(install_dir)?;
    state::write(install_dir, &rollback_dir, Phase::PendingRestart)?;
    Ok(rollback_dir)
}

pub(super) fn rollback_pair(rollback_dir: &Path, install_dir: &Path) -> anyhow::Result<()> {
    state::read(install_dir)?
        .map(|transaction| ensure_expected_transaction(&transaction, rollback_dir))
        .transpose()?;
    state::write(install_dir, rollback_dir, Phase::Activating)?;
    recover_interrupted_activation(install_dir).map(|_| ())
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
    files::sync_dir(&rollback_dir)?;
    files::sync_dir(&rollback_root)?;
    files::sync_dir(install_dir)?;
    Ok(rollback_dir)
}

fn recover_transaction(install_dir: &Path, transaction: Transaction) -> anyhow::Result<PathBuf> {
    files::restore_pair(&transaction.rollback_dir, install_dir)?;
    state::remove(install_dir)?;
    Ok(transaction.rollback_dir)
}

fn ensure_expected_transaction(
    transaction: &Transaction,
    expected_rollback_dir: &Path,
) -> anyhow::Result<()> {
    if transaction.rollback_dir != expected_rollback_dir {
        anyhow::bail!("update transaction references an unexpected rollback directory");
    }
    Ok(())
}

fn replace_file(source: &Path, destination: &Path) -> std::io::Result<()> {
    std::fs::rename(source, destination)
}
