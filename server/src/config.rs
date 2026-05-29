use std::sync::OnceLock;

pub struct AppConfig {
    pub db_url: String,
    pub max_conns: u32,
    pub poll_period_secs: u64,
    pub jwt_secret: String,
    pub stale_worker_threshold_secs: u64,
    pub instance_password: String,
    pub redis_url: String,
}

impl AppConfig {
    pub fn from_env() -> Result<Self, eyre::Error> {
        let db_url = std::env::var("DATABASE_URL")?;
        let max_conns = std::env::var("MAX_CONNS")
            .unwrap_or_else(|_| "32".to_owned())
            .parse::<u32>()?;
        let poll_period_secs = std::env::var("POLL_PERIOD_SECS")
            .unwrap_or_else(|_| "30".to_owned())
            .parse::<u64>()?;
        let jwt_secret = std::env::var("JWT_SECRET")?;
        let stale_worker_threshold_secs = std::env::var("STALE_WORKER_THRESHOLD_SECS")
            .unwrap_or_else(|_| "120".to_owned())
            .parse::<u64>()?;
        let instance_password = std::env::var("INSTANCE_PASSWORD")?;
        let redis_url =
            std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_owned());

        Ok(Self {
            db_url,
            max_conns,
            poll_period_secs,
            jwt_secret,
            stale_worker_threshold_secs,
            instance_password,
            redis_url,
        })
    }
}

pub fn config() -> &'static AppConfig {
    static CONFIG: OnceLock<AppConfig> = OnceLock::new();
    CONFIG.get_or_init(|| AppConfig::from_env().expect("Failed to load configuration from env"))
}
