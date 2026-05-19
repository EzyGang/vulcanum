pub mod workers;

#[cfg(test)]
pub mod workers_tests;

use sqlx::PgPool;

use crate::config::AppConfig;
use crate::services::workers::repository::WorkersRepository;

pub struct WorkersService {
    pub repo: WorkersRepository,
    pub db: PgPool,
    jwt_secret: String,
    codes: tokio::sync::RwLock<std::collections::HashMap<String, chrono::DateTime<chrono::Utc>>>,
}

impl Clone for WorkersService {
    fn clone(&self) -> Self {
        Self {
            repo: self.repo.clone(),
            db: self.db.clone(),
            jwt_secret: self.jwt_secret.clone(),
            codes: tokio::sync::RwLock::new(std::collections::HashMap::new()),
        }
    }
}

impl WorkersService {
    pub fn new(repo: WorkersRepository, db: PgPool, config: &AppConfig) -> Self {
        Self {
            repo,
            db,
            jwt_secret: config.jwt_secret.clone(),
            codes: tokio::sync::RwLock::new(std::collections::HashMap::new()),
        }
    }
}
