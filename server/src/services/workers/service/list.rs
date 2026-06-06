use crate::services::workers::errors::WorkersError;
use crate::services::workers::model::WorkerResponse;
use crate::services::workers::service::WorkersService;

impl WorkersService {
    pub async fn list_all(&self) -> Result<Vec<WorkerResponse>, WorkersError> {
        let workers = self.repo.list_all(&self.db).await?;
        Ok(workers.into_iter().map(WorkerResponse::from).collect())
    }
}
