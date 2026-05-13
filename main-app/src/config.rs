use std::sync::OnceLock;

pub struct AppConfig {
    pub db_url: String,
    pub max_conns: u32,
}

impl AppConfig {
    pub fn from_env() -> Result<Self, eyre::Error> {
        dotenvy::dotenv().ok();

        let db_url = std::env::var("DATABASE_URL")?;
        let max_conns = std::env::var("MAX_CONNS")
            .unwrap_or_else(|_| "32".to_owned())
            .parse::<u32>()?;

        Ok(AppConfig { db_url, max_conns })
    }
}

pub fn config() -> &'static AppConfig {
    static CONFIG: OnceLock<AppConfig> = OnceLock::new();
    CONFIG.get_or_init(|| AppConfig::from_env().expect("Failed to load configuration from env"))
}
