use std::sync::Arc;

use serde_json::json;

use crate::services::model_providers::auth::device_flow::InMemoryDeviceFlowStore;
use crate::services::model_providers::auth::encryption::SecretCipher;
use crate::services::model_providers::auth::openai_chatgpt::OpenAiChatGptDeviceAuthProvider;
use crate::services::model_providers::catalog::ModelCatalogClient;
use crate::services::model_providers::errors::ModelProvidersError;
use crate::services::model_providers::model::{
    CatalogModel, CatalogProvider, CatalogResponse, CreateModelProviderRequest,
};
use crate::services::model_providers::repository::ModelProvidersRepository;
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
                auth_type:
                    crate::services::model_providers::auth::credentials::ModelProviderAuthType::ApiKey,
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

async fn service(pool: sqlx::PgPool) -> ModelProvidersService {
    ModelProvidersService::new(
        ModelProvidersRepository::new(),
        pool,
        ModelCatalogClient::from_catalog(test_catalog()).await,
        SecretCipher::new("AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=").expect("test cipher"),
        Arc::new(InMemoryDeviceFlowStore::new()),
        Arc::new(OpenAiChatGptDeviceAuthProvider::new()),
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
