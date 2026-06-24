use chrono::Utc;
use serde_json::json;

use crate::services::model_providers::errors::ModelProvidersError;
use crate::services::model_providers::model::{
    CreateModelProviderRequest, AUTH_TYPE_API_KEY, OPENAI_PROVIDER_KEY,
};
use crate::services::model_providers::service::oauth_client::{
    DevicePollOutcome, OAuthRefreshTokenResponse,
};
use crate::services::model_providers::service_chatgpt_oauth_test_support::{
    complete_auth, service_with_oauth, service_with_oauth_client, service_with_oauth_options,
};
use crate::test_helpers::insert_team;

#[sqlx::test]
async fn selected_auth_material_refreshes_expired_chatgpt_oauth(pool: sqlx::PgPool) {
    let team_id = insert_team(&pool, "Refresh ChatGPT Auth Team").await;
    let service = service_with_oauth_options(
        pool,
        vec![DevicePollOutcome::Authorized("auth-code".to_owned())],
        Some(-60),
        vec![OAuthRefreshTokenResponse {
            access_token: "refreshed-access-token".to_owned(),
            refresh_token: Some("new-refresh-token".to_owned()),
            expires_in: Some(3600),
            id_token: None,
        }],
    )
    .await;
    let provider = complete_auth(&service, team_id).await;

    let selected = service
        .selected_auth_material(team_id, Some(provider.id), None)
        .await
        .expect("Should select refreshed auth material");
    let auth_content = selected
        .opencode_auth_content
        .expect("Should render OpenCode auth content");
    let auth: serde_json::Value =
        serde_json::from_str(&auth_content).expect("auth content should be valid json");

    assert_eq!(auth["openai"]["access"], "refreshed-access-token");
    assert_eq!(auth["openai"]["refresh"], "new-refresh-token");
    assert!(
        auth["openai"]["expires"]
            .as_i64()
            .expect("expires should be an integer")
            > Utc::now().timestamp_millis()
    );
}

#[sqlx::test]
async fn selected_auth_material_rejects_mixed_openai_auth_modes(pool: sqlx::PgPool) {
    let team_id = insert_team(&pool, "Mixed OpenAI Auth Team").await;
    let service = service_with_oauth(
        pool,
        vec![DevicePollOutcome::Authorized("auth-code".to_owned())],
    )
    .await;
    let oauth_provider = complete_auth(&service, team_id).await;
    let api_provider = service
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
        .expect("Should create API key provider");

    let result = service
        .selected_auth_material(team_id, Some(oauth_provider.id), Some(api_provider.id))
        .await;

    match result {
        Err(ModelProvidersError::InvalidSelection(message)) => {
            assert_eq!(message, "openai cannot use multiple auth modes in one job");
        }
        _ => panic!("Expected mixed auth mode selection error"),
    }
}

#[sqlx::test]
async fn selected_auth_material_errors_when_chatgpt_credentials_missing(pool: sqlx::PgPool) {
    let team_id = insert_team(&pool, "Missing OAuth Credentials Team").await;
    let service = service_with_oauth(
        pool.clone(),
        vec![DevicePollOutcome::Authorized("auth-code".to_owned())],
    )
    .await;
    let provider = complete_auth(&service, team_id).await;
    sqlx::query!(
        "UPDATE model_provider_configs SET oauth_credentials = NULL WHERE id = $1",
        provider.id,
    )
    .execute(&pool)
    .await
    .expect("Should remove OAuth credentials");

    let result = service
        .selected_auth_material(team_id, Some(provider.id), None)
        .await;

    match result {
        Err(ModelProvidersError::InvalidSelection(message)) => {
            assert_eq!(message, "ChatGPT OAuth provider is missing credentials");
        }
        _ => panic!("Expected missing credentials error"),
    }
}

#[sqlx::test]
async fn selected_auth_material_returns_refresh_error(pool: sqlx::PgPool) {
    let team_id = insert_team(&pool, "Refresh Error ChatGPT Auth Team").await;
    let service = service_with_oauth_client(
        pool,
        vec![DevicePollOutcome::Authorized("auth-code".to_owned())],
        Some(-60),
        Vec::new(),
        Some("refresh failed".to_owned()),
    )
    .await;
    let provider = complete_auth(&service, team_id).await;

    let result = service
        .selected_auth_material(team_id, Some(provider.id), None)
        .await;

    match result {
        Err(ModelProvidersError::OAuth(message)) => assert_eq!(message, "refresh failed"),
        _ => panic!("Expected refresh error"),
    }
}
