use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Component, Path, PathBuf};

use anyhow::Context;
use uuid::Uuid;

use crate::update::activation::{files, ROLLBACK_DIR};

const STATE_FILE: &str = ".vulcanum-update-state";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Phase {
    Activating,
    PendingRestart,
    Verifying,
}

impl Phase {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Activating => "activating",
            Self::PendingRestart => "pending_restart",
            Self::Verifying => "verifying",
        }
    }

    fn parse(value: &str) -> Option<Self> {
        match value {
            "activating" => Some(Self::Activating),
            "pending_restart" => Some(Self::PendingRestart),
            "verifying" => Some(Self::Verifying),
            _ => None,
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub(super) struct Transaction {
    pub(super) phase: Phase,
    pub(super) rollback_dir: PathBuf,
}

pub(super) fn read(install_dir: &Path) -> anyhow::Result<Option<Transaction>> {
    let state_path = install_dir.join(STATE_FILE);
    let state = match std::fs::read_to_string(&state_path) {
        Ok(state) => state,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => {
            return Err(error)
                .with_context(|| format!("failed to read update state {}", state_path.display()));
        }
    };
    let mut lines = state.lines();
    let first = lines
        .next()
        .ok_or_else(|| anyhow::anyhow!("update state is empty"))?;
    let (phase, rollback_name) = match Phase::parse(first) {
        Some(phase) => {
            let name = lines
                .next()
                .ok_or_else(|| anyhow::anyhow!("update state has no rollback directory"))?;
            (phase, name)
        }
        None => (Phase::Activating, first),
    };
    if lines.next().is_some() {
        anyhow::bail!("update state contains unexpected data");
    }
    let rollback_name = valid_rollback_name(rollback_name)?;
    Ok(Some(Transaction {
        phase,
        rollback_dir: install_dir.join(ROLLBACK_DIR).join(rollback_name),
    }))
}

pub(super) fn write(install_dir: &Path, rollback_dir: &Path, phase: Phase) -> anyhow::Result<()> {
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
        writeln!(file, "{}", phase.as_str()).context("failed to write update state phase")?;
        writeln!(file, "{}", rollback_name.to_string_lossy())
            .context("failed to write update rollback directory")?;
        file.sync_all().context("failed to sync update state")?;
        std::fs::rename(&temporary, &state_path)
            .with_context(|| format!("failed to publish update state {}", state_path.display()))?;
        files::sync_dir(install_dir)
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&temporary);
    }
    result
}

pub(super) fn remove(install_dir: &Path) -> anyhow::Result<()> {
    let state_path = install_dir.join(STATE_FILE);
    std::fs::remove_file(&state_path)
        .with_context(|| format!("failed to remove update state {}", state_path.display()))?;
    files::sync_dir(install_dir)
}

fn valid_rollback_name(value: &str) -> anyhow::Result<&Path> {
    let path = Path::new(value);
    let mut components = path.components();
    match (components.next(), components.next()) {
        (Some(Component::Normal(_)), None) => Ok(path),
        _ => anyhow::bail!("update state contains an invalid rollback directory"),
    }
}
