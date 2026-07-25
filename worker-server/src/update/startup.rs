use std::path::{Path, PathBuf};

use crate::update::activation::{self, StartupActivation};
use crate::update::install;
use crate::update::release::current_target;
use crate::update::service::PlatformServiceRestarter;
use crate::update::{AutomaticUpdater, StartupUpdate, LATEST_RELEASE_URL};

impl AutomaticUpdater<PlatformServiceRestarter> {
    pub(crate) fn prepare_current_install() -> anyhow::Result<StartupUpdate> {
        let install_dir = install::current_dir()?;
        match activation::prepare_startup(&install_dir)? {
            StartupActivation::Clean => Ok(StartupUpdate::Clean),
            StartupActivation::Recovered(rollback_dir) => {
                Ok(StartupUpdate::Recovered(rollback_dir))
            }
            StartupActivation::Pending(rollback_dir) => Ok(StartupUpdate::Pending(rollback_dir)),
        }
    }

    pub(crate) fn confirm_current_install(rollback_dir: &Path) -> anyhow::Result<()> {
        let install_dir = install::current_dir()?;
        activation::confirm_pending_activation(&install_dir, rollback_dir)
    }

    pub(crate) fn rollback_current_install() -> anyhow::Result<Option<PathBuf>> {
        let install_dir = install::current_dir()?;
        activation::rollback_pending_activation(&install_dir)
    }

    pub(crate) fn current_install_has_transaction() -> anyhow::Result<bool> {
        let install_dir = install::current_dir()?;
        activation::has_transaction(&install_dir)
    }

    pub(crate) fn for_current_install() -> anyhow::Result<Self> {
        let install_dir = install::current_dir()?;
        let current_version = install::read_version(&install_dir)?;
        Self::new(
            LATEST_RELEASE_URL.to_owned(),
            install_dir,
            current_version,
            current_target()?.to_owned(),
            PlatformServiceRestarter,
        )
    }
}
