use sqlx::PgPool;
use uuid::Uuid;

use crate::services::model_providers::catalog::ModelCatalogClient;
use crate::services::model_providers::errors::ModelProvidersError;
use crate::services::model_providers::model::{
    CatalogResponse, CreateModelProviderRequest, ModelProviderConfig, UpdateModelProviderRequest,
};
use crate::services::model_providers::repository::ModelProvidersRepository;

#[derive(Clone)]
pub struct ModelProvidersService {
    pub repo: ModelProvidersRepository,
    pub db: PgPool,
    pub catalog: ModelCatalogClient,
}

impl ModelProvidersService {
    pub fn new(repo: ModelProvidersRepository, db: PgPool, catalog: ModelCatalogClient) -> Self {
        Self { repo, db, catalog }
    }

    pub async fn catalog(&self) -> Result<CatalogResponse, ModelProvidersError> {
        self.catalog.catalog().await
    }

    pub async fn list_all(
        &self,
        team_id: Uuid,
    ) -> Result<Vec<ModelProviderConfig>, ModelProvidersError> {
        self.repo.list_all(&self.db, team_id).await
    }

    pub async fn create(
        &self,
        team_id: Uuid,
        params: CreateModelProviderRequest,
    ) -> Result<ModelProviderConfig, ModelProvidersError> {
        self.catalog.validate_provider(&params.provider_key).await?;
        self.repo.create(&self.db, team_id, &params).await
    }

    pub async fn update(
        &self,
        id: Uuid,
        team_id: Uuid,
        params: UpdateModelProviderRequest,
    ) -> Result<ModelProviderConfig, ModelProvidersError> {
        self.repo.update(&self.db, id, team_id, &params).await
    }

    pub async fn delete(&self, id: Uuid, team_id: Uuid) -> Result<(), ModelProvidersError> {
        self.repo.delete(&self.db, id, team_id).await
    }

    pub async fn validate_model_selection(
        &self,
        team_id: Uuid,
        provider_key: Option<&str>,
        model_id: Option<&str>,
    ) -> Result<(), ModelProvidersError> {
        let Some(provider_key) = provider_key.filter(|value| !value.is_empty()) else {
            return Ok(());
        };
        let Some(model_id) = model_id.filter(|value| !value.is_empty()) else {
            return Ok(());
        };

        self.repo
            .find_by_provider_key(&self.db, team_id, provider_key)
            .await?;
        self.catalog.validate_model(provider_key, model_id).await
    }
}
