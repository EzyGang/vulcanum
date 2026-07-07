use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::Deserialize;

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

fn parse_stored_auth(
    stored: StoredAuth,
    cipher: &SecretCipher,
) -> Result<ParsedAuth, ModelProvidersError> {
    if stored.schema_version != 1 {
        return Err(ModelProvidersError::InvalidAuthConfig(
            "unsupported model provider auth schema".to_owned(),
        ));
    }

    match stored.auth_type {
        ModelProviderAuthType::ApiKey => {
            let api_key = stored.api_key.ok_or_else(|| {
                ModelProvidersError::InvalidAuthConfig("missing api key auth".to_owned())
            })?;
            let mut credentials: HashMap<String, String> = HashMap::new();
            for field in &api_key.fields {
                let normalized_field = normalize_credential_env_key(field)?;
                let secret = api_key
                    .secrets
                    .get(field)
                    .or_else(|| api_key.secrets.get(normalized_field.as_str()))
                    .ok_or_else(|| {
                        ModelProvidersError::InvalidAuthConfig(format!(
                            "missing encrypted credential for {normalized_field}"
                        ))
                    })?;
                if credentials
                    .insert(normalized_field.clone(), cipher.decrypt(secret)?)
                    .is_some()
                {
                    return Err(ModelProvidersError::InvalidAuthConfig(format!(
                        "duplicate encrypted credential for {normalized_field}"
                    )));
                }
            }
            for secret_field in api_key.secrets.keys() {
                let normalized = normalize_credential_env_key(secret_field)?;
                if !api_key.fields.iter().any(|field| {
                    normalize_credential_env_key(field)
                        .map(|candidate| candidate == normalized)
                        .unwrap_or(false)
                }) {
                    return Err(ModelProvidersError::InvalidAuthConfig(format!(
                        "undeclared encrypted credential for {normalized}"
                    )));
                }
            }
            Ok(ParsedAuth::ApiKey(credentials))
        }
        ModelProviderAuthType::DeviceOauth => {
            let oauth = stored.device_oauth.ok_or_else(|| {
                ModelProvidersError::InvalidAuthConfig("missing oauth auth".to_owned())
            })?;
            Ok(ParsedAuth::DeviceOAuth(OAuthCredential {
                provider: oauth.provider,
                account_id: oauth.account_id,
                email: oauth.email,
                expires: oauth.expires,
                refresh: cipher.decrypt(&oauth.refresh)?,
                access: cipher.decrypt(&oauth.access)?,
            }))
        }
        ModelProviderAuthType::None => Ok(ParsedAuth::None),
    }
}

fn parse_legacy_api_key_auth(
    credentials: &serde_json::Value,
) -> Result<ParsedAuth, ModelProvidersError> {
    let api_keys = parse_api_key_credentials(credentials)?;

    match api_keys.is_empty() {
        true => Ok(ParsedAuth::None),
        false => Ok(ParsedAuth::ApiKey(api_keys)),
    }
}

fn parse_api_key_credentials(
    credentials: &serde_json::Value,
) -> Result<HashMap<String, String>, ModelProvidersError> {
    let object = credentials.as_object().ok_or_else(|| {
        ModelProvidersError::InvalidAuthConfig("credentials must be an object".to_owned())
    })?;
    let mut api_keys: HashMap<String, String> = HashMap::new();
    for (key, value) in object {
        let normalized_key = normalize_credential_env_key(key)?;
        match value.as_str() {
            Some(secret) if !secret.is_empty() => {
                if api_keys
                    .insert(normalized_key.clone(), secret.to_owned())
                    .is_some()
                {
                    return Err(ModelProvidersError::InvalidAuthConfig(format!(
                        "duplicate credential field {normalized_key}"
                    )));
                }
            }
            Some(_) => (),
            None => {
                return Err(ModelProvidersError::InvalidAuthConfig(
                    "credential values must be strings".to_owned(),
                ));
            }
        }
    }
    Ok(api_keys)
}

pub fn normalize_credential_env_key(key: &str) -> Result<String, ModelProvidersError> {
    let normalized = decode_legacy_snake_case_env_key(key).unwrap_or_else(|| key.to_owned());
    validate_credential_env_key(&normalized)?;
    Ok(normalized)
}

fn validate_credential_env_key(key: &str) -> Result<(), ModelProvidersError> {
    if key.is_empty()
        || !key
            .chars()
            .all(|ch| ch.is_ascii_uppercase() || ch.is_ascii_digit() || ch == '_')
        || key.starts_with('_')
        || key.chars().next().is_some_and(|ch| ch.is_ascii_digit())
    {
        return Err(ModelProvidersError::InvalidAuthConfig(format!(
            "invalid credential env field {key}"
        )));
    }
    if DANGEROUS_ENV_KEYS.contains(&key)
        || DANGEROUS_ENV_PREFIXES
            .iter()
            .any(|prefix| key.starts_with(prefix))
    {
        return Err(ModelProvidersError::InvalidAuthConfig(format!(
            "credential env field {key} is not allowed"
        )));
    }
    Ok(())
}

fn encrypted_secrets_value(
    secrets: HashMap<String, EncryptedSecret>,
) -> serde_json::Map<String, serde_json::Value> {
    secrets
        .into_iter()
        .map(|(key, secret)| (key, encrypted_secret_value(&secret)))
        .collect()
}

fn encrypted_secret_value(secret: &EncryptedSecret) -> serde_json::Value {
    serde_json::json!({
        "nonce": secret.nonce.as_str(),
        "ciphertext": secret.ciphertext.as_str(),
    })
}

fn decode_legacy_snake_case_env_key(key: &str) -> Option<String> {
    if !key.starts_with('_') || !key.contains("__") {
        return None;
    }

    let trimmed = key.trim_matches('_');
    if trimmed.is_empty() {
        return None;
    }

    let mut parts: Vec<String> = Vec::new();
    for segment in trimmed.split("__") {
        if segment.is_empty()
            || !segment
                .chars()
                .all(|ch| ch == '_' || ch.is_ascii_lowercase())
        {
            return None;
        }

        let part = segment.replace('_', "").to_ascii_uppercase();
        if part.is_empty() {
            return None;
        }
        parts.push(part);
    }

    match parts.is_empty() {
        true => None,
        false => Some(parts.join("_")),
    }
}
