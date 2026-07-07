use sqlx::PgPool;
use uuid::Uuid;

use crate::db::provider_configs::queries::UpdateProviderParams;
use crate::db::provider_configs::IntegrationProvidersRepository;
use crate::models::provider_configs::errors::IntegrationProvidersError;
use crate::models::provider_configs::model::{
    CreateProviderRequest, IntegrationProviderResponse, UpdateProviderRequest,
};

#[derive(Clone)]
pub struct IntegrationProvidersService {
    repo: IntegrationProvidersRepository,
    db: PgPool,
}

impl IntegrationProvidersService {
    pub fn new(repo: IntegrationProvidersRepository, db: PgPool) -> Self {
        Self { repo, db }
    }

    #[must_use]
    pub fn repository(&self) -> IntegrationProvidersRepository {
        self.repo.clone()
    }

    pub async fn list_all(
        &self,
        team_id: Uuid,
    ) -> Result<Vec<IntegrationProviderResponse>, IntegrationProvidersError> {
        let providers = self.repo.list_all(&self.db, team_id).await?;
        Ok(providers.into_iter().map(Into::into).collect())
    }

    pub async fn get_by_id(
        &self,
        id: Uuid,
        team_id: Uuid,
    ) -> Result<IntegrationProviderResponse, IntegrationProvidersError> {
        self.repo
            .find_by_id(&self.db, id, team_id)
            .await
            .map(Into::into)
    }

    pub async fn create(
        &self,
        team_id: Uuid,
        params: CreateProviderRequest,
    ) -> Result<IntegrationProviderResponse, IntegrationProvidersError> {
        self.repo
            .create(&self.db, team_id, &params)
            .await
            .map(Into::into)
    }

    pub async fn update(
        &self,
        id: Uuid,
        team_id: Uuid,
        params: UpdateProviderRequest,
    ) -> Result<IntegrationProviderResponse, IntegrationProvidersError> {
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
            .map(Into::into)
    }

    pub async fn delete(&self, id: Uuid, team_id: Uuid) -> Result<(), IntegrationProvidersError> {
        self.repo.delete(&self.db, id, team_id).await
    }
}
