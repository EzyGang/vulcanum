pub mod append_events;
pub mod list_events;
pub mod list_recent;

#[cfg(test)]
mod work_run_events_tests;

use std::sync::Arc;

use sqlx::PgPool;

use crate::db::work_run_events::WorkRunEventsRepository;
use crate::db::work_runs::WorkRunsRepository;
use crate::models::work_run_events::errors::WorkRunEventsError;
use crate::models::work_runs::errors::WorkRunsError;
use crate::services::dispatcher::cancel_store::CancelStore;

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
        | WorkRunsError::ModelProvider(_)
        | WorkRunsError::ProjectConfig(_)
        | WorkRunsError::Team(_)
        | WorkRunsError::Worker(_)
        | WorkRunsError::Dispatch(_) => WorkRunEventsError::Internal(e.to_string()),
    }
}
