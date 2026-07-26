use super::*;

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
        rendered.env.get("VULCANUM_OMP_PROVIDER"),
        Some(&"anthropic".to_owned())
    );
    assert_eq!(
        rendered.env.get("VULCANUM_OMP_MODEL"),
        Some(&"claude-sonnet-4-5".to_owned())
    );
    assert_eq!(
        rendered.env.get("VULCANUM_OMP_SMOL"),
        Some(&"claude-haiku-4-5".to_owned())
    );
    assert!(!rendered.env.contains_key("PI_PROVIDER"));
    assert!(!rendered.env.contains_key("PI_MODEL"));
    assert!(!rendered.env.contains_key("PI_SMALL_MODEL"));
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
        rendered.env.get("VULCANUM_OMP_PROVIDER"),
        Some(&"openai-codex".to_owned())
    );
    assert_eq!(
        rendered.env.get("VULCANUM_OMP_MODEL"),
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
        rendered.env.get("VULCANUM_OMP_PROVIDER"),
        Some(&"openai-codex".to_owned())
    );
    assert_eq!(
        rendered.env.get("VULCANUM_OMP_MODEL"),
        Some(&"gpt-5-codex".to_owned())
    );
}
