mod daemon;
mod isolation;
mod providers;
mod recovery;
mod state;
mod storage;
mod update;

use std::ffi::OsStr;
use std::path::Path;

use anyhow::Context;
use vulcanum_shared::config::load_config;

use crate::update::{AutomaticUpdater, StartupUpdate};

const UPDATE_PREFLIGHT_ARG: &str = "--vulcanum-update-preflight";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    if std::env::args_os().nth(1).as_deref() == Some(OsStr::new(UPDATE_PREFLIGHT_ARG)) {
        return Ok(());
    }

    let startup_update = AutomaticUpdater::prepare_current_install()?;
    let pending_rollback = match startup_update {
        StartupUpdate::Clean => None,
        StartupUpdate::Recovered(rollback_dir) => {
            anyhow::bail!(
                "recovered an interrupted automatic update from {}; restarting the worker daemon",
                rollback_dir.display()
            );
        }
        StartupUpdate::Pending(rollback_dir) => Some(rollback_dir),
    };

    match run_worker(pending_rollback.as_deref()).await {
        Ok(()) => Ok(()),
        Err(error) => match AutomaticUpdater::rollback_current_install() {
            Ok(Some(rollback_dir)) => Err(error.context(format!(
                "worker failed; restored the previous release pair from {}",
                rollback_dir.display()
            ))),
            Ok(None) => Err(error),
            Err(rollback_error) => Err(error.context(format!(
                "worker failed while update rollback also failed: {rollback_error:#}"
            ))),
        },
    }
}

async fn run_worker(pending_rollback: Option<&Path>) -> anyhow::Result<()> {
    let config = load_config().context("failed to load worker config")?;
    vulcanum_shared::telemetry::init_with_config(config.debug, config.log_format.as_deref());
    daemon::run(pending_rollback)
        .await
        .context("daemon exited with error")
}
