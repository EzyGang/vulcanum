use std::sync::Arc;

use sqlx::PgPool;

use crate::config::AppConfig;
use crate::services::workers::code_store::CodeStore;
use crate::services::workers::repository::WorkersRepository;

pub mod workers;

#[cfg(test)]
pub mod workers_tests;

#[derive(Clone)]
pub struct WorkersService {
    pub repo: WorkersRepository,
    pub db: PgPool,
    jwt_secret: String,
    pub(crate) code_store: Arc<dyn CodeStore>,
}

impl WorkersService {
    pub fn new(
        repo: WorkersRepository,
        db: PgPool,
        config: &AppConfig,
        code_store: Arc<dyn CodeStore>,
    ) -> Self {
        Self {
            repo,
            db,
            jwt_secret: config.jwt_secret.clone(),
            code_store,
        }
    }
}
