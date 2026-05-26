pub mod work_runs;

#[cfg(test)]
mod work_runs_tests;

use std::sync::Arc;

use sqlx::PgPool;

use crate::services::dispatcher::flag_store::DispatchStore;
use crate::services::kaneo::client::KaneoClient;
use crate::services::project_configs::repository::ProjectConfigsRepository;
use crate::services::work_runs::repository::WorkRunsRepository;
use crate::services::workers::repository::WorkersRepository;

pub struct WorkRunsService {
    pub work_runs_repo: WorkRunsRepository,
    pub workers_repo: WorkersRepository,
    pub project_configs_repo: ProjectConfigsRepository,
    pub db: PgPool,
    pub dispatch_store: Arc<dyn DispatchStore>,
    pub kaneo: KaneoClient,
}

impl Clone for WorkRunsService {
    fn clone(&self) -> Self {
        Self {
            work_runs_repo: self.work_runs_repo.clone(),
            workers_repo: self.workers_repo.clone(),
            project_configs_repo: self.project_configs_repo.clone(),
            db: self.db.clone(),
            dispatch_store: self.dispatch_store.clone(),
            kaneo: self.kaneo.clone(),
        }
    }
}

impl WorkRunsService {
    pub fn new(
        work_runs_repo: WorkRunsRepository,
        workers_repo: WorkersRepository,
        project_configs_repo: ProjectConfigsRepository,
        db: PgPool,
        dispatch_store: Arc<dyn DispatchStore>,
        kaneo: KaneoClient,
    ) -> Self {
        Self {
            work_runs_repo,
            workers_repo,
            project_configs_repo,
            db,
            dispatch_store,
            kaneo,
        }
    }
}
