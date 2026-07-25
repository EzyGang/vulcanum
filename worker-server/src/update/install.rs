use std::path::{Path, PathBuf};

use anyhow::Context;

use crate::update::VERSION_FILE;

const BUILD_VERSION: &str = match option_env!("VULCANUM_RELEASE_VERSION") {
    Some(version) => version,
    None => env!("CARGO_PKG_VERSION"),
};

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
