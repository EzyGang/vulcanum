use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use base64::Engine;

use crate::services::model_providers::catalog::ModelCatalogClient;
use crate::services::model_providers::errors::ModelProvidersError;
use crate::services::model_providers::model::{
    ChatGptAuthStartResponse, ModelProviderConfig, StartChatGptAuthRequest,
};
use crate::services::model_providers::repository::ModelProvidersRepository;
use crate::services::model_providers::service::oauth_client::{
    ChatGptOAuthClient, DevicePollOutcome, DeviceUserCodeResponse, OAuthRefreshTokenResponse,
    OAuthTokenResponse,
};
use crate::services::model_providers::service::ModelProvidersService;
use crate::services::model_providers::service_tests::test_catalog;

#[derive(Debug)]
struct FakeChatGptOAuthClient {
    polls: Mutex<VecDeque<DevicePollOutcome>>,
    refreshes: Mutex<VecDeque<OAuthRefreshTokenResponse>>,
    refresh_error: Mutex<Option<String>>,
    exchange_expires_in: Option<i64>,
}

#[async_trait::async_trait]
impl ChatGptOAuthClient for FakeChatGptOAuthClient {
    async fn start_device_flow(&self) -> Result<DeviceUserCodeResponse, ModelProvidersError> {
        Ok(DeviceUserCodeResponse {
            device_code: "device-secret".to_owned(),
            user_code: "ABCD-EFGH".to_owned(),
            verification_uri: "https://auth.example/device".to_owned(),
            expires_in: 600,
            interval: Some(1),
        })
    }

    async fn poll_device_token(
        &self,
        _device_code: &str,
    ) -> Result<DevicePollOutcome, ModelProvidersError> {
        Ok(self
            .polls
            .lock()
            .expect("fake poll mutex should not be poisoned")
            .pop_front()
            .unwrap_or(DevicePollOutcome::Pending))
    }

    async fn exchange_authorization_code(
        &self,
        _code: &str,
    ) -> Result<OAuthTokenResponse, ModelProvidersError> {
        Ok(OAuthTokenResponse {
            access_token: "access-token".to_owned(),
            refresh_token: "refresh-token".to_owned(),
            expires_in: self.exchange_expires_in,
            id_token: None,
        })
    }

    async fn refresh_access_token(
        &self,
        _refresh_token: &str,
    ) -> Result<OAuthRefreshTokenResponse, ModelProvidersError> {
        if let Some(message) = self
            .refresh_error
            .lock()
            .expect("fake refresh error mutex should not be poisoned")
            .take()
        {
            return Err(ModelProvidersError::OAuth(message));
        }

        Ok(self
            .refreshes
            .lock()
            .expect("fake refresh mutex should not be poisoned")
            .pop_front()
            .unwrap_or(OAuthRefreshTokenResponse {
                access_token: "refreshed-access-token".to_owned(),
                refresh_token: None,
                expires_in: Some(3600),
                id_token: None,
            }))
    }
}

pub(crate) async fn complete_auth(
    service: &ModelProvidersService,
    team_id: uuid::Uuid,
) -> ModelProviderConfig {
    let start = start_auth(service, team_id).await;
    service
        .chatgpt_auth_status(team_id, "user-1", start.attempt_id)
        .await
        .expect("Should complete auth")
        .provider
        .expect("Should return provider")
}

pub(crate) async fn start_auth(
    service: &ModelProvidersService,
    team_id: uuid::Uuid,
) -> ChatGptAuthStartResponse {
    service
        .start_chatgpt_auth(
            team_id,
            "user-1",
            StartChatGptAuthRequest {
                display_name: String::new(),
            },
        )
        .await
        .expect("Should start device auth")
}

pub(crate) async fn service_with_oauth(
    pool: sqlx::PgPool,
    polls: Vec<DevicePollOutcome>,
) -> ModelProvidersService {
    service_with_oauth_options(pool, polls, Some(3600), Vec::new()).await
}

pub(crate) async fn service_with_oauth_options(
    pool: sqlx::PgPool,
    polls: Vec<DevicePollOutcome>,
    exchange_expires_in: Option<i64>,
    refreshes: Vec<OAuthRefreshTokenResponse>,
) -> ModelProvidersService {
    service_with_oauth_client(pool, polls, exchange_expires_in, refreshes, None).await
}

pub(crate) async fn service_with_oauth_client(
    pool: sqlx::PgPool,
    polls: Vec<DevicePollOutcome>,
    exchange_expires_in: Option<i64>,
    refreshes: Vec<OAuthRefreshTokenResponse>,
    refresh_error: Option<String>,
) -> ModelProvidersService {
    ModelProvidersService::new_for_tests(
        ModelProvidersRepository::new(),
        pool,
        ModelCatalogClient::from_catalog(test_catalog()).await,
        "test-secret",
        Arc::new(FakeChatGptOAuthClient {
            polls: Mutex::new(VecDeque::from(polls)),
            refreshes: Mutex::new(VecDeque::from(refreshes)),
            refresh_error: Mutex::new(refresh_error),
            exchange_expires_in,
        }),
    )
}

#[must_use]
pub(crate) fn unsigned_jwt(payload: serde_json::Value) -> String {
    let encoded_payload = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .encode(serde_json::to_vec(&payload).expect("test JWT payload should serialize"));
    format!("header.{encoded_payload}.signature")
}
