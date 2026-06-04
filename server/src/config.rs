use std::sync::OnceLock;

pub struct AppConfig {
    pub db_url: String,
    pub max_conns: u32,
    pub poll_period_secs: u64,
    pub jwt_secret: String,
    pub stale_worker_threshold_secs: u64,
    pub unhealthy_threshold: i32,
    pub stalled_running_threshold_secs: u64,
    pub instance_password: String,
    pub redis_url: String,
    pub github_app_id: Option<u64>,
    pub github_app_private_key: Option<String>,
    pub github_app_slug: Option<String>,
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
        let unhealthy_threshold = std::env::var("UNHEALTHY_THRESHOLD")
            .unwrap_or_else(|_| "3".to_owned())
            .parse::<i32>()?;
        let stalled_running_threshold_secs = std::env::var("STALLED_RUNNING_THRESHOLD_SECS")
            .unwrap_or_else(|_| "1800".to_owned())
            .parse::<u64>()?;
        let instance_password = std::env::var("INSTANCE_PASSWORD")?;
        let redis_url =
            std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_owned());
        let github_app_id = std::env::var("GITHUB_APP_ID")
            .ok()
            .and_then(|v| v.parse::<u64>().ok());
        let github_app_private_key = std::env::var("GITHUB_APP_PRIVATE_KEY").ok();
        let github_app_slug = std::env::var("GITHUB_APP_SLUG").ok();

        Ok(Self {
            db_url,
            max_conns,
            poll_period_secs,
            jwt_secret,
            stale_worker_threshold_secs,
            unhealthy_threshold,
            stalled_running_threshold_secs,
            instance_password,
            redis_url,
            github_app_id,
            github_app_private_key,
            github_app_slug,
        })
    }
}

pub fn config() -> &'static AppConfig {
    static CONFIG: OnceLock<AppConfig> = OnceLock::new();
    CONFIG.get_or_init(|| AppConfig::from_env().expect("Failed to load configuration from env"))
}
