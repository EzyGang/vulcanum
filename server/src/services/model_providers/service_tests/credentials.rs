use super::*;

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
