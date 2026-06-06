use uuid::Uuid;

use crate::services::workers::errors::WorkersError;
use crate::services::workers::model::UpdateWorkerStatusRequest;
use crate::services::workers::model::{WorkerResponse, WorkerStatus, WorkerStatusOverride};
use crate::services::workers::service::WorkersService;

impl WorkersService {
    pub async fn set_worker_status(
        &self,
        worker_id: Uuid,
        req: UpdateWorkerStatusRequest,
    ) -> Result<WorkerResponse, WorkersError> {
        match req.status {
            WorkerStatusOverride::Unhealthy => {
                let mut tx = self.db.begin().await.map_err(WorkersError::Database)?;

                self.repo
                    .set_status(&mut *tx, worker_id, WorkerStatus::Unhealthy)
                    .await?;

                let reset_count = self
                    .work_runs_repo
                    .reset_worker_active_jobs(&mut *tx, worker_id)
                    .await
                    .map_err(|e| match e {
                        crate::services::work_runs::errors::WorkRunsError::Database(e) => {
                            WorkersError::Database(e)
                        }
                        _ => WorkersError::WorkerNotFound,
                    })?;

                self.repo
                    .reset_active_jobs_only(&mut *tx, worker_id)
                    .await?;

                tx.commit().await.map_err(WorkersError::Database)?;

                tracing::info!(
                    worker_id = %worker_id,
                    reset_jobs = reset_count,
                    "worker marked unhealthy, active jobs reset"
                );
            }
            WorkerStatusOverride::Idle => {
                self.repo
                    .set_status_and_reset(&self.db, worker_id, WorkerStatus::Idle)
                    .await?;
            }
        }

        let worker = self.repo.find_by_id(&self.db, worker_id).await?;
        Ok(WorkerResponse::from(worker))
    }
}
