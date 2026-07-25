use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Duration;

use anyhow::Context;

use crate::update::VERSION_FILE;

const BUILD_VERSION: &str = match option_env!("VULCANUM_RELEASE_VERSION") {
    Some(version) => version,
    None => env!("CARGO_PKG_VERSION"),
};
const PREFLIGHT_TIMEOUT: Duration = Duration::from_secs(10);

pub(super) fn current_dir() -> anyhow::Result<PathBuf> {
    let executable = std::env::current_exe().context("failed to locate the worker executable")?;
    executable
        .parent()
        .map(Path::to_path_buf)
        .ok_or_else(|| anyhow::anyhow!("worker executable has no parent directory"))
}

pub(super) fn read_version(install_dir: &Path) -> anyhow::Result<String> {
    let path = install_dir.join(VERSION_FILE);
    match std::fs::read_to_string(&path) {
        Ok(version) => {
            let version = version.trim();
            if version.is_empty() {
                anyhow::bail!("installed release marker {} is empty", path.display());
            }
            Ok(version.to_owned())
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(BUILD_VERSION.to_owned()),
        Err(error) => Err(error)
            .with_context(|| format!("failed to read installed release marker {}", path.display())),
    }
}

pub(super) async fn preflight_worker(path: &Path) -> anyhow::Result<()> {
    let mut command = tokio::process::Command::new(path);
    command
        .arg("--vulcanum-update-preflight")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .kill_on_drop(true);
    let status = tokio::time::timeout(PREFLIGHT_TIMEOUT, command.status())
        .await
        .context("staged worker preflight timed out")?
        .with_context(|| format!("failed to execute staged worker {}", path.display()))?;
    if !status.success() {
        anyhow::bail!("staged worker preflight failed with {status}");
    }
    Ok(())
}
