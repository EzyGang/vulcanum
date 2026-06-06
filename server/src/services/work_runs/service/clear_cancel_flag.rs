use uuid::Uuid;

use crate::services::work_runs::service::WorkRunsService;

impl WorkRunsService {
    pub(crate) async fn clear_cancel_flag(&self, work_run_id: Uuid) {
        if let Err(e) = self.cancel_store.take_cancel(work_run_id).await {
            tracing::warn!(
                error = %e,
                work_run_id = %work_run_id,
                "failed to clear cancel flag on terminal status"
            );
        }
    }
}
