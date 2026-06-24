pub(crate) mod chatgpt_oauth;
pub(crate) mod oauth_client;
mod selection;

use std::sync::Arc;

use sqlx::PgPool;
use uuid::Uuid;

use crate::services::model_providers::catalog::ModelCatalogClient;
use crate::services::model_providers::crypto::CredentialCipher;
use crate::services::model_providers::errors::ModelProvidersError;
use crate::services::model_providers::model::{
    CatalogResponse, CreateModelProviderRequest, ModelProviderConfig, UpdateModelProviderRequest,
    AUTH_TYPE_API_KEY, AUTH_TYPE_CHATGPT_OAUTH,
};
use crate::services::model_providers::repository::ModelProvidersRepository;
use crate::services::model_providers::service::oauth_client::{
    ChatGptOAuthClient, OpenAiChatGptOAuthClient,
};

#[derive(Debug, Clone)]
pub struct ModelProvidersService {
    pub repo: ModelProvidersRepository,
    pub db: PgPool,
    pub catalog: ModelCatalogClient,
    cipher: CredentialCipher,
    oauth_client: Arc<dyn ChatGptOAuthClient>,
}

#[derive(Debug, Default)]
pub struct SelectedModelProviderAuth {
    pub providers: Vec<ModelProviderConfig>,
    pub opencode_auth_content: Option<String>,
}

impl ModelProvidersService {
    #[must_use]
    pub fn new(
        repo: ModelProvidersRepository,
        db: PgPool,
        catalog: ModelCatalogClient,
        encryption_secret: &str,
    ) -> Self {
        Self {
            repo,
            db,
            catalog,
            cipher: CredentialCipher::new(encryption_secret),
            oauth_client: Arc::new(OpenAiChatGptOAuthClient::default()),
        }
    }

    #[cfg(test)]
    pub(crate) fn new_for_tests(
        repo: ModelProvidersRepository,
        db: PgPool,
        catalog: ModelCatalogClient,
        encryption_secret: &str,
        oauth_client: Arc<dyn ChatGptOAuthClient>,
    ) -> Self {
        Self {
            repo,
            db,
            catalog,
            cipher: CredentialCipher::new(encryption_secret),
            oauth_client,
        }
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
        self.validate_auth_type(&params.auth_type)?;
        if params.auth_type == AUTH_TYPE_CHATGPT_OAUTH {
            return Err(ModelProvidersError::InvalidSelection(
                "ChatGPT OAuth providers must be created via device login".to_owned(),
            ));
        }
        self.catalog.validate_provider(&params.provider_key).await?;
        self.repo.create(&self.db, team_id, &params).await
    }

    pub async fn update(
        &self,
        id: Uuid,
        team_id: Uuid,
        mut params: UpdateModelProviderRequest,
    ) -> Result<ModelProviderConfig, ModelProvidersError> {
        let provider = self.repo.find_by_id(&self.db, id, team_id).await?;
        if provider.auth_type == AUTH_TYPE_CHATGPT_OAUTH {
            match params.credentials.as_ref() {
                Some(credentials)
                    if credentials
                        .as_object()
                        .is_some_and(|object| object.is_empty()) =>
                {
                    params.credentials = None;
                }
                Some(_) => {
                    return Err(ModelProvidersError::InvalidSelection(
                        "ChatGPT OAuth credentials can only be updated by reconnecting".to_owned(),
                    ));
                }
                None => (),
            }
        }
        self.repo.update(&self.db, id, team_id, &params).await
    }

    pub async fn delete(&self, id: Uuid, team_id: Uuid) -> Result<(), ModelProvidersError> {
        self.repo.delete(&self.db, id, team_id).await
    }

    pub async fn validate_model_selection(
        &self,
        team_id: Uuid,
        provider_config_id: Option<Uuid>,
        model_id: Option<&str>,
    ) -> Result<(), ModelProvidersError> {
        let Some(provider_config_id) = provider_config_id else {
            return Ok(());
        };

        let provider = self
            .repo
            .find_by_id(&self.db, provider_config_id, team_id)
            .await?;
        let Some(model_id) = model_id.filter(|value| !value.is_empty()) else {
            return Ok(());
        };
        self.catalog
            .validate_model(&provider.provider_key, model_id)
            .await
    }

    pub async fn provider_config_id_for_key(
        &self,
        team_id: Uuid,
        provider_key: &str,
    ) -> Result<Uuid, ModelProvidersError> {
        self.repo
            .find_by_provider_key(&self.db, team_id, provider_key)
            .await
            .map(|provider| provider.id)
    }

    fn validate_auth_type(&self, auth_type: &str) -> Result<(), ModelProvidersError> {
        match auth_type {
            AUTH_TYPE_API_KEY | AUTH_TYPE_CHATGPT_OAUTH => Ok(()),
            other => Err(ModelProvidersError::InvalidAuthType(other.to_owned())),
        }
    }
}
