use sqlx::PgPool;
use uuid::Uuid;

use crate::services::provider_configs::errors::IntegrationProvidersError;
use crate::services::provider_configs::model::{
    CreateProviderRequest, IntegrationProvider, UpdateProviderRequest,
};
use crate::services::provider_configs::repository::queries::UpdateProviderParams;
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

    pub async fn list_all(
        &self,
        team_id: Uuid,
    ) -> Result<Vec<IntegrationProvider>, IntegrationProvidersError> {
        self.repo.list_all(&self.db, team_id).await
    }

    pub async fn get_by_id(
        &self,
        id: Uuid,
        team_id: Uuid,
    ) -> Result<IntegrationProvider, IntegrationProvidersError> {
        self.repo.find_by_id(&self.db, id, team_id).await
    }

    pub async fn create(
        &self,
        team_id: Uuid,
        params: CreateProviderRequest,
    ) -> Result<IntegrationProvider, IntegrationProvidersError> {
        self.repo.create(&self.db, team_id, &params).await
    }

    pub async fn update(
        &self,
        id: Uuid,
        team_id: Uuid,
        params: UpdateProviderRequest,
    ) -> Result<IntegrationProvider, IntegrationProvidersError> {
        self.repo
            .update(
                &self.db,
                UpdateProviderParams {
                    id,
                    team_id,
                    name: params.name.as_deref(),
                    provider_type: params.provider_type,
                    instance_url: params.instance_url.as_deref(),
                    api_key: params.api_key.as_deref(),
                },
            )
            .await
    }

    pub async fn delete(&self, id: Uuid, team_id: Uuid) -> Result<(), IntegrationProvidersError> {
        self.repo.delete(&self.db, id, team_id).await
    }
}
