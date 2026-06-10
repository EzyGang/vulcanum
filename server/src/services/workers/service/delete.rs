use uuid::Uuid;

use crate::services::workers::errors::WorkersError;
use crate::services::workers::service::WorkersService;

impl WorkersService {
    pub async fn delete_worker(&self, worker_id: Uuid, team_id: Uuid) -> Result<(), WorkersError> {
        let worker = self.repo.find_by_id(&self.db, worker_id).await?;
        if worker.team_id != team_id {
            return Err(WorkersError::WorkerNotFound);
        }
        self.repo.delete(&self.db, worker_id).await
    }
}
