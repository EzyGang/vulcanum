use crate::services::workers::errors::WorkersError;
use crate::services::workers::model::WorkerResponse;
use crate::services::workers::service::WorkersService;

impl WorkersService {
    pub async fn list_all(&self, team_id: uuid::Uuid) -> Result<Vec<WorkerResponse>, WorkersError> {
        let workers = self.repo.list_all(&self.db, team_id).await?;
        Ok(workers.into_iter().map(WorkerResponse::from).collect())
    }
}
