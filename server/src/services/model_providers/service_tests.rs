use serde_json::json;

use crate::services::model_providers::catalog::ModelCatalogClient;
use crate::services::model_providers::errors::ModelProvidersError;
use crate::services::model_providers::model::{
    CatalogModel, CatalogProvider, CatalogResponse, CreateModelProviderRequest, AUTH_TYPE_API_KEY,
    AUTH_TYPE_CHATGPT_OAUTH, OPENAI_PROVIDER_KEY,
};
use crate::services::model_providers::repository::{
    CreateOAuthProviderParams, ModelProvidersRepository,
};
use crate::services::model_providers::service::ModelProvidersService;
use crate::test_helpers::{insert_team, DEFAULT_TEAM_ID};

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
async fn validate_model_selection_checks_provider_without_model(pool: sqlx::PgPool) {
    let team_id = insert_team(&pool, "Provider Only Model Team").await;
    let service = service(pool).await;

    let result = service
        .validate_model_selection(team_id, Some(uuid::Uuid::new_v4()), None)
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
async fn provider_config_id_for_key_prefers_api_key_config(pool: sqlx::PgPool) {
    let team_id = insert_team(&pool, "Legacy Provider Key Team").await;
    let service = service(pool.clone()).await;
    let api_key_provider = service
        .create(
            team_id,
            CreateModelProviderRequest {
                provider_key: OPENAI_PROVIDER_KEY.to_owned(),
                auth_type: AUTH_TYPE_API_KEY.to_owned(),
                display_name: "OpenAI API".to_owned(),
                credentials: json!({ "OPENAI_API_KEY": "secret" }),
            },
        )
        .await
        .expect("API key provider should be created");
    let oauth_credentials = json!({ "access": "token" });
    let oauth_metadata = json!({});
    ModelProvidersRepository::new()
        .upsert_chatgpt_oauth(
            &pool,
            team_id,
            &CreateOAuthProviderParams {
                display_name: "ChatGPT",
                oauth_credentials: &oauth_credentials,
                oauth_metadata: &oauth_metadata,
            },
        )
        .await
        .expect("OAuth provider should be created");

    let provider_config_id = service
        .provider_config_id_for_key(team_id, OPENAI_PROVIDER_KEY)
        .await
        .expect("legacy provider key should resolve");

    assert_eq!(provider_config_id, api_key_provider.id);
}

#[sqlx::test]
async fn create_rejects_chatgpt_oauth_with_device_login_message(pool: sqlx::PgPool) {
    let team_id = insert_team(&pool, "ChatGPT Create Error Team").await;
    let service = service(pool).await;

    let result = service
        .create(
            team_id,
            CreateModelProviderRequest {
                provider_key: OPENAI_PROVIDER_KEY.to_owned(),
                auth_type: AUTH_TYPE_CHATGPT_OAUTH.to_owned(),
                display_name: "ChatGPT".to_owned(),
                credentials: json!({}),
            },
        )
        .await;

    match result {
        Err(ModelProvidersError::InvalidSelection(message)) => {
            assert_eq!(
                message,
                "ChatGPT OAuth providers must be created via device login"
            );
        }
        _ => panic!("Expected ChatGPT OAuth device login error"),
    }
}

async fn service(pool: sqlx::PgPool) -> ModelProvidersService {
    ModelProvidersService::new(
        ModelProvidersRepository::new(),
        pool,
        ModelCatalogClient::from_catalog(test_catalog()).await,
        "test-secret",
    )
}

pub(crate) fn test_catalog() -> CatalogResponse {
    CatalogResponse {
        providers: vec![
            CatalogProvider {
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
            },
            CatalogProvider {
                id: OPENAI_PROVIDER_KEY.to_owned(),
                name: "OpenAI".to_owned(),
                doc: String::new(),
                env: vec!["OPENAI_API_KEY".to_owned()],
                models: vec![CatalogModel {
                    id: "gpt-5".to_owned(),
                    name: "GPT-5".to_owned(),
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
            },
        ],
    }
}
