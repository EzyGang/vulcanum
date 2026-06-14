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
    assert!(rendered
        .opencode_config
        .contains("anthropic/claude-sonnet-4-5"));
    assert!(rendered
        .opencode_config
        .contains("anthropic/claude-haiku-4-5"));
    assert!(rendered.opencode_config.contains("{env:ANTHROPIC_API_KEY}"));
}
