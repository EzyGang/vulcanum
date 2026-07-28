use uuid::Uuid;

use crate::models::workers::errors::WorkersError;
use crate::models::workers::model::{RenameWorkerRequest, WorkerResponse};
use crate::services::workers::service::WorkersService;

impl WorkersService {
    pub async fn rename_worker(
        &self,
        worker_id: Uuid,
        team_id: Uuid,
        req: RenameWorkerRequest,
    ) -> Result<WorkerResponse, WorkersError> {
        let existing = self.repo.find_by_id(&self.db, worker_id).await?;
        if existing.team_id != team_id {
            return Err(WorkersError::WorkerNotFound);
        }

        let worker = self.repo.rename(&self.db, worker_id, &req.name).await?;
        Ok(WorkerResponse::from(worker))
    }
}
