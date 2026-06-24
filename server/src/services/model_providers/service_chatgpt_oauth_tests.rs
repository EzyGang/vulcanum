use crate::services::model_providers::catalog::ModelCatalogClient;
use crate::services::model_providers::errors::ModelProvidersError;
use crate::services::model_providers::model::{
    ChatGptAuthStartResponse, ModelProviderConfig, StartChatGptAuthRequest,
    UpdateModelProviderRequest,
};
use crate::services::model_providers::repository::ModelProvidersRepository;
use crate::services::model_providers::service::oauth_client::{
    ChatGptOAuthClient, DevicePollOutcome, DeviceUserCodeResponse, OAuthRefreshTokenResponse,
    OAuthTokenResponse,
};
use crate::services::model_providers::service::ModelProvidersService;
use crate::services::model_providers::service_tests::test_catalog;
use crate::test_helpers::insert_team;
use chrono::Utc;
use serde_json::json;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
#[derive(Debug)]
struct FakeChatGptOAuthClient {
    polls: Mutex<VecDeque<DevicePollOutcome>>,
    refreshes: Mutex<VecDeque<OAuthRefreshTokenResponse>>,
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
#[sqlx::test]
async fn chatgpt_auth_flow_encrypts_device_code_and_uses_display_name(pool: sqlx::PgPool) {
    let team_id = insert_team(&pool, "ChatGPT Auth Team").await;
    let service = service_with_oauth(
        pool.clone(),
        vec![
            DevicePollOutcome::Pending,
            DevicePollOutcome::Authorized("auth-code".to_owned()),
        ],
    )
    .await;

    let start = service
        .start_chatgpt_auth(
            team_id,
            "user-1",
            StartChatGptAuthRequest {
                display_name: "Custom ChatGPT".to_owned(),
            },
        )
        .await
        .expect("Should start device auth");
    let stored = sqlx::query!(
        r#"SELECT encrypted_device_code::text AS "encrypted_device_code!"
           FROM model_provider_auth_attempts WHERE id = $1"#,
        start.attempt_id,
    )
    .fetch_one(&pool)
    .await
    .expect("Should fetch auth attempt");
    assert!(!stored.encrypted_device_code.contains("device-secret"));

    let pending = service
        .chatgpt_auth_status(team_id, "user-1", start.attempt_id)
        .await
        .expect("Should poll pending status");
    assert_eq!(pending.status, "pending");
    assert!(pending.provider.is_none());
    let complete = service
        .chatgpt_auth_status(team_id, "user-1", start.attempt_id)
        .await
        .expect("Should complete device auth");
    let provider = complete.provider.expect("Should return connected provider");
    assert_eq!(complete.status, "complete");
    assert_eq!(provider.display_name, "Custom ChatGPT");
    assert!(provider.oauth_credentials.is_some());
}
#[sqlx::test]
async fn chatgpt_auth_slow_down_backs_off_poll_interval(pool: sqlx::PgPool) {
    let team_id = insert_team(&pool, "Slow Down ChatGPT Auth Team").await;
    let service = service_with_oauth(pool, vec![DevicePollOutcome::SlowDown]).await;
    let start = service
        .start_chatgpt_auth(
            team_id,
            "user-1",
            StartChatGptAuthRequest {
                display_name: String::new(),
            },
        )
        .await
        .expect("Should start device auth");
    let status = service
        .chatgpt_auth_status(team_id, "user-1", start.attempt_id)
        .await
        .expect("Should poll slow down status");
    assert_eq!(status.status, "pending");
    assert_eq!(status.poll_interval_seconds, Some(6));
}
#[sqlx::test]
async fn chatgpt_oauth_rename_allows_empty_credentials_payload(pool: sqlx::PgPool) {
    let team_id = insert_team(&pool, "Rename ChatGPT Auth Team").await;
    let service = service_with_oauth(
        pool,
        vec![DevicePollOutcome::Authorized("auth-code".to_owned())],
    )
    .await;
    let provider = complete_auth(&service, team_id).await;

    let updated = service
        .update(
            provider.id,
            team_id,
            UpdateModelProviderRequest {
                display_name: Some("Renamed ChatGPT".to_owned()),
                credentials: Some(json!({})),
            },
        )
        .await
        .expect("Should rename OAuth provider");
    assert_eq!(updated.display_name, "Renamed ChatGPT");
}
#[sqlx::test]
async fn selected_auth_material_refreshes_expired_chatgpt_oauth(pool: sqlx::PgPool) {
    let team_id = insert_team(&pool, "Refresh ChatGPT Auth Team").await;
    let service = service_with_oauth_options(
        pool,
        vec![DevicePollOutcome::Authorized("auth-code".to_owned())],
        Some(-60),
        vec![OAuthRefreshTokenResponse {
            access_token: "refreshed-access-token".to_owned(),
            refresh_token: Some("new-refresh-token".to_owned()),
            expires_in: Some(3600),
            id_token: None,
        }],
    )
    .await;
    let provider = complete_auth(&service, team_id).await;

    let selected = service
        .selected_auth_material(team_id, Some(provider.id), None)
        .await
        .expect("Should select refreshed auth material");
    let auth_content = selected
        .opencode_auth_content
        .expect("Should render OpenCode auth content");
    let auth: serde_json::Value =
        serde_json::from_str(&auth_content).expect("auth content should be valid json");

    assert_eq!(auth["openai"]["access"], "refreshed-access-token");
    assert_eq!(auth["openai"]["refresh"], "new-refresh-token");
    assert!(
        auth["openai"]["expires"]
            .as_i64()
            .expect("expires should be an integer")
            > Utc::now().timestamp_millis()
    );
}
#[sqlx::test]
async fn cancel_chatgpt_auth_marks_attempt_failed(pool: sqlx::PgPool) {
    let team_id = insert_team(&pool, "Cancel ChatGPT Auth Team").await;
    let service = service_with_oauth(pool, Vec::new()).await;
    let start = start_auth(&service, team_id).await;

    service
        .cancel_chatgpt_auth(team_id, "user-1", start.attempt_id)
        .await
        .expect("Should cancel device auth");
    let status = service
        .chatgpt_auth_status(team_id, "user-1", start.attempt_id)
        .await
        .expect("Should read cancelled status");
    assert_eq!(status.status, "failed");
    assert_eq!(status.error.as_deref(), Some("Device login cancelled"));
}
#[sqlx::test]
async fn cancel_chatgpt_auth_does_not_overwrite_complete_attempt(pool: sqlx::PgPool) {
    let team_id = insert_team(&pool, "Late Cancel ChatGPT Auth Team").await;
    let service = service_with_oauth(
        pool,
        vec![DevicePollOutcome::Authorized("auth-code".to_owned())],
    )
    .await;
    let start = start_auth(&service, team_id).await;
    service
        .chatgpt_auth_status(team_id, "user-1", start.attempt_id)
        .await
        .expect("Should complete auth");

    service
        .cancel_chatgpt_auth(team_id, "user-1", start.attempt_id)
        .await
        .expect("Late cancel should be idempotent");
    let status = service
        .chatgpt_auth_status(team_id, "user-1", start.attempt_id)
        .await
        .expect("Should read completed status");
    assert_eq!(status.status, "complete");
    assert!(status.provider.is_some());
}
async fn complete_auth(
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
async fn start_auth(
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
async fn service_with_oauth(
    pool: sqlx::PgPool,
    polls: Vec<DevicePollOutcome>,
) -> ModelProvidersService {
    service_with_oauth_options(pool, polls, Some(3600), Vec::new()).await
}
async fn service_with_oauth_options(
    pool: sqlx::PgPool,
    polls: Vec<DevicePollOutcome>,
    exchange_expires_in: Option<i64>,
    refreshes: Vec<OAuthRefreshTokenResponse>,
) -> ModelProvidersService {
    ModelProvidersService::new_for_tests(
        ModelProvidersRepository::new(),
        pool,
        ModelCatalogClient::from_catalog(test_catalog()).await,
        "test-secret",
        Arc::new(FakeChatGptOAuthClient {
            polls: Mutex::new(VecDeque::from(polls)),
            refreshes: Mutex::new(VecDeque::from(refreshes)),
            exchange_expires_in,
        }),
    )
}
