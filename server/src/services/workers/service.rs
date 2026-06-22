use std::sync::Arc;

use sqlx::PgPool;

use crate::config::AppConfig;
use crate::services::work_runs::repository::WorkRunsRepository;
use crate::services::workers::registration_code_store::CodeStore;
use crate::services::workers::repository::WorkersRepository;

pub mod connect;
pub mod delete;
pub mod generate_code;
pub mod list;
pub mod refresh;
pub mod set_status;
mod token;

#[cfg(test)]
pub mod workers_tests;

#[derive(Clone)]
pub struct WorkersService {
    pub repo: WorkersRepository,
    pub work_runs_repo: WorkRunsRepository,
    pub db: PgPool,
    jwt_secret: String,
    pub(crate) code_store: Arc<dyn CodeStore>,
    pub unhealthy_threshold: i32,
}

impl WorkersService {
    pub fn new(
        repo: WorkersRepository,
        work_runs_repo: WorkRunsRepository,
        db: PgPool,
        config: &AppConfig,
        code_store: Arc<dyn CodeStore>,
    ) -> Self {
        Self {
            repo,
            work_runs_repo,
            db,
            jwt_secret: config.jwt_secret.clone(),
            code_store,
            unhealthy_threshold: config.unhealthy_threshold,
        }
    }
}
