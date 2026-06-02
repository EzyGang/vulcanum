pub mod submit_result;
pub mod work_runs;

#[cfg(test)]
mod work_runs_tests;

use std::sync::Arc;

use sqlx::PgPool;

use crate::services::dispatcher::cancel_store::CancelStore;
use crate::services::dispatcher::flag_store::DispatchStore;
use crate::services::integration_providers::repository::IntegrationProvidersRepository;
use crate::services::project_configs::repository::ProjectConfigsRepository;
use crate::services::work_runs::repository::WorkRunsRepository;
use crate::services::workers::repository::WorkersRepository;

pub struct WorkRunsService {
    pub work_runs_repo: WorkRunsRepository,
    pub workers_repo: WorkersRepository,
    pub project_configs_repo: ProjectConfigsRepository,
    pub db: PgPool,
    pub dispatch_store: Arc<dyn DispatchStore>,
    pub cancel_store: Arc<dyn CancelStore>,
    pub providers_repo: IntegrationProvidersRepository,
    pub unhealthy_threshold: i32,
}

impl Clone for WorkRunsService {
    fn clone(&self) -> Self {
        Self {
            work_runs_repo: self.work_runs_repo.clone(),
            workers_repo: self.workers_repo.clone(),
            project_configs_repo: self.project_configs_repo.clone(),
            db: self.db.clone(),
            dispatch_store: self.dispatch_store.clone(),
            cancel_store: self.cancel_store.clone(),
            providers_repo: self.providers_repo.clone(),
            unhealthy_threshold: self.unhealthy_threshold,
        }
    }
}

impl WorkRunsService {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        work_runs_repo: WorkRunsRepository,
        workers_repo: WorkersRepository,
        project_configs_repo: ProjectConfigsRepository,
        db: PgPool,
        dispatch_store: Arc<dyn DispatchStore>,
        providers_repo: IntegrationProvidersRepository,
        cancel_store: Arc<dyn CancelStore>,
        unhealthy_threshold: i32,
    ) -> Self {
        Self {
            work_runs_repo,
            workers_repo,
            project_configs_repo,
            db,
            dispatch_store,
            cancel_store,
            providers_repo,
            unhealthy_threshold,
        }
    }
}
