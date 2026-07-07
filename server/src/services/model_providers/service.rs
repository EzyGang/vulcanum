use std::collections::HashSet;
use std::sync::Arc;

use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;
use vulcanum_shared::api_types::AgentBackend;

use crate::db::model_providers::ModelProvidersRepository;
use crate::models::model_providers::errors::ModelProvidersError;
use crate::models::model_providers::model::{
    CatalogResponse, CreateModelProviderRequest, ModelProviderAuthType, ModelProviderConfig,
    ModelProviderResponse, PollDeviceFlowResponse, StartDeviceFlowRequest, StartDeviceFlowResponse,
    UpdateModelProviderRequest,
};
use crate::services::model_providers::auth::credentials::{
    api_key_credential_fields, encrypted_api_key_credentials, encrypted_oauth_credentials,
    parse_auth, to_response, ParsedAuth, OPENAI_PROVIDER_KEY,
};
use crate::services::model_providers::auth::device_flow::{
    DeviceAuthProvider, DeviceFlowStore, DevicePoll, PendingDeviceFlow,
};
use crate::services::model_providers::auth::encryption::SecretCipher;
use crate::services::model_providers::catalog::{
    is_codex_compatible_openai_model, ModelCatalogClient,
};
use crate::services::model_providers::renderer::{
    render_agent_config, ModelSelection, RenderedAgentConfig,
};

#[derive(Clone)]
pub struct ModelProvidersService {
    repo: ModelProvidersRepository,
    db: PgPool,
    catalog: ModelCatalogClient,
    cipher: SecretCipher,
    device_flow_store: Arc<dyn DeviceFlowStore>,
    device_auth_provider: Arc<dyn DeviceAuthProvider>,
}

impl ModelProvidersService {
    pub fn new(
        repo: ModelProvidersRepository,
        db: PgPool,
        catalog: ModelCatalogClient,
        cipher: SecretCipher,
        device_flow_store: Arc<dyn DeviceFlowStore>,
        device_auth_provider: Arc<dyn DeviceAuthProvider>,
    ) -> Self {
        Self {
            repo,
            db,
            catalog,
            cipher,
            device_flow_store,
            device_auth_provider,
        }
    }

    pub async fn catalog(&self) -> Result<CatalogResponse, ModelProvidersError> {
        self.catalog.catalog().await
    }

    pub async fn list_all(
        &self,
        team_id: Uuid,
    ) -> Result<Vec<ModelProviderResponse>, ModelProvidersError> {
        let providers = self.repo.list_all(&self.db, team_id).await?;
        let mut responses = Vec::with_capacity(providers.len());
        for provider in providers {
            let response = to_response(provider, &self.cipher)?;
            self.catalog
                .validate_credential_fields(&response.provider_key, &response.credential_fields)
                .await?;
            responses.push(response);
        }
        Ok(responses)
    }

    pub async fn render_agent_config_for_team(
        &self,
        team_id: Uuid,
        backend: AgentBackend,
        selection: ModelSelection<'_>,
    ) -> Result<RenderedAgentConfig, ModelProvidersError> {
        let selected_keys = selected_provider_keys(&selection);
        let mut providers = self.repo.list_all(&self.db, team_id).await?;
        if !selected_keys.is_empty() {
            providers.retain(|provider| selected_keys.contains(provider.provider_key.as_str()));
        }
        for provider in &mut providers {
            self.validate_stored_api_key_credentials(provider).await?;
            self.refresh_provider_if_needed(provider).await?;
        }
        render_agent_config(backend, &providers, &self.cipher, selection)
    }

    pub async fn create(
        &self,
        team_id: Uuid,
        params: CreateModelProviderRequest,
    ) -> Result<ModelProviderResponse, ModelProvidersError> {
        self.catalog.validate_provider(&params.provider_key).await?;
        let mut stored = params.clone();
        match params.auth_type {
            ModelProviderAuthType::ApiKey => {
                self.validate_api_key_credentials(&params.provider_key, &params.credentials)
                    .await?;
                stored.credentials =
                    encrypted_api_key_credentials(&params.credentials, &self.cipher)?;
            }
            ModelProviderAuthType::None => {
                reject_credentials_for_none_auth(&params.credentials)?;
                stored.credentials = serde_json::Value::Null;
            }
            ModelProviderAuthType::DeviceOauth => {
                return Err(ModelProvidersError::InvalidAuthConfig(
                    "device OAuth must be connected with the device flow endpoint".to_owned(),
                ));
            }
        }
        let provider = self.repo.create(&self.db, team_id, &stored).await?;
        to_response(provider, &self.cipher)
    }

