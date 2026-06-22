use base64::Engine;
use chrono::Utc;
use serde::Deserialize;
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

use crate::services::model_providers::catalog::ModelCatalogClient;
use crate::services::model_providers::crypto::CredentialCipher;
use crate::services::model_providers::errors::ModelProvidersError;
use crate::services::model_providers::model::{
    CatalogResponse, ChatGptAuthStartResponse, ChatGptAuthStatusResponse,
    CreateModelProviderRequest, ModelProviderConfig, OAuthCredentials, OAuthMetadata,
    StartChatGptAuthRequest, UpdateModelProviderRequest, AUTH_TYPE_API_KEY,
    AUTH_TYPE_CHATGPT_OAUTH, OPENAI_PROVIDER_KEY,
};
use crate::services::model_providers::repository::{
    CreateAuthAttemptParams, CreateOAuthProviderParams, ModelProvidersRepository,
};

const CHATGPT_AUTH_PENDING: &str = "pending";
const CHATGPT_AUTH_COMPLETE: &str = "complete";
const CHATGPT_AUTH_EXPIRED: &str = "expired";
const CHATGPT_AUTH_FAILED: &str = "failed";
const OPENAI_OAUTH_CLIENT_ID: &str = "app_EMoamEEZ73f0CkXaXp7hrann";
const DEVICE_USER_CODE_URL: &str = "https://auth.openai.com/api/accounts/deviceauth/usercode";
const DEVICE_TOKEN_URL: &str = "https://auth.openai.com/api/accounts/deviceauth/token";
const OAUTH_TOKEN_URL: &str = "https://auth.openai.com/oauth/token";
const DEFAULT_CHATGPT_DISPLAY_NAME: &str = "OpenAI ChatGPT Pro/Plus";
const DEFAULT_DEVICE_POLL_SECONDS: i32 = 5;

#[derive(Clone)]
pub struct ModelProvidersService {
    pub repo: ModelProvidersRepository,
    pub db: PgPool,
    pub catalog: ModelCatalogClient,
    client: reqwest::Client,
    cipher: CredentialCipher,
}

#[derive(Debug, Default)]
pub struct SelectedModelProviderAuth {
    pub providers: Vec<ModelProviderConfig>,
    pub opencode_auth_content: Option<String>,
}

#[derive(Debug, Deserialize)]
struct DeviceUserCodeResponse {
    device_code: String,
    user_code: String,
    #[serde(alias = "verification_url")]
    verification_uri: String,
    expires_in: i64,
    #[serde(default)]
    interval: Option<i32>,
}

