use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::services::model_providers::auth::encryption::{EncryptedSecret, SecretCipher};
use crate::services::model_providers::errors::ModelProvidersError;
use crate::services::model_providers::model::{
    ModelProviderConfig, ModelProviderOAuthStatus, ModelProviderResponse,
};

pub const OPENAI_PROVIDER_KEY: &str = "openai";
pub const OPENAI_CHATGPT_PROVIDER_ID: &str = "openai_chatgpt";

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelProviderAuthType {
    ApiKey,
    DeviceOauth,
    None,
}

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

#[derive(Debug, Deserialize, Serialize)]
struct StoredAuth {
    schema_version: i32,
    auth_type: ModelProviderAuthType,
    #[serde(default)]
    api_key: Option<StoredApiKeyAuth>,
    #[serde(default)]
    device_oauth: Option<StoredOAuthAuth>,
}

#[derive(Debug, Deserialize, Serialize)]
struct StoredApiKeyAuth {
    fields: Vec<String>,
    secrets: HashMap<String, EncryptedSecret>,
}

#[derive(Debug, Deserialize, Serialize)]
struct StoredOAuthAuth {
    provider: String,
    account_id: Option<String>,
    email: Option<String>,
    expires: i64,
    refresh: EncryptedSecret,
    access: EncryptedSecret,
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

    serde_json::to_value(StoredAuth {
        schema_version: 1,
        auth_type: ModelProviderAuthType::ApiKey,
        api_key: Some(StoredApiKeyAuth { fields, secrets }),
        device_oauth: None,
    })
    .map_err(|e| ModelProvidersError::InvalidAuthConfig(e.to_string()))
}

#[must_use = "encrypted credentials must be persisted"]
pub fn encrypted_oauth_credentials(
    credential: &OAuthCredential,
    cipher: &SecretCipher,
) -> Result<serde_json::Value, ModelProvidersError> {
    serde_json::to_value(StoredAuth {
        schema_version: 1,
        auth_type: ModelProviderAuthType::DeviceOauth,
        api_key: None,
        device_oauth: Some(StoredOAuthAuth {
            provider: credential.provider.clone(),
            account_id: credential.account_id.clone(),
            email: credential.email.clone(),
            expires: credential.expires,
            refresh: cipher.encrypt(&credential.refresh)?,
            access: cipher.encrypt(&credential.access)?,
        }),
    })
    .map_err(|e| ModelProvidersError::InvalidAuthConfig(e.to_string()))
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
            for field in api_key.fields {
                if let Some(secret) = api_key.secrets.get(&field) {
                    credentials.insert(field, cipher.decrypt(secret)?);
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
        match value.as_str() {
            Some(secret) if !secret.is_empty() => {
                api_keys.insert(key.to_owned(), secret.to_owned());
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