    pub async fn update(
        &self,
        id: Uuid,
        team_id: Uuid,
        params: UpdateModelProviderRequest,
    ) -> Result<ModelProviderResponse, ModelProvidersError> {
        let mut stored = params.clone();
        match params.auth_type {
            Some(ModelProviderAuthType::DeviceOauth) => {
                return Err(ModelProvidersError::InvalidAuthConfig(
                    "device OAuth must be connected with the device flow endpoint".to_owned(),
                ));
            }
            Some(ModelProviderAuthType::None) => {
                reject_optional_credentials_for_none_auth(params.credentials.as_ref())?;
                stored.credentials = Some(serde_json::Value::Null);
            }
            Some(ModelProviderAuthType::ApiKey) => {
                let credentials = params.credentials.as_ref().ok_or_else(|| {
                    ModelProvidersError::InvalidAuthConfig(
                        "api key auth requires credentials".to_owned(),
                    )
                })?;
                let existing = self.repo.find_by_id(&self.db, id, team_id).await?;
                self.validate_api_key_credentials(&existing.provider_key, credentials)
                    .await?;
                stored.credentials =
                    Some(encrypted_api_key_credentials(credentials, &self.cipher)?);
            }
            None => {
                if let Some(credentials) = params.credentials.as_ref() {
                    let existing = self.repo.find_by_id(&self.db, id, team_id).await?;
                    self.validate_api_key_credentials(&existing.provider_key, credentials)
                        .await?;
                    stored.credentials =
                        Some(encrypted_api_key_credentials(credentials, &self.cipher)?);
                }
            }
        }
        let provider = self.repo.update(&self.db, id, team_id, &stored).await?;
        to_response(provider, &self.cipher)
    }

    pub async fn start_device_flow(
        &self,
        team_id: Uuid,
        user_id: Option<&str>,
        params: StartDeviceFlowRequest,
    ) -> Result<StartDeviceFlowResponse, ModelProvidersError> {
        if params.provider_key != self.device_auth_provider.model_provider_key()
            || params.device_provider != self.device_auth_provider.provider_id()
        {
            return Err(ModelProvidersError::InvalidAuthConfig(
                "unsupported device flow provider".to_owned(),
            ));
        }
        let device_start = self.device_auth_provider.start().await?;
        let now = Utc::now();
        let expires_at = now + chrono::Duration::minutes(10);
        let next_poll_at = now + chrono::Duration::seconds(device_start.interval_seconds);
        let attempt_id = Uuid::new_v4();

        self.device_flow_store
            .insert(PendingDeviceFlow {
                attempt_id,
                team_id,
                user_id: user_id.map(str::to_owned),
                provider_key: params.provider_key,
                device_provider: params.device_provider,
                display_name: match params.display_name.is_empty() {
                    true => "ChatGPT Plus".to_owned(),
                    false => params.display_name,
                },
                device_auth_id: device_start.device_auth_id,
                user_code: device_start.user_code.clone(),
                verification_uri: device_start.verification_uri.clone(),
                interval_seconds: device_start.interval_seconds,
                next_poll_at,
                expires_at,
            })
            .await?;

        Ok(StartDeviceFlowResponse {
            attempt_id,
            verification_uri: device_start.verification_uri,
            user_code: device_start.user_code,
            interval_seconds: device_start.interval_seconds,
            expires_at,
        })
    }