#[derive(Debug, Deserialize)]
struct DeviceTokenResponse {
    #[serde(default, alias = "authorization_code")]
    code: Option<String>,
    #[serde(default)]
    error: Option<String>,
    #[serde(default)]
    error_description: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OAuthTokenResponse {
    access_token: String,
    refresh_token: String,
    #[serde(default)]
    expires_in: Option<i64>,
    #[serde(default)]
    id_token: Option<String>,
}

impl ModelProvidersService {
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
            client: reqwest::Client::new(),
            cipher: CredentialCipher::new(encryption_secret),
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
            return Err(ModelProvidersError::InvalidAuthType(params.auth_type));
        }
        self.catalog.validate_provider(&params.provider_key).await?;
        self.repo.create(&self.db, team_id, &params).await
    }

    pub async fn update(
        &self,
        id: Uuid,
        team_id: Uuid,
        params: UpdateModelProviderRequest,
    ) -> Result<ModelProviderConfig, ModelProvidersError> {
        let provider = self.repo.find_by_id(&self.db, id, team_id).await?;
        if provider.auth_type == AUTH_TYPE_CHATGPT_OAUTH && params.credentials.is_some() {
            return Err(ModelProvidersError::InvalidSelection(
                "ChatGPT OAuth credentials can only be updated by reconnecting".to_owned(),
            ));
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
        let Some(model_id) = model_id.filter(|value| !value.is_empty()) else {
            return Ok(());
        };

        let provider = self
            .repo
            .find_by_id(&self.db, provider_config_id, team_id)
            .await?;
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

    pub async fn start_chatgpt_auth(
        &self,
        team_id: Uuid,
        user_id: &str,
        params: StartChatGptAuthRequest,
    ) -> Result<ChatGptAuthStartResponse, ModelProvidersError> {
        let response = self
            .client
            .post(DEVICE_USER_CODE_URL)
            .json(&json!({ "client_id": OPENAI_OAUTH_CLIENT_ID }))
            .send()
            .await
            .map_err(|e| ModelProvidersError::OAuth(format!("starting device flow: {e}")))?;
        if !response.status().is_success() {
            return Err(ModelProvidersError::OAuth(format!(
                "starting device flow returned HTTP {}",
                response.status()
            )));
        }
        let body = response
            .json::<DeviceUserCodeResponse>()
            .await
            .map_err(|e| {
                ModelProvidersError::OAuth(format!("parsing device flow response: {e}"))
            })?;
        let attempt = self
            .repo
            .create_auth_attempt(
                &self.db,
                team_id,
                &CreateAuthAttemptParams {
                    user_id,
                    device_code: &body.device_code,
                    user_code: &body.user_code,
                    verification_uri: &body.verification_uri,
                    interval_seconds: body.interval.unwrap_or(DEFAULT_DEVICE_POLL_SECONDS),
                    expires_at: Utc::now() + chrono::Duration::seconds(body.expires_in),
                },
            )
            .await?;

        if !params.display_name.trim().is_empty() {
            tracing::debug!(attempt_id = %attempt.id, "stored requested ChatGPT display name for auth flow");
        }

        Ok(ChatGptAuthStartResponse {
            attempt_id: attempt.id,
            verification_uri: attempt.verification_uri,
            user_code: attempt.user_code,
            expires_at: attempt.expires_at,
            poll_interval_seconds: attempt.interval_seconds,
        })
    }

    pub async fn chatgpt_auth_status(
        &self,
        team_id: Uuid,
        user_id: &str,
        attempt_id: Uuid,
    ) -> Result<ChatGptAuthStatusResponse, ModelProvidersError> {
        let attempt = self
            .repo
            .find_auth_attempt(&self.db, attempt_id, team_id, user_id)
            .await?;
        if attempt.status != CHATGPT_AUTH_PENDING {
            return self
                .auth_status_from_attempt(team_id, &attempt.status, attempt.error)
                .await;
        }
        if attempt.expires_at <= Utc::now() {
            self.repo
                .update_auth_attempt_status(
                    &self.db,
                    attempt.id,
                    CHATGPT_AUTH_EXPIRED,
                    Some("Device login expired"),
                )
                .await?;
            return Ok(ChatGptAuthStatusResponse {
                status: CHATGPT_AUTH_EXPIRED.to_owned(),
                error: Some("Device login expired".to_owned()),
                provider: None,
            });
        }

        let Some(code) = self.poll_device_token(&attempt.device_code).await? else {
            return Ok(ChatGptAuthStatusResponse {
                status: CHATGPT_AUTH_PENDING.to_owned(),
                error: None,
                provider: None,
            });
        };

        let token = self.exchange_authorization_code(&code).await?;
        let expires_at =
            Utc::now() + chrono::Duration::seconds(token.expires_in.unwrap_or(60 * 60 * 24));
        let account_id = token
            .id_token
            .as_deref()
            .and_then(extract_account_id)
            .or_else(|| extract_account_id(&token.access_token));
        let email = token.id_token.as_deref().and_then(extract_email);
        let credentials = OAuthCredentials {
            access: token.access_token,
            refresh: token.refresh_token,
            expires: expires_at.timestamp_millis(),
            account_id: account_id.clone(),
        };
        let metadata = OAuthMetadata {
            account_id,
            email,
            expires_at: Some(expires_at),
        };
        let encrypted = self.cipher.encrypt_json(&credentials)?;
        let metadata_json =
            serde_json::to_value(metadata).map_err(|_| ModelProvidersError::Crypto)?;
        let provider = self
            .repo
            .upsert_chatgpt_oauth(
                &self.db,
                team_id,
                &CreateOAuthProviderParams {
                    display_name: DEFAULT_CHATGPT_DISPLAY_NAME,
                    oauth_credentials: &encrypted,
                    oauth_metadata: &metadata_json,
                },
            )
            .await?;
        self.repo
            .update_auth_attempt_status(&self.db, attempt.id, CHATGPT_AUTH_COMPLETE, None)
            .await?;

        Ok(ChatGptAuthStatusResponse {
            status: CHATGPT_AUTH_COMPLETE.to_owned(),
            error: None,
            provider: Some(provider),
        })
    }

    pub async fn cancel_chatgpt_auth(
        &self,
        team_id: Uuid,
        user_id: &str,
        attempt_id: Uuid,
    ) -> Result<(), ModelProvidersError> {
        let attempt = self
            .repo
            .find_auth_attempt(&self.db, attempt_id, team_id, user_id)
            .await?;
        self.repo
            .update_auth_attempt_status(
                &self.db,
                attempt.id,
                CHATGPT_AUTH_FAILED,
                Some("Device login cancelled"),
            )
            .await
    }

    pub async fn selected_auth_material(
        &self,
        team_id: Uuid,
        primary_provider_config_id: Option<Uuid>,
        small_provider_config_id: Option<Uuid>,
    ) -> Result<SelectedModelProviderAuth, ModelProvidersError> {
        let ids = selected_ids(primary_provider_config_id, small_provider_config_id);
        if ids.is_empty() {
            return Ok(SelectedModelProviderAuth::default());
        }
        let providers = self.repo.find_by_ids(&self.db, team_id, &ids).await?;
        if providers.len() != ids.len() {
            return Err(ModelProvidersError::NotFound);
        }
        validate_auth_compatibility(&providers)?;
        let opencode_auth_content = self.opencode_auth_content(&providers)?;
        Ok(SelectedModelProviderAuth {
            providers,
            opencode_auth_content,
        })
    }

    fn validate_auth_type(&self, auth_type: &str) -> Result<(), ModelProvidersError> {
        match auth_type {
            AUTH_TYPE_API_KEY | AUTH_TYPE_CHATGPT_OAUTH => Ok(()),
            other => Err(ModelProvidersError::InvalidAuthType(other.to_owned())),
        }
    }

    async fn poll_device_token(
        &self,
        device_code: &str,
    ) -> Result<Option<String>, ModelProvidersError> {
        let response = self
            .client
            .post(DEVICE_TOKEN_URL)
            .json(&json!({
                "client_id": OPENAI_OAUTH_CLIENT_ID,
                "device_code": device_code,
            }))
            .send()
            .await
            .map_err(|e| ModelProvidersError::OAuth(format!("polling device token: {e}")))?;
        let status = response.status();
        let body = response.json::<DeviceTokenResponse>().await.map_err(|e| {
            ModelProvidersError::OAuth(format!("parsing device token response: {e}"))
        })?;
        match body.error.as_deref() {
            Some("authorization_pending") | Some("slow_down") => return Ok(None),
            Some(error) => {
                let description = body.error_description.unwrap_or_else(|| error.to_owned());
                return Err(ModelProvidersError::OAuth(description));
            }
            None => (),
        }
        if !status.is_success() {
            return Err(ModelProvidersError::OAuth(format!(
                "polling device token returned HTTP {status}"
            )));
        }
        body.code.map(Some).ok_or_else(|| {
            ModelProvidersError::OAuth("device token response missing code".to_owned())
        })
    }

    async fn exchange_authorization_code(
        &self,
        code: &str,
    ) -> Result<OAuthTokenResponse, ModelProvidersError> {
        let response = self
            .client
            .post(OAUTH_TOKEN_URL)
            .form(&[
                ("client_id", OPENAI_OAUTH_CLIENT_ID),
                ("grant_type", "authorization_code"),
                ("code", code),
            ])
            .send()
            .await
            .map_err(|e| ModelProvidersError::OAuth(format!("exchanging oauth code: {e}")))?;
        if !response.status().is_success() {
            return Err(ModelProvidersError::OAuth(format!(
                "exchanging oauth code returned HTTP {}",
                response.status()
            )));
        }
        response
            .json::<OAuthTokenResponse>()
            .await
            .map_err(|e| ModelProvidersError::OAuth(format!("parsing oauth token response: {e}")))
    }

    async fn auth_status_from_attempt(
        &self,
        team_id: Uuid,
        status: &str,
        error: Option<String>,
    ) -> Result<ChatGptAuthStatusResponse, ModelProvidersError> {
        let provider = match status {
            CHATGPT_AUTH_COMPLETE => self
                .repo
                .find_by_provider_auth(
                    &self.db,
                    team_id,
                    OPENAI_PROVIDER_KEY,
                    AUTH_TYPE_CHATGPT_OAUTH,
                )
                .await
                .ok(),
            _ => None,
        };
        Ok(ChatGptAuthStatusResponse {
            status: status.to_owned(),
            error,
            provider,
        })
    }

    fn opencode_auth_content(
        &self,
        providers: &[ModelProviderConfig],
    ) -> Result<Option<String>, ModelProvidersError> {
        let Some(provider) = providers
            .iter()
            .find(|provider| provider.auth_type == AUTH_TYPE_CHATGPT_OAUTH)
        else {
            return Ok(None);
        };
        let Some(encrypted) = provider.oauth_credentials.as_ref() else {
            return Err(ModelProvidersError::InvalidSelection(
                "ChatGPT OAuth provider is missing credentials".to_owned(),
            ));
        };
        let credentials: OAuthCredentials = self.cipher.decrypt_json(encrypted)?;
        let mut openai = serde_json::Map::new();
        openai.insert("type".to_owned(), json!("oauth"));
        openai.insert("refresh".to_owned(), json!(credentials.refresh));
        openai.insert("access".to_owned(), json!(credentials.access));
        openai.insert("expires".to_owned(), json!(credentials.expires));
        if let Some(account_id) = credentials.account_id {
            openai.insert("accountId".to_owned(), json!(account_id));
        }
        Ok(Some(json!({ OPENAI_PROVIDER_KEY: openai }).to_string()))
    }
}

fn selected_ids(primary: Option<Uuid>, small: Option<Uuid>) -> Vec<Uuid> {
    let mut ids = Vec::new();
    for id in [primary, small].into_iter().flatten() {
        if !ids.contains(&id) {
            ids.push(id);
        }
    }
    ids
}

fn validate_auth_compatibility(
    providers: &[ModelProviderConfig],
) -> Result<(), ModelProvidersError> {
    for (index, provider) in providers.iter().enumerate() {
        for other in providers.iter().skip(index + 1) {
            if provider.provider_key == other.provider_key && provider.auth_type != other.auth_type
            {
                return Err(ModelProvidersError::InvalidSelection(format!(
                    "{} cannot use multiple auth modes in one job",
                    provider.provider_key
                )));
            }
        }
    }
    Ok(())
}

fn extract_account_id(token: &str) -> Option<String> {
    jwt_payload(token).and_then(|payload| {
        payload
            .get("chatgpt_account_id")
            .or_else(|| payload.get("account_id"))
            .and_then(|value| value.as_str())
            .map(str::to_owned)
    })
}

fn extract_email(token: &str) -> Option<String> {
    jwt_payload(token).and_then(|payload| {
        payload
            .get("email")
            .and_then(|value| value.as_str())
            .map(str::to_owned)
    })
}

fn jwt_payload(token: &str) -> Option<serde_json::Value> {
    let payload = token.split('.').nth(1)?;
    let decoded = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(payload)
        .ok()?;
    serde_json::from_slice(&decoded).ok()
}
