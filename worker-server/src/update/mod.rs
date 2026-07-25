mod activation;
mod archive;
mod install;
mod release;
mod service;
mod startup;
#[cfg(test)]
mod tests;

use std::path::PathBuf;
use std::time::Duration;

use anyhow::Context;

use crate::update::release::{fetch_latest_release, is_newer, select_package};
use crate::update::service::{PlatformServiceRestarter, ServiceRestarter};

pub(crate) const VERSION_FILE: &str = ".vulcanum-version";
const LATEST_RELEASE_URL: &str = "https://api.github.com/repos/EzyGang/vulcanum/releases/latest";
const UPDATE_HTTP_TIMEOUT: Duration = Duration::from_secs(60);

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum UpdateOutcome {
    UpToDate {
        version: String,
    },
    Applied {
        previous_version: String,
        target_version: String,
        rollback_dir: PathBuf,
    },
    Failed {
        current_version: String,
        target_version: Option<String>,
        rollback_dir: Option<PathBuf>,
        error: String,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum StartupUpdate {
    Clean,
    Recovered(PathBuf),
    Pending(PathBuf),
}

impl UpdateOutcome {
    pub(crate) fn log(&self) {
        match self {
            Self::UpToDate { version } => {
                tracing::info!(
                    version,
                    "automatic update check: installation is up to date"
                );
            }
            Self::Applied {
                previous_version,
                target_version,
                rollback_dir,
            } => {
                tracing::info!(
                    previous_version,
                    target_version,
                    rollback_dir = %rollback_dir.display(),
                    "automatic update applied; worker service restart requested"
                );
            }
            Self::Failed {
                current_version,
                target_version,
                rollback_dir,
                error,
            } => {
                tracing::warn!(
                    current_version,
                    target_version = target_version.as_deref().unwrap_or("unknown"),
                    rollback_dir = rollback_dir.as_ref().map_or_else(
                        || "unavailable".to_owned(),
                        |path| path.display().to_string()
                    ),
                    error,
                    "automatic update failed; existing installation retained or recovery required"
                );
            }
        }
    }

    #[must_use]
    pub(crate) const fn is_applied(&self) -> bool {
        matches!(self, Self::Applied { .. })
    }
}

pub(crate) struct AutomaticUpdater<R = PlatformServiceRestarter> {
    client: reqwest::Client,
    api_url: String,
    install_dir: PathBuf,
    current_version: String,
    target: String,
    restarter: R,
}

impl<R> AutomaticUpdater<R>
where
    R: ServiceRestarter,
{
    fn new(
        api_url: String,
        install_dir: PathBuf,
        current_version: String,
        target: String,
        restarter: R,
    ) -> anyhow::Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(UPDATE_HTTP_TIMEOUT)
            .user_agent("vulcanum-worker-updater")
            .build()
            .context("failed to create update HTTP client")?;
        Ok(Self {
            client,
            api_url,
            install_dir,
            current_version,
            target,
            restarter,
        })
    }

    pub(crate) async fn check_and_apply(&self) -> UpdateOutcome {
        let release = match fetch_latest_release(&self.client, &self.api_url).await {
            Ok(release) => release,
            Err(error) => return self.failure(None, error),
        };
        let target_version = release.tag_name.clone();

        match is_newer(&self.current_version, &target_version) {
            Ok(false) => {
                return UpdateOutcome::UpToDate {
                    version: self.current_version.clone(),
                };
            }
            Ok(true) => (),
            Err(error) => return self.failure(Some(target_version), error),
        }

        let package = match select_package(&release, &self.target) {
            Ok(package) => package,
            Err(error) => return self.failure(Some(target_version), error),
        };
        let staging = match tempfile::Builder::new()
            .prefix(".vulcanum-update-")
            .tempdir_in(&self.install_dir)
        {
            Ok(staging) => staging,
            Err(error) => {
                return self.failure(
                    Some(target_version),
                    anyhow::anyhow!("failed to create update staging directory: {error}"),
                );
            }
        };
        let archive_path = staging.path().join(&package.archive_name);
        let checksum_path = staging
            .path()
            .join(format!("{}.sha256", package.archive_name));

        if let Err(error) = archive::download(
            &self.client,
            &package.archive_url,
            &archive_path,
            archive::MAX_ARCHIVE_BYTES,
        )
        .await
        {
            return self.failure(Some(target_version), error);
        }
        if let Err(error) = archive::download(
            &self.client,
            &package.checksum_url,
            &checksum_path,
            archive::MAX_CHECKSUM_BYTES,
        )
        .await
        {
            return self.failure(Some(target_version), error);
        }

        let archive_for_task = archive_path.clone();
        let checksum_for_task = checksum_path.clone();
        let staging_for_task = staging.path().to_path_buf();
        let stage_result = tokio::task::spawn_blocking(move || {
            archive::verify_and_extract(&archive_for_task, &checksum_for_task, &staging_for_task)
        })
        .await;
        match stage_result {
            Ok(Ok(())) => (),
            Ok(Err(error)) => return self.failure(Some(target_version), error),
            Err(error) => {
                return self.failure(
                    Some(target_version),
                    anyhow::anyhow!("update staging task failed: {error}"),
                );
            }
        }

        let staged_version = staging.path().join(VERSION_FILE);
        if let Err(error) = std::fs::write(&staged_version, format!("{target_version}\n")) {
            return self.failure(
                Some(target_version),
                anyhow::anyhow!("failed to stage release version marker: {error}"),
            );
        }

        if let Err(error) = install::preflight_worker(&staging.path().join("vulcanum-server")).await
        {
            return self.failure(Some(target_version), error);
        }

        let rollback_dir = match activation::activate_pair(
            staging.path(),
            &self.install_dir,
            &self.current_version,
        ) {
            Ok(rollback_dir) => rollback_dir,
            Err(error) => return self.failure(Some(target_version), error),
        };

        if let Err(restart_error) = self.restarter.restart().await {
            let rollback_result = activation::rollback_pair(&rollback_dir, &self.install_dir);
            return match rollback_result {
                Ok(()) => UpdateOutcome::Failed {
                    current_version: self.current_version.clone(),
                    target_version: Some(target_version),
                    rollback_dir: Some(rollback_dir),
                    error: format!(
                        "worker service restart failed: {restart_error:#}; restored the previous release pair"
                    ),
                },
                Err(rollback_error) => UpdateOutcome::Failed {
                    current_version: target_version.clone(),
                    target_version: Some(target_version),
                    rollback_dir: Some(rollback_dir),
                    error: format!(
                        "worker service restart failed: {restart_error:#}; rollback is pending recovery after also failing: {rollback_error:#}"
                    ),
                },
            };
        }

        UpdateOutcome::Applied {
            previous_version: self.current_version.clone(),
            target_version,
            rollback_dir,
        }
    }

    fn failure(&self, target_version: Option<String>, error: anyhow::Error) -> UpdateOutcome {
        UpdateOutcome::Failed {
            current_version: self.current_version.clone(),
            target_version,
            rollback_dir: None,
            error: format!("{error:#}"),
        }
    }
}
