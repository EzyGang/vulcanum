mod daemon;
mod harness;
mod state;

use anyhow::Context;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    vulcanum_shared::telemetry::init();
    daemon::run().await.context("daemon exited with error")
}
