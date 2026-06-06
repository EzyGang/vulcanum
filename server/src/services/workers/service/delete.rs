use uuid::Uuid;

use crate::services::workers::errors::WorkersError;
use crate::services::workers::service::WorkersService;

impl WorkersService {
    pub async fn delete_worker(&self, worker_id: Uuid) -> Result<(), WorkersError> {
        self.repo.delete(&self.db, worker_id).await
    }
}
