use std::sync::Arc;

use serde_json::{json, Value};
use vulcanum_shared::api::wire::AgentBackend;

use crate::db::model_providers::ModelProvidersRepository;
use crate::models::model_providers::errors::ModelProvidersError;
use crate::models::model_providers::model::{
    CatalogModel, CatalogProvider, CatalogResponse, CreateModelProviderRequest,
    ModelProviderAuthType, UpdateModelProviderRequest,
};
use crate::services::model_providers::auth::credentials::encrypted_api_key_credentials;
use crate::services::model_providers::auth::device_flow::InMemoryDeviceFlowStore;
use crate::services::model_providers::auth::encryption::SecretCipher;
use crate::services::model_providers::auth::openai_chatgpt::OpenAiChatGptDeviceAuthProvider;
use crate::services::model_providers::catalog::ModelCatalogClient;
use crate::services::model_providers::renderer::ModelSelection;
use crate::services::model_providers::service::ModelProvidersService;
use crate::test_helpers::{insert_team, DEFAULT_TEAM_ID};

#[sqlx::test]
async fn validate_model_selection_skips_empty_selection(pool: sqlx::PgPool) {
    let service = service(pool).await;

    let result = service
        .validate_model_selection(DEFAULT_TEAM_ID, Some(""), Some(""))
        .await;

    assert!(result.is_ok());
}

#[sqlx::test]
async fn validate_model_selection_requires_connected_provider(pool: sqlx::PgPool) {
    let team_id = insert_team(&pool, "Model Team").await;
    let service = service(pool).await;

    let result = service
        .validate_model_selection(team_id, Some("anthropic"), Some("claude-sonnet-4"))
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
    service
        .create(
            team_id,
            CreateModelProviderRequest {
                provider_key: "anthropic".to_owned(),
                display_name: "Anthropic".to_owned(),
                auth_type: ModelProviderAuthType::ApiKey,
                credentials: json!({ "ANTHROPIC_API_KEY": "secret" }),
            },
        )
        .await
        .expect("Should create model provider");

    let result = service
        .validate_model_selection(team_id, Some("anthropic"), Some("claude-sonnet-4"))
        .await;

    assert!(result.is_ok());
}

#[sqlx::test]
async fn create_rejects_non_catalog_api_key_field_before_persistence(pool: sqlx::PgPool) {
    let team_id = insert_team(&pool, "Rejected Field Team").await;
    let service = service(pool).await;

    let result = service
        .create(
            team_id,
            CreateModelProviderRequest {
                provider_key: "anthropic".to_owned(),
                display_name: "Anthropic".to_owned(),
                auth_type: ModelProviderAuthType::ApiKey,
                credentials: json!({
                    "ANTHROPIC_API_KEY": "secret",
                    "OPENAI_API_KEY": "wrong-provider-secret",
                }),
            },
        )
        .await;

    assert_invalid_auth_config(
        result,
        "credential field OPENAI_API_KEY is not allowed for provider anthropic",
    );
    assert!(service
        .list_all(team_id)
        .await
        .expect("list providers")
        .is_empty());
}

#[sqlx::test]
async fn create_rejects_dangerous_env_key_before_persistence(pool: sqlx::PgPool) {
    let team_id = insert_team(&pool, "Dangerous Field Team").await;
    let service = service(pool).await;

    let result = service
        .create(
            team_id,
            CreateModelProviderRequest {
                provider_key: "anthropic".to_owned(),
                display_name: "Anthropic".to_owned(),
                auth_type: ModelProviderAuthType::ApiKey,
                credentials: json!({ "PATH": "/tmp/fake-bin" }),
            },
        )
        .await;

    assert_invalid_auth_config(result, "credential env field PATH is not allowed");
    assert!(service
        .list_all(team_id)
        .await
        .expect("list providers")
        .is_empty());
}

#[sqlx::test]
async fn render_rejects_stored_non_catalog_api_key_field(pool: sqlx::PgPool) {
    let team_id = insert_team(&pool, "Render Rejected Field Team").await;
    let credentials = encrypted_api_key_credentials(
        &json!({ "OPENAI_API_KEY": "wrong-provider-secret" }),
        &test_cipher(),
    )
    .expect("encrypt credentials");
    ModelProvidersRepository::new()
        .create(
            &pool,
            team_id,
            &CreateModelProviderRequest {
                provider_key: "anthropic".to_owned(),
                display_name: "Anthropic".to_owned(),
                auth_type: ModelProviderAuthType::ApiKey,
                credentials,
            },
        )
        .await
        .expect("insert stored provider");
    let service = service(pool).await;

    let result = service
        .render_agent_config_for_team(
            team_id,
            AgentBackend::OpenCode,
            ModelSelection {
                primary_provider_key: Some("anthropic"),
                primary_model_id: Some("claude-sonnet-4"),
                small_provider_key: None,
                small_model_id: None,
            },
        )
        .await;

    assert_invalid_auth_config(
        result,
        "credential field OPENAI_API_KEY is not allowed for provider anthropic",
    );
}

