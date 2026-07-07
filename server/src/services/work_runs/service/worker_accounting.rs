use uuid::Uuid;

use crate::models::work_runs::errors::WorkRunsError;
use crate::models::workers::errors::WorkersError;
use crate::services::work_runs::service::WorkRunsService;

impl WorkRunsService {
    pub(crate) async fn release_worker_active_slot(
        &self,
        db: &mut sqlx::PgConnection,
        worker_id: Uuid,
        work_run_id: Uuid,
    ) -> Result<(), WorkRunsError> {
        match self.workers_repo.decrement_active_jobs(db, worker_id).await {
            Ok(()) => Ok(()),
            Err(WorkersError::ActiveJobsInvariant {
                worker_id,
                active_jobs,
            }) => {
                tracing::error!(
                    %worker_id,
                    %work_run_id,
                    active_jobs,
                    "worker active_jobs was already released while finishing work run"
                );
                Ok(())
            }
            Err(err) => Err(err.into()),
        }
    }
}
