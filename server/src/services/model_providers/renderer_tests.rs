use chrono::Utc;
use uuid::Uuid;

use crate::services::model_providers::model::ModelProviderConfig;
use crate::services::model_providers::renderer::{render_opencode_config, ModelSelection};

#[test]
fn render_opencode_config_extracts_env_and_models() {
    let provider = ModelProviderConfig {
        id: Uuid::new_v4(),
        team_id: Uuid::new_v4(),
        provider_key: "anthropic".to_owned(),
        display_name: "Anthropic".to_owned(),
        credentials: serde_json::json!({"ANTHROPIC_API_KEY": "secret"}),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    let rendered = render_opencode_config(
        &[provider],
        ModelSelection {
            primary_provider_key: Some("anthropic"),
            primary_model_id: Some("claude-sonnet-4-5"),
            small_provider_key: Some("anthropic"),
            small_model_id: Some("claude-haiku-4-5"),
        },
    );

    assert_eq!(
        rendered.env.get("ANTHROPIC_API_KEY"),
        Some(&"secret".to_owned())
    );
    let config: serde_json::Value = serde_json::from_str(&rendered.opencode_config)
        .expect("rendered config should be valid json");
    assert_eq!(config["model"], "anthropic/claude-sonnet-4-5");
    assert_eq!(config["small_model"], "anthropic/claude-haiku-4-5");
    assert_eq!(
        config["provider"]["anthropic"]["options"]["apiKey"],
        "{env:ANTHROPIC_API_KEY}"
    );
    assert_eq!(config["permission"]["*"], "allow");
    assert_eq!(config["permission"]["question"], "deny");
}

#[test]
fn render_opencode_config_includes_permissions_without_model_config() {
    let rendered = render_opencode_config(
        &[],
        ModelSelection {
            primary_provider_key: None,
            primary_model_id: None,
            small_provider_key: None,
            small_model_id: None,
        },
    );

    let config: serde_json::Value = serde_json::from_str(&rendered.opencode_config)
        .expect("rendered config should be valid json");
    assert_eq!(config["permission"]["*"], "allow");
    assert_eq!(config["permission"]["question"], "deny");
    assert!(config.get("provider").is_none());
}
