mod credentials;
mod selection;

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
use crate::test_helpers::teams::insert_team;
use crate::test_helpers::DEFAULT_TEAM_ID;
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
