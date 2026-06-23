use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use serde_json::json;

use crate::services::model_providers::catalog::ModelCatalogClient;
use crate::services::model_providers::errors::ModelProvidersError;
use crate::services::model_providers::model::{
    CatalogModel, CatalogProvider, CatalogResponse, CreateModelProviderRequest,
    StartChatGptAuthRequest, AUTH_TYPE_API_KEY,
};
use crate::services::model_providers::repository::ModelProvidersRepository;
use crate::services::model_providers::service::oauth_client::{
    ChatGptOAuthClient, DevicePollOutcome, DeviceUserCodeResponse, OAuthTokenResponse,
};
use crate::services::model_providers::service::ModelProvidersService;
use crate::test_helpers::{insert_team, DEFAULT_TEAM_ID};

struct FakeChatGptOAuthClient {
    polls: Mutex<VecDeque<DevicePollOutcome>>,
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
            expires_in: Some(3600),
            id_token: None,
        })
    }
}

#[sqlx::test]
async fn validate_model_selection_skips_empty_selection(pool: sqlx::PgPool) {
    let service = service(pool).await;

    let result = service
        .validate_model_selection(DEFAULT_TEAM_ID, None, Some(""))
        .await;

    assert!(result.is_ok());
}

#[sqlx::test]
async fn validate_model_selection_requires_connected_provider(pool: sqlx::PgPool) {
    let team_id = insert_team(&pool, "Model Team").await;
    let service = service(pool).await;

    let result = service
        .validate_model_selection(team_id, Some(uuid::Uuid::new_v4()), Some("claude-sonnet-4"))
        .await;

    match result {
        Err(ModelProvidersError::NotFound) => (),
        _ => panic!("Expected missing connected provider error"),
    }
}

#[sqlx::test]
async fn validate_model_selection_accepts_connected_catalog_model(pool: sqlx::PgPool) {
    let team_id = insert_team(&pool, "Connected Model Team").await;
    let service = service(pool).await;
    let provider = service
        .create(
            team_id,
            CreateModelProviderRequest {
                provider_key: "anthropic".to_owned(),
                auth_type: AUTH_TYPE_API_KEY.to_owned(),
                display_name: "Anthropic".to_owned(),
                credentials: json!({ "ANTHROPIC_API_KEY": "secret" }),
            },
        )
        .await
        .expect("Should create model provider");

    let result = service
        .validate_model_selection(team_id, Some(provider.id), Some("claude-sonnet-4"))
        .await;

    assert!(result.is_ok());
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
async fn cancel_chatgpt_auth_marks_attempt_failed(pool: sqlx::PgPool) {
    let team_id = insert_team(&pool, "Cancel ChatGPT Auth Team").await;
    let service = service_with_oauth(pool, Vec::new()).await;
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

async fn service(pool: sqlx::PgPool) -> ModelProvidersService {
    ModelProvidersService::new(
        ModelProvidersRepository::new(),
        pool,
        ModelCatalogClient::from_catalog(test_catalog()).await,
        "test-secret",
    )
}

async fn service_with_oauth(
    pool: sqlx::PgPool,
    polls: Vec<DevicePollOutcome>,
) -> ModelProvidersService {
    ModelProvidersService::new_for_tests(
        ModelProvidersRepository::new(),
        pool,
        ModelCatalogClient::from_catalog(test_catalog()).await,
        "test-secret",
        Arc::new(FakeChatGptOAuthClient {
            polls: Mutex::new(VecDeque::from(polls)),
        }),
    )
}

fn test_catalog() -> CatalogResponse {
    CatalogResponse {
        providers: vec![CatalogProvider {
            id: "anthropic".to_owned(),
            name: "Anthropic".to_owned(),
            doc: String::new(),
            env: vec!["ANTHROPIC_API_KEY".to_owned()],
            models: vec![CatalogModel {
                id: "claude-sonnet-4".to_owned(),
                name: "Claude Sonnet 4".to_owned(),
                status: None,
                context_limit: None,
                output_limit: None,
                input_cost: None,
                output_cost: None,
                attachment: false,
                reasoning: true,
                tool_call: true,
                structured_output: true,
            }],
        }],
    }
}
