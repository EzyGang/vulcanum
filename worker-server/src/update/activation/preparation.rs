use std::path::{Path, PathBuf};

use anyhow::Context;

use crate::update::activation::{files, CLI_BINARY, WORKER_BINARY};
use crate::update::VERSION_FILE;

const MARKER: &str = ".vulcanum-rollback-preparing";
const PREFIX: &str = ".vulcanum-rollback-preparing-";

pub(super) fn prepare(
    rollback_root: &Path,
    rollback_name: &str,
    installed_cli: &Path,
    installed_worker: &Path,
    installed_version: &Path,
    current_version: &str,
) -> anyhow::Result<PathBuf> {
    std::fs::create_dir_all(rollback_root)
        .with_context(|| format!("failed to create rollback root {}", rollback_root.display()))?;
    files::sync_dir(rollback_root)?;
    let install_dir = rollback_root.parent().with_context(|| {
        format!(
            "rollback root {} has no parent directory",
            rollback_root.display()
        )
    })?;
    files::sync_dir(install_dir)?;
    remove_abandoned(rollback_root)?;

    let preparation = tempfile::Builder::new()
        .prefix(PREFIX)
        .tempdir_in(rollback_root)
        .with_context(|| {
            format!(
                "failed to create rollback preparation in {}",
                rollback_root.display()
            )
        })?;
    let preparation_dir = preparation.path();
    files::sync_dir(rollback_root)?;
    files::backup(installed_cli, &preparation_dir.join(CLI_BINARY))?;
    files::backup(installed_worker, &preparation_dir.join(WORKER_BINARY))?;
    files::backup_version(
        installed_version,
        &preparation_dir.join(VERSION_FILE),
        current_version,
    )?;
    let marker = preparation_dir.join(MARKER);
    std::fs::write(&marker, []).with_context(|| {
        format!(
            "failed to write rollback preparation marker {}",
            marker.display()
        )
    })?;
    files::sync_file(&marker)?;
    files::sync_dir(preparation_dir)?;

    let rollback_dir = rollback_root.join(rollback_name);
    std::fs::rename(preparation_dir, &rollback_dir).with_context(|| {
        format!(
            "failed to publish rollback directory {}",
            rollback_dir.display()
        )
    })?;
    files::sync_dir(rollback_root)?;
    Ok(rollback_dir)
}

pub(super) fn finish(rollback_dir: &Path) -> anyhow::Result<()> {
    let marker = rollback_dir.join(MARKER);
    std::fs::remove_file(&marker).with_context(|| {
        format!(
            "failed to remove rollback preparation marker {}",
            marker.display()
        )
    })?;
    files::sync_dir(rollback_dir)
}

fn remove_abandoned(rollback_root: &Path) -> anyhow::Result<()> {
    let entries = std::fs::read_dir(rollback_root).with_context(|| {
        format!(
            "failed to inspect rollback root {}",
            rollback_root.display()
        )
    })?;
    let mut removed = false;
    for entry in entries {
        let entry = entry.with_context(|| {
            format!(
                "failed to inspect rollback entries in {}",
                rollback_root.display()
            )
        })?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let path = entry.path();
        let file_name = entry.file_name();
        let is_preparation = file_name.to_string_lossy().starts_with(PREFIX);
        let has_marker = path.join(MARKER).try_exists().with_context(|| {
            format!(
                "failed to inspect rollback preparation marker in {}",
                path.display()
            )
        })?;
        if !is_preparation && !has_marker {
            continue;
        }
        std::fs::remove_dir_all(&path).with_context(|| {
            format!(
                "failed to remove abandoned rollback preparation {}",
                path.display()
            )
        })?;
        removed = true;
    }
    if removed {
        files::sync_dir(rollback_root)?;
    }
    Ok(())
}
