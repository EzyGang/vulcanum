use chrono::Utc;
use uuid::Uuid;
use vulcanum_shared::api_types::{AgentBackend, AgentConfigPayload};

use crate::models::model_providers::model::ModelProviderConfig;
use crate::services::model_providers::auth::credentials::{
    encrypted_oauth_credentials, OAuthCredential,
};
use crate::services::model_providers::auth::encryption::SecretCipher;
use crate::services::model_providers::renderer::{
    render_agent_config, render_opencode_config, ModelSelection,
};

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
        &test_cipher(),
        ModelSelection {
            primary_provider_key: Some("anthropic"),
            primary_model_id: Some("claude-sonnet-4-5"),
            small_provider_key: Some("anthropic"),
            small_model_id: Some("claude-haiku-4-5"),
        },
    )
    .expect("render config");

    assert_eq!(
        rendered.env.get("ANTHROPIC_API_KEY"),
        Some(&"secret".to_owned())
    );
    assert_eq!(
        rendered.model.as_deref(),
        Some("anthropic/claude-sonnet-4-5")
    );
    assert_eq!(
        rendered.small_model.as_deref(),
        Some("anthropic/claude-haiku-4-5")
    );
    assert_eq!(
        rendered
            .providers
            .get("anthropic")
            .and_then(|provider| provider.options.get("apiKey"))
            .map(String::as_str),
        Some("{env:ANTHROPIC_API_KEY}")
    );
}

#[test]
fn render_opencode_config_skips_empty_model_config() {
    let rendered = render_opencode_config(
        &[],
        &test_cipher(),
        ModelSelection {
            primary_provider_key: None,
            primary_model_id: None,
            small_provider_key: None,
            small_model_id: None,
        },
    )
    .expect("render config");

    assert!(rendered.providers.is_empty());
    assert!(rendered.model.is_none());
    assert!(rendered.small_model.is_none());
}

