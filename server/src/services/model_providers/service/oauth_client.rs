use serde::Deserialize;
use serde_json::json;

use crate::services::model_providers::errors::ModelProvidersError;

const OPENAI_OAUTH_CLIENT_ID: &str = "app_EMoamEEZ73f0CkXaXp7hrann";
const DEVICE_USER_CODE_URL: &str = "https://auth.openai.com/api/accounts/deviceauth/usercode";
const DEVICE_TOKEN_URL: &str = "https://auth.openai.com/api/accounts/deviceauth/token";
const OAUTH_TOKEN_URL: &str = "https://auth.openai.com/oauth/token";

#[derive(Debug, Clone)]
pub(crate) struct DeviceUserCodeResponse {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    pub expires_in: i64,
    pub interval: Option<i32>,
}

#[derive(Debug, Deserialize)]
struct OpenAiDeviceUserCodeResponse {
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

#[derive(Debug, Clone)]
pub(crate) enum DevicePollOutcome {
    Pending,
    Authorized(String),
    Failed(String),
}

#[derive(Debug, Clone)]
pub(crate) struct OAuthTokenResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: Option<i64>,
    pub id_token: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OpenAiOAuthTokenResponse {
    access_token: String,
    refresh_token: String,
    #[serde(default)]
    expires_in: Option<i64>,
    #[serde(default)]
    id_token: Option<String>,
}

#[async_trait::async_trait]
pub(crate) trait ChatGptOAuthClient: Send + Sync {
    async fn start_device_flow(&self) -> Result<DeviceUserCodeResponse, ModelProvidersError>;

    async fn poll_device_token(
        &self,
        device_code: &str,
    ) -> Result<DevicePollOutcome, ModelProvidersError>;

    async fn exchange_authorization_code(
        &self,
        code: &str,
    ) -> Result<OAuthTokenResponse, ModelProvidersError>;
}

#[derive(Clone)]
pub(crate) struct OpenAiChatGptOAuthClient {
    client: reqwest::Client,
}

impl Default for OpenAiChatGptOAuthClient {
    fn default() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait::async_trait]
impl ChatGptOAuthClient for OpenAiChatGptOAuthClient {
    async fn start_device_flow(&self) -> Result<DeviceUserCodeResponse, ModelProvidersError> {
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
            .json::<OpenAiDeviceUserCodeResponse>()
            .await
            .map_err(|e| {
                ModelProvidersError::OAuth(format!("parsing device flow response: {e}"))
            })?;
        Ok(DeviceUserCodeResponse {
            device_code: body.device_code,
            user_code: body.user_code,
            verification_uri: body.verification_uri,
            expires_in: body.expires_in,
            interval: body.interval,
        })
    }

    async fn poll_device_token(
        &self,
        device_code: &str,
    ) -> Result<DevicePollOutcome, ModelProvidersError> {
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
            Some("authorization_pending") | Some("slow_down") => {
                return Ok(DevicePollOutcome::Pending);
            }
            Some(error) => {
                let description = body.error_description.unwrap_or_else(|| error.to_owned());
                return Ok(DevicePollOutcome::Failed(description));
            }
            None => (),
        }
        if !status.is_success() {
            return Err(ModelProvidersError::OAuth(format!(
                "polling device token returned HTTP {status}"
            )));
        }
        body.code.map(DevicePollOutcome::Authorized).ok_or_else(|| {
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
        let body = response
            .json::<OpenAiOAuthTokenResponse>()
            .await
            .map_err(|e| {
                ModelProvidersError::OAuth(format!("parsing oauth token response: {e}"))
            })?;
        Ok(OAuthTokenResponse {
            access_token: body.access_token,
            refresh_token: body.refresh_token,
            expires_in: body.expires_in,
            id_token: body.id_token,
        })
    }
}
