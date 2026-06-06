use sqlx::PgPool;
use uuid::Uuid;

use crate::services::provider_configs::errors::IntegrationProvidersError;
use crate::services::provider_configs::model::{
    CreateProviderRequest, IntegrationProvider, UpdateProviderRequest,
};
use crate::services::provider_configs::repository::IntegrationProvidersRepository;

#[derive(Clone)]
pub struct IntegrationProvidersService {
    pub repo: IntegrationProvidersRepository,
    pub db: PgPool,
}

impl IntegrationProvidersService {
    pub fn new(repo: IntegrationProvidersRepository, db: PgPool) -> Self {
        Self { repo, db }
    }

    pub async fn list_all(&self) -> Result<Vec<IntegrationProvider>, IntegrationProvidersError> {
        self.repo.list_all(&self.db).await
    }

    pub async fn get_by_id(
        &self,
        id: Uuid,
    ) -> Result<IntegrationProvider, IntegrationProvidersError> {
        self.repo.find_by_id(&self.db, id).await
    }

    pub async fn create(
        &self,
        params: CreateProviderRequest,
    ) -> Result<IntegrationProvider, IntegrationProvidersError> {
        self.repo.create(&self.db, &params).await
    }

    pub async fn update(
        &self,
        id: Uuid,
        params: UpdateProviderRequest,
    ) -> Result<IntegrationProvider, IntegrationProvidersError> {
        self.repo
            .update(
                &self.db,
                id,
                params.name.as_deref(),
                params.provider_type,
                params.instance_url.as_deref(),
                params.api_key.as_deref(),
            )
            .await
    }

    pub async fn delete(&self, id: Uuid) -> Result<(), IntegrationProvidersError> {
        self.repo.delete(&self.db, id).await
    }
}
