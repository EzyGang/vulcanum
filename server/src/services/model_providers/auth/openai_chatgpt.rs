use async_trait::async_trait;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use chrono::Utc;
use serde::Deserialize;

use crate::services::model_providers::auth::credentials::{
    OAuthCredential, OPENAI_CHATGPT_PROVIDER_ID, OPENAI_PROVIDER_KEY,
};
use crate::services::model_providers::auth::device_flow::{
    DeviceAuthProvider, DevicePoll, DeviceStart, PendingDeviceFlow,
};
use crate::services::model_providers::errors::ModelProvidersError;

const CLIENT_ID: &str = "app_EMoamEEZ73f0CkXaXp7hrann";
const ISSUER: &str = "https://auth.openai.com";
const VERIFICATION_URI: &str = "https://auth.openai.com/codex/device";
const REDIRECT_URI: &str = "https://auth.openai.com/deviceauth/callback";

#[derive(Clone)]
pub struct OpenAiChatGptDeviceAuthProvider {
    client: reqwest::Client,
}

#[derive(Deserialize)]
struct StartResponse {
    device_auth_id: String,
    user_code: String,
    interval: String,
}

#[derive(Deserialize)]
struct PollSuccessResponse {
    authorization_code: String,
    code_verifier: String,
}

#[derive(Deserialize)]
struct TokenResponse {
    id_token: Option<String>,
    access_token: String,
    refresh_token: String,
    expires_in: i64,
}

impl OpenAiChatGptDeviceAuthProvider {
    #[must_use]
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    async fn exchange_code(
        &self,
        authorization_code: &str,
        code_verifier: &str,
    ) -> Result<OAuthCredential, ModelProvidersError> {
        let response = self
            .client
            .post(format!("{ISSUER}/oauth/token"))
            .form(&[
                ("grant_type", "authorization_code"),
                ("code", authorization_code),
                ("redirect_uri", REDIRECT_URI),
                ("client_id", CLIENT_ID),
                ("code_verifier", code_verifier),
            ])
            .send()
            .await
            .map_err(|e| ModelProvidersError::DeviceFlowFailed(e.to_string()))?;

        if !response.status().is_success() {
            return Err(ModelProvidersError::DeviceFlowFailed(format!(
                "token exchange returned HTTP {}",
                response.status()
            )));
        }

        let token = response
            .json::<TokenResponse>()
            .await
            .map_err(|e| ModelProvidersError::DeviceFlowFailed(e.to_string()))?;
        Ok(token.into_credential())
    }
}

