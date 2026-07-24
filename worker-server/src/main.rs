mod daemon;
mod isolation;
mod providers;
mod recovery;
mod state;
mod storage;
mod update;

use anyhow::Context;
use vulcanum_shared::config::load_config;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = load_config().context("failed to load worker config")?;
    vulcanum_shared::telemetry::init_with_config(config.debug, config.log_format.as_deref());
    daemon::run().await.context("daemon exited with error")
}
