use uuid::Uuid;

use crate::models::work_runs::errors::WorkRunsError;
use crate::models::workers::errors::WorkersError;
use crate::services::workers::service::WorkersService;

impl WorkersService {
    pub async fn delete_worker(&self, worker_id: Uuid, team_id: Uuid) -> Result<(), WorkersError> {
        let worker = self.repo.find_by_id(&self.db, worker_id).await?;
        if worker.team_id != team_id {
            return Err(WorkersError::WorkerNotFound);
        }
        self.delete_worker_with_reset(worker_id).await
    }

    pub async fn delete_self(&self, worker_id: Uuid) -> Result<(), WorkersError> {
        self.repo.find_by_id(&self.db, worker_id).await?;
        self.delete_worker_with_reset(worker_id).await
    }

    async fn delete_worker_with_reset(&self, worker_id: Uuid) -> Result<(), WorkersError> {
        let mut tx = self.db.begin().await.map_err(WorkersError::Database)?;

        self.work_runs_repo
            .reset_worker_active_jobs(&mut *tx, worker_id)
            .await
            .map_err(work_runs_to_workers_error)?;

        self.repo.delete(&mut *tx, worker_id).await?;
        tx.commit().await.map_err(WorkersError::Database)
    }
}

fn work_runs_to_workers_error(err: WorkRunsError) -> WorkersError {
    match err {
        WorkRunsError::Database(err) => WorkersError::Database(err),
        _ => WorkersError::WorkerNotFound,
    }
}
