#[tokio::main]
async fn main() {
    vulcanum_shared::telemetry::init();
    tracing::info!("vulcanum-worker-server placeholder");
    std::process::exit(0);
}
