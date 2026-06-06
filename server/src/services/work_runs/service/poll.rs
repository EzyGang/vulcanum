use uuid::Uuid;

use crate::services::work_runs::errors::WorkRunsError;
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

        let dispatched_id = self.dispatch_store.take_dispatched(worker_id).await?;

        Ok(dispatched_id)
    }
}
