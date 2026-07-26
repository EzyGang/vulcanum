use super::*;

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
