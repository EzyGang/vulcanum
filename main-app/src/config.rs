use std::sync::OnceLock;

pub struct AppConfig {
    pub db_url: String,
    pub max_conns: u32,
    pub poll_period_secs: u64,
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

        Ok(AppConfig {
            db_url,
            max_conns,
            poll_period_secs,
        })
    }
}

pub fn config() -> &'static AppConfig {
    static CONFIG: OnceLock<AppConfig> = OnceLock::new();
    CONFIG.get_or_init(|| AppConfig::from_env().expect("Failed to load configuration from env"))
}