#[test]
fn render_opencode_config_restores_legacy_snake_cased_env_keys() {
    let provider = ModelProviderConfig {
        id: Uuid::new_v4(),
        team_id: Uuid::new_v4(),
        provider_key: "deepseek".to_owned(),
        display_name: "DeepSeek".to_owned(),
        credentials: serde_json::json!({"_d_e_e_p_s_e_e_k__a_p_i__k_e_y": "secret"}),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    let rendered = render_opencode_config(
        &[provider],
        &test_cipher(),
        ModelSelection {
            primary_provider_key: Some("deepseek"),
            primary_model_id: Some("deepseek-chat"),
            small_provider_key: None,
            small_model_id: None,
        },
    )
    .expect("render config");

    assert_eq!(
        rendered.env.get("DEEPSEEK_API_KEY"),
        Some(&"secret".to_owned())
    );
    assert_eq!(
        rendered
            .providers
            .get("deepseek")
            .and_then(|provider| provider.options.get("apiKey"))
            .map(String::as_str),
        Some("{env:DEEPSEEK_API_KEY}")
    );
}

#[test]
fn render_opencode_config_materializes_chatgpt_oauth_content() {
    let cipher = test_cipher();
    let credentials = encrypted_oauth_credentials(
        &OAuthCredential {
            provider: "openai_chatgpt".to_owned(),
            account_id: Some("acct".to_owned()),
            email: None,
            expires: 1782942233000,
            refresh: "refresh".to_owned(),
            access: "access".to_owned(),
        },
        &cipher,
    )
    .expect("encrypt oauth");
    let provider = ModelProviderConfig {
        id: Uuid::new_v4(),
        team_id: Uuid::new_v4(),
        provider_key: "openai".to_owned(),
        display_name: "ChatGPT".to_owned(),
        credentials,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    let rendered = render_opencode_config(
        &[provider],
        &cipher,
        ModelSelection {
            primary_provider_key: Some("openai"),
            primary_model_id: Some("gpt-5.5"),
            small_provider_key: None,
            small_model_id: None,
        },
    )
    .expect("render config");

    assert!(!rendered.env.contains_key("OPENAI_API_KEY"));
    assert_eq!(
        rendered
            .providers
            .get("openai")
            .map(|provider| provider.options.is_empty()),
        Some(true)
    );
    let auth_content: serde_json::Value = serde_json::from_str(
        rendered
            .opencode_auth_content
            .as_deref()
            .expect("auth content"),
    )
    .expect("auth content json");
    assert_eq!(auth_content["openai"]["type"], "oauth");
    assert_eq!(auth_content["openai"]["accountId"], "acct");
}

#[test]
fn render_opencode_config_skips_unsupported_oauth_providers() {
    let cipher = test_cipher();
    let credentials = encrypted_oauth_credentials(
        &OAuthCredential {
            provider: "future_provider".to_owned(),
            account_id: Some("acct".to_owned()),
            email: None,
            expires: 1782942233000,
            refresh: "refresh".to_owned(),
            access: "access".to_owned(),
        },
        &cipher,
    )
    .expect("encrypt oauth");
    let unsupported = ModelProviderConfig {
        id: Uuid::new_v4(),
        team_id: Uuid::new_v4(),
        provider_key: "future-provider".to_owned(),
        display_name: "Future Provider".to_owned(),
        credentials,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    let rendered = render_opencode_config(
        &[unsupported],
        &cipher,
        ModelSelection {
            primary_provider_key: None,
            primary_model_id: None,
            small_provider_key: None,
            small_model_id: None,
        },
    )
    .expect("render config");

    assert!(rendered.providers.is_empty());
    assert!(rendered.opencode_auth_content.is_none());
}

#[test]
fn render_omp_config_extracts_env_and_models() {
    let provider = ModelProviderConfig {
        id: Uuid::new_v4(),
        team_id: Uuid::new_v4(),
        provider_key: "anthropic".to_owned(),
        display_name: "Anthropic".to_owned(),
        credentials: serde_json::json!({"ANTHROPIC_API_KEY": "secret"}),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    let rendered = render_agent_config(
        AgentBackend::OmpRpc,
        &[provider],
        &test_cipher(),
        ModelSelection {
            primary_provider_key: Some("anthropic"),
            primary_model_id: Some("claude-sonnet-4-5"),
            small_provider_key: None,
            small_model_id: Some("claude-haiku-4-5"),
        },
    )
    .expect("render OMP config");

    assert_eq!(
        rendered.agent_config,
        AgentConfigPayload::OmpRpc { config_yml: None }
    );
    assert_eq!(
        rendered.env.get("ANTHROPIC_API_KEY"),
        Some(&"secret".to_owned())
    );
    assert_eq!(
        rendered.env.get("PI_PROVIDER"),
        Some(&"anthropic".to_owned())
    );
    assert_eq!(
        rendered.env.get("PI_MODEL"),
        Some(&"claude-sonnet-4-5".to_owned())
    );
    assert_eq!(
        rendered.env.get("PI_SMALL_MODEL"),
        Some(&"claude-haiku-4-5".to_owned())
    );
}

#[test]
fn render_omp_config_maps_openai_oauth_provider_for_omp() {
    let cipher = test_cipher();
    let provider = ModelProviderConfig {
        id: Uuid::new_v4(),
        team_id: Uuid::new_v4(),
        provider_key: "openai".to_owned(),
        display_name: "OpenAI".to_owned(),
        credentials: encrypted_oauth_credentials(
            &OAuthCredential {
                provider: "openai_chatgpt".to_owned(),
                account_id: Some("acct".to_owned()),
                email: Some("dev@example.com".to_owned()),
                expires: Utc::now().timestamp_millis() + 3_600_000,
                refresh: "refresh-secret".to_owned(),
                access: "access-secret".to_owned(),
            },
            &cipher,
        )
        .expect("oauth credentials"),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    let rendered = render_agent_config(
        AgentBackend::OmpRpc,
        &[provider],
        &cipher,
        ModelSelection {
            primary_provider_key: Some("openai"),
            primary_model_id: Some("gpt-5-codex"),
            small_provider_key: None,
            small_model_id: None,
        },
    )
    .expect("render OMP config");

    assert_eq!(
        rendered.env.get("OPENAI_CODEX_OAUTH_TOKEN"),
        Some(&"access-secret".to_owned())
    );
    assert!(!rendered.env.contains_key("OPENAI_API_KEY"));
    assert_eq!(
        rendered.env.get("PI_PROVIDER"),
        Some(&"openai-codex".to_owned())
    );
    assert_eq!(
        rendered.env.get("PI_MODEL"),
        Some(&"gpt-5-codex".to_owned())
    );
}

#[test]
fn render_omp_config_maps_openai_oauth_provider_without_access_token() {
    let cipher = test_cipher();
    let provider = ModelProviderConfig {
        id: Uuid::new_v4(),
        team_id: Uuid::new_v4(),
        provider_key: "openai".to_owned(),
        display_name: "OpenAI".to_owned(),
        credentials: encrypted_oauth_credentials(
            &OAuthCredential {
                provider: "openai_chatgpt".to_owned(),
                account_id: Some("acct".to_owned()),
                email: Some("dev@example.com".to_owned()),
                expires: Utc::now().timestamp_millis() + 3_600_000,
                refresh: "refresh-secret".to_owned(),
                access: String::new(),
            },
            &cipher,
        )
        .expect("oauth credentials"),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    let rendered = render_agent_config(
        AgentBackend::OmpRpc,
        &[provider],
        &cipher,
        ModelSelection {
            primary_provider_key: Some("openai"),
            primary_model_id: Some("gpt-5-codex"),
            small_provider_key: None,
            small_model_id: None,
        },
    )
    .expect("render OMP config");

    assert!(!rendered.env.contains_key("OPENAI_CODEX_OAUTH_TOKEN"));
    assert_eq!(
        rendered.env.get("PI_PROVIDER"),
        Some(&"openai-codex".to_owned())
    );
    assert_eq!(
        rendered.env.get("PI_MODEL"),
        Some(&"gpt-5-codex".to_owned())
    );
}

fn test_cipher() -> SecretCipher {
    SecretCipher::new("AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=").expect("test cipher")
}
