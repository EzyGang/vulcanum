use std::sync::OnceLock;

use sqlx::postgres::PgPoolOptions;
use tokio::signal;

use vulcanum_server::services::dispatcher::flag_store::RedisDispatchStore;
use vulcanum_server::services::dispatcher::repository::DispatchRepository;
use vulcanum_server::services::dispatcher::service::DispatcherService;
use vulcanum_server::services::work_runs::repository::WorkRunsRepository;
use vulcanum_server::services::workers::repository::WorkersRepository;

struct DispatcherConfig {
    db_url: String,
    redis_url: String,
    dispatch_interval_secs: u64,
    stale_worker_threshold_secs: u64,
}

impl DispatcherConfig {
    fn from_env() -> Result<Self, eyre::Error> {
        let db_url = std::env::var("DATABASE_URL")?;
        let redis_url =
            std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_owned());
        let dispatch_interval_secs = std::env::var("DISPATCH_INTERVAL_SECS")
            .unwrap_or_else(|_| "15".to_owned())
            .parse::<u64>()?;
        let stale_worker_threshold_secs = std::env::var("STALE_WORKER_THRESHOLD_SECS")
            .unwrap_or_else(|_| "120".to_owned())
            .parse::<u64>()?;

        Ok(Self {
            db_url,
            redis_url,
            dispatch_interval_secs,
            stale_worker_threshold_secs,
        })
    }
}

fn config() -> &'static DispatcherConfig {
    static CONFIG: OnceLock<DispatcherConfig> = OnceLock::new();
    CONFIG.get_or_init(|| DispatcherConfig::from_env().expect("Failed to load config from env"))
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    dotenvy::dotenv().ok();
    vulcanum_shared::telemetry::init();

    let cfg = config();

    let db_pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&cfg.db_url)
        .await?;

    tracing::info!("Applying migrations...");
    sqlx::migrate!().run(&db_pool).await?;

    let dispatch_store =
        RedisDispatchStore::new(&cfg.redis_url).map_err(|e| eyre::eyre!(e.to_string()))?;

    let svc = DispatcherService::new(
        DispatchRepository::new(),
        WorkersRepository::new(),
        WorkRunsRepository::new(),
        db_pool,
        std::sync::Arc::new(dispatch_store),
        cfg.stale_worker_threshold_secs,
    );

    tracing::info!(
        interval_secs = cfg.dispatch_interval_secs,
        "dispatcher started",
    );

    let mut interval =
        tokio::time::interval(std::time::Duration::from_secs(cfg.dispatch_interval_secs));

    loop {
        tokio::select! {
            _ = signal::ctrl_c() => {
                tracing::info!("received SIGINT, shutting down");
                return Ok(());
            }
            _ = interval.tick() => {
                match svc.dispatch_once().await {
                    Ok(summary) => {
                        tracing::debug!(
                            dispatched = summary.dispatched,
                            workers = summary.workers_available,
                            pending = summary.pending_jobs,
                            disconnected = summary.disconnected,
                            "dispatch cycle complete",
                        );
                    }
                    Err(e) => {
                        tracing::error!("dispatch cycle failed: {e}");
                    }
                }
            }
        }
    }
}