impl Default for OpenAiChatGptDeviceAuthProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DeviceAuthProvider for OpenAiChatGptDeviceAuthProvider {
    fn provider_id(&self) -> &'static str {
        OPENAI_CHATGPT_PROVIDER_ID
    }

    fn model_provider_key(&self) -> &'static str {
        OPENAI_PROVIDER_KEY
    }

    async fn start(&self) -> Result<DeviceStart, ModelProvidersError> {
        let response = self
            .client
            .post(format!("{ISSUER}/api/accounts/deviceauth/usercode"))
            .json(&serde_json::json!({"client_id": CLIENT_ID}))
            .send()
            .await
            .map_err(|e| ModelProvidersError::DeviceFlowFailed(e.to_string()))?;

        if !response.status().is_success() {
            return Err(ModelProvidersError::DeviceFlowFailed(format!(
                "device start returned HTTP {}",
                response.status()
            )));
        }

        let body = response
            .json::<StartResponse>()
            .await
            .map_err(|e| ModelProvidersError::DeviceFlowFailed(e.to_string()))?;
        let interval_seconds = body.interval.parse::<i64>().unwrap_or(5).max(1);

        Ok(DeviceStart {
            device_auth_id: body.device_auth_id,
            user_code: body.user_code,
            verification_uri: VERIFICATION_URI.to_owned(),
            interval_seconds,
        })
    }

    async fn poll(&self, pending: &PendingDeviceFlow) -> Result<DevicePoll, ModelProvidersError> {
        let response = self
            .client
            .post(format!("{ISSUER}/api/accounts/deviceauth/token"))
            .json(&serde_json::json!({
                "device_auth_id": pending.device_auth_id,
                "user_code": pending.user_code,
            }))
            .send()
            .await
            .map_err(|e| ModelProvidersError::DeviceFlowFailed(e.to_string()))?;

        if response.status() == reqwest::StatusCode::FORBIDDEN
            || response.status() == reqwest::StatusCode::NOT_FOUND
        {
            return Ok(DevicePoll::Pending);
        }
        if !response.status().is_success() {
            return Err(ModelProvidersError::DeviceFlowFailed(format!(
                "device poll returned HTTP {}",
                response.status()
            )));
        }

        let body = response
            .json::<PollSuccessResponse>()
            .await
            .map_err(|e| ModelProvidersError::DeviceFlowFailed(e.to_string()))?;
        self.exchange_code(&body.authorization_code, &body.code_verifier)
            .await
            .map(DevicePoll::Complete)
    }

    async fn refresh(
        &self,
        credential: &OAuthCredential,
    ) -> Result<OAuthCredential, ModelProvidersError> {
        let response = self
            .client
            .post(format!("{ISSUER}/oauth/token"))
            .form(&[
                ("grant_type", "refresh_token"),
                ("refresh_token", credential.refresh.as_str()),
                ("client_id", CLIENT_ID),
            ])
            .send()
            .await
            .map_err(|e| ModelProvidersError::OAuthRefreshFailed(e.to_string()))?;

        if !response.status().is_success() {
            return Err(ModelProvidersError::OAuthRefreshFailed(format!(
                "refresh returned HTTP {}",
                response.status()
            )));
        }

        let mut refreshed = response
            .json::<TokenResponse>()
            .await
            .map_err(|e| ModelProvidersError::OAuthRefreshFailed(e.to_string()))?
            .into_credential();
        if refreshed.account_id.is_none() {
            refreshed.account_id = credential.account_id.clone();
        }
        if refreshed.email.is_none() {
            refreshed.email = credential.email.clone();
        }
        Ok(refreshed)
    }
}

impl TokenResponse {
    fn into_credential(self) -> OAuthCredential {
        let account_id = self
            .id_token
            .as_deref()
            .and_then(extract_account_id)
            .or_else(|| extract_account_id(&self.access_token));
        let email = self
            .id_token
            .as_deref()
            .and_then(extract_claims)
            .and_then(|claims| {
                claims
                    .get("email")
                    .and_then(|value| value.as_str())
                    .map(str::to_owned)
            });

        OAuthCredential {
            provider: OPENAI_CHATGPT_PROVIDER_ID.to_owned(),
            account_id,
            email,
            expires: (Utc::now() + chrono::Duration::seconds(self.expires_in)).timestamp_millis(),
            refresh: self.refresh_token,
            access: self.access_token,
        }
    }
}

pub fn extract_account_id(token: &str) -> Option<String> {
    let claims = extract_claims(token)?;
    claims
        .get("chatgpt_account_id")
        .or_else(|| claims.get("https://api.openai.com/auth.chatgpt_account_id"))
        .and_then(|value| value.as_str())
        .map(str::to_owned)
        .or_else(|| {
            claims
                .get("organizations")
                .and_then(|value| value.as_array())
                .and_then(|organizations| organizations.first())
                .and_then(|organization| organization.get("id"))
                .and_then(|value| value.as_str())
                .map(str::to_owned)
        })
}

fn extract_claims(token: &str) -> Option<serde_json::Map<String, serde_json::Value>> {
    let payload = token.split('.').nth(1)?;
    let bytes = URL_SAFE_NO_PAD.decode(payload).ok()?;
    serde_json::from_slice::<serde_json::Value>(&bytes)
        .ok()?
        .as_object()
        .cloned()
}
