use crate::services::model_providers::catalog::{
    is_codex_compatible_openai_model, ModelCatalogClient,
};
use crate::services::model_providers::errors::ModelProvidersError;
use crate::services::model_providers::model::{CatalogModel, CatalogProvider, CatalogResponse};

#[tokio::test]
async fn validate_provider_accepts_known_provider() {
    let catalog = test_catalog();
    let client = ModelCatalogClient::from_catalog(catalog).await;

    let result = client.validate_provider("anthropic").await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn validate_provider_rejects_unknown_provider() {
    let catalog = test_catalog();
    let client = ModelCatalogClient::from_catalog(catalog).await;

    let result = client.validate_provider("missing").await;

    match result {
        Err(ModelProvidersError::UnknownProvider(provider_key)) => {
            assert_eq!(provider_key, "missing");
        }
        _ => panic!("Expected unknown provider error"),
    }
}

#[tokio::test]
async fn validate_model_rejects_unknown_model() {
    let catalog = test_catalog();
    let client = ModelCatalogClient::from_catalog(catalog).await;

    let result = client.validate_model("anthropic", "missing-model").await;

    match result {
        Err(ModelProvidersError::UnknownModel {
            provider_key,
            model_id,
        }) => {
            assert_eq!(provider_key, "anthropic");
            assert_eq!(model_id, "missing-model");
        }
        _ => panic!("Expected unknown model error"),
    }
}

#[test]
fn codex_compatible_model_predicate_matches_policy() {
    assert!(is_codex_compatible_openai_model("gpt-5.5"));
    assert!(is_codex_compatible_openai_model("gpt-5.4"));
    assert!(is_codex_compatible_openai_model("gpt-5.6"));
    assert!(is_codex_compatible_openai_model("gpt-6.0"));
    assert!(!is_codex_compatible_openai_model("gpt-5.5-pro"));
    assert!(!is_codex_compatible_openai_model("gpt-5.3"));
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
