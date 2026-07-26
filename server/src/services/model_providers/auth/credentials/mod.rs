mod storage;

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::Deserialize;

use self::storage::{
    decode_legacy_snake_case_env_key, encrypted_secret_value, encrypted_secrets_value,
    parse_api_key_credentials, parse_legacy_api_key_auth, parse_stored_auth,
    validate_credential_env_key,
};
use crate::models::model_providers::errors::ModelProvidersError;
use crate::models::model_providers::model::{
    ModelProviderAuthType, ModelProviderConfig, ModelProviderOAuthStatus, ModelProviderResponse,
};
use crate::services::model_providers::auth::encryption::{EncryptedSecret, SecretCipher};

pub const OPENAI_PROVIDER_KEY: &str = "openai";
pub const OPENAI_CHATGPT_PROVIDER_ID: &str = "openai_chatgpt";
const DANGEROUS_ENV_KEYS: &[&str] = &[
    "BASH_ENV",
    "ENV",
    "GIT_CONFIG_GLOBAL",
    "GIT_CONFIG_SYSTEM",
    "GIT_SSH",
    "GIT_SSH_COMMAND",
    "HOME",
    "IFS",
    "LD_LIBRARY_PATH",
    "LD_PRELOAD",
    "NODE_OPTIONS",
    "PATH",
    "PYTHONPATH",
    "RUSTC_WRAPPER",
    "RUSTFLAGS",
    "SHELL",
];
const DANGEROUS_ENV_PREFIXES: &[&str] = &["DYLD_", "LD_"];

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OAuthCredential {
    pub provider: String,
    pub account_id: Option<String>,
    pub email: Option<String>,
    pub expires: i64,
    pub refresh: String,
    pub access: String,
}

impl OAuthCredential {
    #[must_use]
    pub fn should_refresh(&self, now: DateTime<Utc>) -> bool {
        self.expires <= (now + chrono::Duration::hours(1)).timestamp_millis()
    }
}

#[derive(Debug)]
pub enum ParsedAuth {
    ApiKey(HashMap<String, String>),
    DeviceOAuth(OAuthCredential),
    None,
}

#[derive(Debug, Deserialize)]
struct StoredAuth {
    schema_version: i32,
    auth_type: ModelProviderAuthType,
    #[serde(default)]
    api_key: Option<StoredApiKeyAuth>,
    #[serde(default)]
    device_oauth: Option<StoredOAuthAuth>,
}

#[derive(Debug, Deserialize)]
struct StoredApiKeyAuth {
    fields: Vec<String>,
    secrets: HashMap<String, EncryptedSecret>,
}

#[derive(Debug, Deserialize)]
struct StoredOAuthAuth {
    provider: String,
    account_id: Option<String>,
    email: Option<String>,
    expires: i64,
    refresh: EncryptedSecret,
    access: EncryptedSecret,
}

#[must_use = "credential fields must be validated against the provider catalog"]
pub fn api_key_credential_fields(
    credentials: &serde_json::Value,
) -> Result<Vec<String>, ModelProvidersError> {
    let api_keys = parse_api_key_credentials(credentials)?;
    let mut fields = api_keys.keys().cloned().collect::<Vec<String>>();
    fields.sort();
    Ok(fields)
}

#[must_use = "encrypted credentials must be persisted"]
pub fn encrypted_api_key_credentials(
    credentials: &serde_json::Value,
    cipher: &SecretCipher,
) -> Result<serde_json::Value, ModelProvidersError> {
    let api_keys = parse_api_key_credentials(credentials)?;
    let mut fields = api_keys.keys().cloned().collect::<Vec<String>>();
    let mut secrets: HashMap<String, EncryptedSecret> = HashMap::new();

    for (key, secret) in api_keys {
        secrets.insert(key, cipher.encrypt(&secret)?);
    }
    fields.sort();

    Ok(serde_json::json!({
        "schema_version": 1,
        "auth_type": ModelProviderAuthType::ApiKey,
        "api_key": {
            "fields": fields,
            "secrets": encrypted_secrets_value(secrets),
        },
        "device_oauth": null,
    }))
}

#[must_use = "encrypted credentials must be persisted"]
pub fn encrypted_oauth_credentials(
    credential: &OAuthCredential,
    cipher: &SecretCipher,
) -> Result<serde_json::Value, ModelProvidersError> {
    let refresh = cipher.encrypt(&credential.refresh)?;
    let access = cipher.encrypt(&credential.access)?;
    Ok(serde_json::json!({
        "schema_version": 1,
        "auth_type": ModelProviderAuthType::DeviceOauth,
        "api_key": null,
        "device_oauth": {
            "provider": credential.provider,
            "account_id": credential.account_id,
            "email": credential.email,
            "expires": credential.expires,
            "refresh": encrypted_secret_value(&refresh),
            "access": encrypted_secret_value(&access),
        },
    }))
}

#[must_use = "parsed auth must be handled"]
pub fn parse_auth(
    credentials: &serde_json::Value,
    cipher: &SecretCipher,
) -> Result<ParsedAuth, ModelProvidersError> {
    if credentials.is_null() {
        return Ok(ParsedAuth::None);
    }

    match serde_json::from_value::<StoredAuth>(credentials.clone()) {
        Ok(stored) => parse_stored_auth(stored, cipher),
        Err(_) => parse_legacy_api_key_auth(credentials),
    }
}

#[must_use = "safe response must be returned"]
pub fn to_response(
    provider: ModelProviderConfig,
    cipher: &SecretCipher,
) -> Result<ModelProviderResponse, ModelProvidersError> {
    let (auth_type, credential_fields, oauth) = match parse_auth(&provider.credentials, cipher)? {
        ParsedAuth::ApiKey(credentials) => {
            let mut fields = credentials.keys().cloned().collect::<Vec<String>>();
            fields.sort();
            (ModelProviderAuthType::ApiKey, fields, None)
        }
        ParsedAuth::DeviceOAuth(credential) => (
            ModelProviderAuthType::DeviceOauth,
            Vec::new(),
            Some(ModelProviderOAuthStatus {
                provider: credential.provider,
                account_id: credential.account_id,
                email: credential.email,
                expires: Some(credential.expires),
            }),
        ),
        ParsedAuth::None => (ModelProviderAuthType::None, Vec::new(), None),
    };

    Ok(ModelProviderResponse {
        id: provider.id,
        team_id: provider.team_id,
        provider_key: provider.provider_key,
        display_name: provider.display_name,
        auth_type,
        credential_fields,
        oauth,
        created_at: provider.created_at,
        updated_at: provider.updated_at,
    })
}

pub fn normalize_credential_env_key(key: &str) -> Result<String, ModelProvidersError> {
    let normalized = decode_legacy_snake_case_env_key(key).unwrap_or_else(|| key.to_owned());
    validate_credential_env_key(&normalized)?;
    Ok(normalized)
}
