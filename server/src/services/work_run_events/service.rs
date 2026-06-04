pub mod append_events;
pub mod list_events;
pub mod list_recent;

#[cfg(test)]
mod work_run_events_tests;

use std::sync::Arc;

use sqlx::PgPool;

use crate::services::dispatcher::cancel_store::CancelStore;
use crate::services::work_run_events::errors::WorkRunEventsError;
use crate::services::work_run_events::repository::WorkRunEventsRepository;
use crate::services::work_runs::errors::WorkRunsError;
use crate::services::work_runs::repository::WorkRunsRepository;

#[derive(Clone)]
pub struct WorkRunEventsService {
    pub repo: WorkRunEventsRepository,
    pub work_runs_repo: WorkRunsRepository,
    pub cancel_store: Arc<dyn CancelStore>,
    pub db: PgPool,
}

impl WorkRunEventsService {
    #[must_use]
    pub fn new(
        repo: WorkRunEventsRepository,
        work_runs_repo: WorkRunsRepository,
        cancel_store: Arc<dyn CancelStore>,
        db: PgPool,
    ) -> Self {
        Self {
            repo,
            work_runs_repo,
            cancel_store,
            db,
        }
    }
}

pub(crate) fn map_work_runs_error(e: WorkRunsError) -> WorkRunEventsError {
    match e {
        WorkRunsError::NotFound => WorkRunEventsError::NotFound,
        WorkRunsError::Database(e) => WorkRunEventsError::Database(e),
        WorkRunsError::AlreadyClaimed
        | WorkRunsError::InvalidStatusTransition
        | WorkRunsError::NotOwned
        | WorkRunsError::DeleteRunning
        | WorkRunsError::GithubApp(_)
        | WorkRunsError::Dispatch(_) => WorkRunEventsError::Internal(e.to_string()),
    }
}
