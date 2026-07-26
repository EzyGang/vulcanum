mod catalog_operations;
mod device_flow;
mod persistence;
mod validation;

use std::collections::HashSet;
use std::sync::Arc;

use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;
use vulcanum_shared::api::wire::AgentBackend;

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