    pub async fn poll_device_flow(
        &self,
        team_id: Uuid,
        user_id: Option<&str>,
        attempt_id: Uuid,
    ) -> Result<PollDeviceFlowResponse, ModelProvidersError> {
        let Some(pending) = self.device_flow_store.get(attempt_id).await? else {
            return Err(ModelProvidersError::DeviceFlowExpired);
        };
        if pending.team_id != team_id || pending.user_id.as_deref() != user_id {
            return Err(ModelProvidersError::DeviceFlowExpired);
        }
        let now = Utc::now();
        if pending.expires_at <= now {
            self.device_flow_store.consume(attempt_id).await?;
            return Err(ModelProvidersError::DeviceFlowExpired);
        }
        if pending.next_poll_at > now {
            return Ok(PollDeviceFlowResponse::Pending {
                next_poll_at: pending.next_poll_at,
            });
        }

        match self.device_auth_provider.poll(&pending).await? {
            DevicePoll::Pending => {
                let next_poll_at = now + chrono::Duration::seconds(pending.interval_seconds);
                self.device_flow_store
                    .update_next_poll(attempt_id, next_poll_at)
                    .await?;
                Ok(PollDeviceFlowResponse::Pending { next_poll_at })
            }
            DevicePoll::Complete(credential) => {
                self.device_flow_store.consume(attempt_id).await?;
                let credentials = encrypted_oauth_credentials(&credential, &self.cipher)?;
                let provider = self
                    .repo
                    .upsert_by_provider_key(
                        &self.db,
                        team_id,
                        &pending.provider_key,
                        &pending.display_name,
                        &credentials,
                    )
                    .await?;
                let response = to_response(provider, &self.cipher)?;
                self.catalog
                    .validate_credential_fields(&response.provider_key, &response.credential_fields)
                    .await?;
                Ok(PollDeviceFlowResponse::Connected {
                    provider: Box::new(response),
                })
            }
        }
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

        let provider = self
            .repo
            .find_by_provider_key(&self.db, team_id, provider_key)
            .await?;
        match parse_auth(&provider.credentials, &self.cipher)? {
            ParsedAuth::DeviceOAuth(_) if provider_key == OPENAI_PROVIDER_KEY => {
                return match is_codex_compatible_openai_model(model_id) {
                    true => Ok(()),
                    false => Err(ModelProvidersError::UnknownModel {
                        provider_key: provider_key.to_owned(),
                        model_id: model_id.to_owned(),
                    }),
                };
            }
            _ => (),
        }
        self.catalog.validate_model(provider_key, model_id).await
    }

    async fn refresh_provider_if_needed(
        &self,
        provider: &mut ModelProviderConfig,
    ) -> Result<(), ModelProvidersError> {
        let ParsedAuth::DeviceOAuth(credential) = parse_auth(&provider.credentials, &self.cipher)?
        else {
            return Ok(());
        };
        if provider.provider_key != self.device_auth_provider.model_provider_key()
            || credential.provider != self.device_auth_provider.provider_id()
        {
            return Err(ModelProvidersError::InvalidAuthConfig(format!(
                "unsupported device OAuth provider {}",
                credential.provider
            )));
        }
        if !credential.should_refresh(Utc::now()) {
            return Ok(());
        }

        let refreshed = self.device_auth_provider.refresh(&credential).await?;
        let credentials = encrypted_oauth_credentials(&refreshed, &self.cipher)?;
        let updated = self
            .repo
            .update_credentials(&self.db, provider.id, provider.team_id, &credentials)
            .await?;
        *provider = updated;
        Ok(())
    }

    async fn validate_api_key_credentials(
        &self,
        provider_key: &str,
        credentials: &serde_json::Value,
    ) -> Result<(), ModelProvidersError> {
        let fields = api_key_credential_fields(credentials)?;
        if fields.is_empty() {
            return Err(ModelProvidersError::InvalidAuthConfig(
                "api key auth requires at least one credential".to_owned(),
            ));
        }
        self.catalog
            .validate_credential_fields(provider_key, &fields)
            .await
    }

    async fn validate_stored_api_key_credentials(
        &self,
        provider: &ModelProviderConfig,
    ) -> Result<(), ModelProvidersError> {
        let ParsedAuth::ApiKey(credentials) = parse_auth(&provider.credentials, &self.cipher)?
        else {
            return Ok(());
        };
        let mut fields = credentials.keys().cloned().collect::<Vec<String>>();
        fields.sort();
        self.catalog
            .validate_credential_fields(&provider.provider_key, &fields)
            .await
    }
}

fn selected_provider_keys<'a>(selection: &ModelSelection<'a>) -> HashSet<&'a str> {
    [selection.primary_provider_key, selection.small_provider_key]
        .into_iter()
        .flatten()
        .filter(|provider_key| !provider_key.is_empty())
        .collect()
}

fn reject_optional_credentials_for_none_auth(
    credentials: Option<&serde_json::Value>,
) -> Result<(), ModelProvidersError> {
    match credentials {
        Some(value) => reject_credentials_for_none_auth(value),
        None => Ok(()),
    }
}

fn reject_credentials_for_none_auth(
    credentials: &serde_json::Value,
) -> Result<(), ModelProvidersError> {
    if credentials.is_null()
        || credentials
            .as_object()
            .map(|object| object.is_empty())
            .unwrap_or(false)
    {
        return Ok(());
    }
    Err(ModelProvidersError::InvalidAuthConfig(
        "none auth cannot include credentials".to_owned(),
    ))
}