#[sqlx::test]
async fn render_rejects_stored_dangerous_api_key_field(pool: sqlx::PgPool) {
    let team_id = insert_team(&pool, "Render Dangerous Field Team").await;
    let cipher = test_cipher();
    let secret = cipher.encrypt("secret").expect("encrypt secret");
    let credentials = json!({
        "schema_version": 1,
        "auth_type": "api_key",
        "api_key": {
            "fields": ["PATH"],
            "secrets": {
                "PATH": {
                    "nonce": secret.nonce,
                    "ciphertext": secret.ciphertext,
                },
            },
        },
        "device_oauth": null,
    });
    ModelProvidersRepository::new()
        .create(
            &pool,
            team_id,
            &CreateModelProviderRequest {
                provider_key: "anthropic".to_owned(),
                display_name: "Anthropic".to_owned(),
                auth_type: ModelProviderAuthType::ApiKey,
                credentials,
            },
        )
        .await
        .expect("insert stored provider");
    let service = service(pool).await;

    let result = service
        .render_agent_config_for_team(
            team_id,
            AgentBackend::OpenCode,
            ModelSelection {
                primary_provider_key: Some("anthropic"),
                primary_model_id: Some("claude-sonnet-4"),
                small_provider_key: None,
                small_model_id: None,
            },
        )
        .await;

    assert_invalid_auth_config(result, "credential env field PATH is not allowed");
}

#[sqlx::test]
async fn update_to_none_auth_clears_stored_credentials(pool: sqlx::PgPool) {
    let team_id = insert_team(&pool, "None Auth Team").await;
    let service = service(pool.clone()).await;
    let created = service
        .create(
            team_id,
            CreateModelProviderRequest {
                provider_key: "anthropic".to_owned(),
                display_name: "Anthropic".to_owned(),
                auth_type: ModelProviderAuthType::ApiKey,
                credentials: json!({ "ANTHROPIC_API_KEY": "secret" }),
            },
        )
        .await
        .expect("create provider");

    let updated = service
        .update(
            created.id,
            team_id,
            UpdateModelProviderRequest {
                display_name: None,
                auth_type: Some(ModelProviderAuthType::None),
                credentials: None,
            },
        )
        .await
        .expect("update provider auth");

    assert_eq!(updated.auth_type, ModelProviderAuthType::None);
    assert!(updated.credential_fields.is_empty());
    assert!(updated.oauth.is_none());

    let stored_credentials: Value = sqlx::query!(
        "SELECT credentials FROM model_provider_configs WHERE id = $1",
        created.id,
    )
    .fetch_one(&pool)
    .await
    .expect("fetch stored provider")
    .credentials;
    assert!(stored_credentials.is_null());
}

async fn service(pool: sqlx::PgPool) -> ModelProvidersService {
    ModelProvidersService::new(
        ModelProvidersRepository::new(),
        pool,
        ModelCatalogClient::from_catalog(test_catalog()).await,
        test_cipher(),
        Arc::new(InMemoryDeviceFlowStore::new()),
        Arc::new(OpenAiChatGptDeviceAuthProvider::new().expect("build device auth client")),
    )
}

fn test_cipher() -> SecretCipher {
    SecretCipher::new("AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=").expect("test cipher")
}

fn test_catalog() -> CatalogResponse {
    CatalogResponse {
        providers: vec![CatalogProvider {
            id: "anthropic".to_owned(),
            name: "Anthropic".to_owned(),
            doc: String::new(),
            env: vec!["ANTHROPIC_API_KEY".to_owned(), "PATH".to_owned()],
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
                opencode_chatgpt_compatible: false,
            }],
        }],
    }
}

fn assert_invalid_auth_config<T>(result: Result<T, ModelProvidersError>, expected_message: &str) {
    match result {
        Err(ModelProvidersError::InvalidAuthConfig(message)) => {
            assert_eq!(message, expected_message);
        }
        _ => panic!("Expected invalid auth config error"),
    }
}
