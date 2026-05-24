use std::sync::OnceLock;

pub struct AppConfig {
    pub db_url: String,
    pub max_conns: u32,
    pub poll_period_secs: u64,
    pub jwt_secret: String,
    pub stale_worker_threshold_secs: u64,
    pub instance_password: String,
    pub kaneo_instance: String,
    pub kaneo_api_key: String,
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
        let kaneo_instance =
            std::env::var("KANEO_INSTANCE").unwrap_or_else(|_| "cloud.kaneo.app".to_owned());
        let kaneo_api_key = std::env::var("KANEO_API_KEY").unwrap_or_default();

        Ok(AppConfig {
            db_url,
            max_conns,
            poll_period_secs,
            jwt_secret,
            stale_worker_threshold_secs,
            instance_password,
            kaneo_instance,
            kaneo_api_key,
        })
    }
}

pub fn config() -> &'static AppConfig {
    static CONFIG: OnceLock<AppConfig> = OnceLock::new();
    CONFIG.get_or_init(|| AppConfig::from_env().expect("Failed to load configuration from env"))
}
