pub mod work_runs;

#[cfg(test)]
mod work_runs_tests;

use chrono::Duration;
use sqlx::PgPool;

use crate::services::kaneo::client::KaneoClient;
use crate::services::poller::notifier::WorkNotifier;
use crate::services::project_configs::repository::ProjectConfigsRepository;
use crate::services::work_runs::repository::WorkRunsRepository;
use crate::services::workers::repository::WorkersRepository;

pub struct WorkRunsService {
    pub work_runs_repo: WorkRunsRepository,
    pub workers_repo: WorkersRepository,
    pub project_configs_repo: ProjectConfigsRepository,
    pub db: PgPool,
    pub notifier: WorkNotifier,
    pub kaneo: KaneoClient,
    pub stale_threshold: Duration,
}

impl Clone for WorkRunsService {
    fn clone(&self) -> Self {
        Self {
            work_runs_repo: self.work_runs_repo.clone(),
            workers_repo: self.workers_repo.clone(),
            project_configs_repo: self.project_configs_repo.clone(),
            db: self.db.clone(),
            notifier: self.notifier.clone(),
            kaneo: self.kaneo.clone(),
            stale_threshold: self.stale_threshold,
        }
    }
}

impl WorkRunsService {
    pub fn new(
        work_runs_repo: WorkRunsRepository,
        workers_repo: WorkersRepository,
        project_configs_repo: ProjectConfigsRepository,
        db: PgPool,
        notifier: WorkNotifier,
        kaneo: KaneoClient,
        stale_threshold_secs: u64,
    ) -> Self {
        Self {
            work_runs_repo,
            workers_repo,
            project_configs_repo,
            db,
            notifier,
            kaneo,
            stale_threshold: Duration::seconds(stale_threshold_secs as i64),
        }
    }
}
