pub mod work_runs;

#[cfg(test)]
mod work_runs_tests;

use chrono::Duration;
use sqlx::PgPool;

use crate::services::poller::notifier::WorkNotifier;
use crate::services::work_runs::repository::WorkRunsRepository;
use crate::services::workers::repository::WorkersRepository;

pub struct WorkRunsService {
    pub work_runs_repo: WorkRunsRepository,
    pub workers_repo: WorkersRepository,
    pub db: PgPool,
    pub notifier: WorkNotifier,
    pub stale_threshold: Duration,
}

impl Clone for WorkRunsService {
    fn clone(&self) -> Self {
        Self {
            work_runs_repo: self.work_runs_repo.clone(),
            workers_repo: self.workers_repo.clone(),
            db: self.db.clone(),
            notifier: self.notifier.clone(),
            stale_threshold: self.stale_threshold,
        }
    }
}

impl WorkRunsService {
    pub fn new(
        work_runs_repo: WorkRunsRepository,
        workers_repo: WorkersRepository,
        db: PgPool,
        notifier: WorkNotifier,
        stale_threshold_secs: u64,
    ) -> Self {
        Self {
            work_runs_repo,
            workers_repo,
            db,
            notifier,
            stale_threshold: Duration::seconds(stale_threshold_secs as i64),
        }
    }
}
