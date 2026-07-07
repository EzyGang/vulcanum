use uuid::Uuid;

use crate::models::work_runs::errors::WorkRunsError;
use crate::services::work_runs::service::WorkRunsService;

impl WorkRunsService {
    pub async fn poll(&self, worker_id: Uuid) -> Result<Option<Uuid>, WorkRunsError> {
        if let Err(e) = self
            .workers_repo
            .update_last_seen(&self.db, worker_id, chrono::Utc::now())
            .await
        {
            tracing::warn!(error = %e, worker_id = %worker_id, "failed to update last_seen");
        }

        match self.dispatch_store.take_dispatched(worker_id).await {
            Ok(Some(dispatched_id)) => Ok(Some(dispatched_id)),
            Ok(None) => {
                self.work_runs_repo
                    .find_dispatched_for_worker(&self.db, worker_id)
                    .await
            }
            Err(e) => {
                tracing::warn!(
                    error = %e,
                    worker_id = %worker_id,
                    "dispatch store unavailable; falling back to database dispatch visibility"
                );
                self.work_runs_repo
                    .find_dispatched_for_worker(&self.db, worker_id)
                    .await
            }
        }
    }
}
