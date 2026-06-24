use crate::services::model_providers::model::{
    StartChatGptAuthRequest, UpdateModelProviderRequest,
};
use crate::services::model_providers::service::oauth_client::DevicePollOutcome;
use crate::services::model_providers::service::oauth_tokens::{extract_account_id, extract_email};
use crate::services::model_providers::service_chatgpt_oauth_test_support::{
    complete_auth, service_with_oauth, start_auth, unsigned_jwt,
};
use crate::test_helpers::insert_team;
use serde_json::json;
#[sqlx::test]
async fn chatgpt_auth_flow_encrypts_device_code_and_uses_display_name(pool: sqlx::PgPool) {
    let team_id = insert_team(&pool, "ChatGPT Auth Team").await;
    let service = service_with_oauth(
        pool.clone(),
        vec![
            DevicePollOutcome::Pending,
            DevicePollOutcome::Authorized("auth-code".to_owned()),
        ],
    )
    .await;

    let start = service
        .start_chatgpt_auth(
            team_id,
            "user-1",
            StartChatGptAuthRequest {
                display_name: "Custom ChatGPT".to_owned(),
            },
        )
        .await
        .expect("Should start device auth");
    let stored = sqlx::query!(
        r#"SELECT encrypted_device_code::text AS "encrypted_device_code!"
           FROM model_provider_auth_attempts WHERE id = $1"#,
        start.attempt_id,
    )
    .fetch_one(&pool)
    .await
    .expect("Should fetch auth attempt");
    assert!(!stored.encrypted_device_code.contains("device-secret"));

    let pending = service
        .chatgpt_auth_status(team_id, "user-1", start.attempt_id)
        .await
        .expect("Should poll pending status");
    assert_eq!(pending.status, "pending");
    assert!(pending.provider.is_none());
    let complete = service
        .chatgpt_auth_status(team_id, "user-1", start.attempt_id)
        .await
        .expect("Should complete device auth");
    let provider = complete.provider.expect("Should return connected provider");
    assert_eq!(complete.status, "complete");
    assert_eq!(provider.display_name, "Custom ChatGPT");
    assert!(provider.oauth_credentials.is_some());
}
#[sqlx::test]
async fn chatgpt_auth_slow_down_backs_off_poll_interval(pool: sqlx::PgPool) {
    let team_id = insert_team(&pool, "Slow Down ChatGPT Auth Team").await;
    let service = service_with_oauth(pool, vec![DevicePollOutcome::SlowDown]).await;
    let start = service
        .start_chatgpt_auth(
            team_id,
            "user-1",
            StartChatGptAuthRequest {
                display_name: String::new(),
            },
        )
        .await
        .expect("Should start device auth");
    let status = service
        .chatgpt_auth_status(team_id, "user-1", start.attempt_id)
        .await
        .expect("Should poll slow down status");
    assert_eq!(status.status, "pending");
    assert_eq!(status.poll_interval_seconds, Some(6));
}

#[sqlx::test]
async fn chatgpt_auth_failed_poll_marks_attempt_failed(pool: sqlx::PgPool) {
    let team_id = insert_team(&pool, "Failed ChatGPT Auth Team").await;
    let service = service_with_oauth(
        pool,
        vec![DevicePollOutcome::Failed("Access denied".to_owned())],
    )
    .await;
    let start = start_auth(&service, team_id).await;

    let status = service
        .chatgpt_auth_status(team_id, "user-1", start.attempt_id)
        .await
        .expect("Should poll failed status");

    assert_eq!(status.status, "failed");
    assert_eq!(status.error.as_deref(), Some("Access denied"));
    assert!(status.provider.is_none());
}

#[sqlx::test]
async fn chatgpt_auth_status_expires_stale_attempt(pool: sqlx::PgPool) {
    let team_id = insert_team(&pool, "Expired ChatGPT Auth Team").await;
    let service = service_with_oauth(pool.clone(), Vec::new()).await;
    let start = start_auth(&service, team_id).await;
    sqlx::query!(
        "UPDATE model_provider_auth_attempts SET expires_at = NOW() - INTERVAL '1 second' WHERE id = $1",
        start.attempt_id,
    )
    .execute(&pool)
    .await
    .expect("Should expire auth attempt");

    let status = service
        .chatgpt_auth_status(team_id, "user-1", start.attempt_id)
        .await
        .expect("Should return expired status");

    assert_eq!(status.status, "expired");
    assert_eq!(status.error.as_deref(), Some("Device login expired"));
    assert!(status.provider.is_none());
}

#[sqlx::test]
async fn start_chatgpt_auth_supersedes_existing_pending_attempt(pool: sqlx::PgPool) {
    let team_id = insert_team(&pool, "Superseded ChatGPT Auth Team").await;
    let service = service_with_oauth(pool, Vec::new()).await;
    let first = start_auth(&service, team_id).await;
    let second = start_auth(&service, team_id).await;

    let first_status = service
        .chatgpt_auth_status(team_id, "user-1", first.attempt_id)
        .await
        .expect("Should read superseded status");
    let second_status = service
        .chatgpt_auth_status(team_id, "user-1", second.attempt_id)
        .await
        .expect("Should read active status");

    assert_eq!(first_status.status, "failed");
    assert_eq!(
        first_status.error.as_deref(),
        Some("Superseded by a newer device login")
    );
    assert_eq!(second_status.status, "pending");
}

#[sqlx::test]
async fn chatgpt_oauth_rename_allows_empty_credentials_payload(pool: sqlx::PgPool) {
    let team_id = insert_team(&pool, "Rename ChatGPT Auth Team").await;
    let service = service_with_oauth(
        pool,
        vec![DevicePollOutcome::Authorized("auth-code".to_owned())],
    )
    .await;
    let provider = complete_auth(&service, team_id).await;

    let updated = service
        .update(
            provider.id,
            team_id,
            UpdateModelProviderRequest {
                display_name: Some("Renamed ChatGPT".to_owned()),
                credentials: Some(json!({})),
            },
        )
        .await
        .expect("Should rename OAuth provider");
    assert_eq!(updated.display_name, "Renamed ChatGPT");
}
#[sqlx::test]
async fn cancel_chatgpt_auth_marks_attempt_failed(pool: sqlx::PgPool) {
    let team_id = insert_team(&pool, "Cancel ChatGPT Auth Team").await;
    let service = service_with_oauth(pool, Vec::new()).await;
    let start = start_auth(&service, team_id).await;

    service
        .cancel_chatgpt_auth(team_id, "user-1", start.attempt_id)
        .await
        .expect("Should cancel device auth");
    service
        .cancel_chatgpt_auth(team_id, "user-1", start.attempt_id)
        .await
        .expect("Second cancel should be idempotent");
    let status = service
        .chatgpt_auth_status(team_id, "user-1", start.attempt_id)
        .await
        .expect("Should read cancelled status");
    assert_eq!(status.status, "failed");
    assert_eq!(status.error.as_deref(), Some("Device login cancelled"));
}
#[sqlx::test]
async fn cancel_chatgpt_auth_does_not_overwrite_complete_attempt(pool: sqlx::PgPool) {
    let team_id = insert_team(&pool, "Late Cancel ChatGPT Auth Team").await;
    let service = service_with_oauth(
        pool,
        vec![DevicePollOutcome::Authorized("auth-code".to_owned())],
    )
    .await;
    let start = start_auth(&service, team_id).await;
    service
        .chatgpt_auth_status(team_id, "user-1", start.attempt_id)
        .await
        .expect("Should complete auth");

    service
        .cancel_chatgpt_auth(team_id, "user-1", start.attempt_id)
        .await
        .expect("Late cancel should be idempotent");
    let status = service
        .chatgpt_auth_status(team_id, "user-1", start.attempt_id)
        .await
        .expect("Should read completed status");
    assert_eq!(status.status, "complete");
    assert!(status.provider.is_some());
}

#[test]
fn jwt_claim_extractors_read_chatgpt_account_id_fallback_and_email() {
    let chatgpt_account_token = unsigned_jwt(json!({
        "chatgpt_account_id": "acct_chatgpt",
        "account_id": "acct_fallback",
        "email": "user@example.com",
    }));
    let fallback_account_token = unsigned_jwt(json!({
        "account_id": "acct_fallback",
    }));

    assert_eq!(
        extract_account_id(&chatgpt_account_token).as_deref(),
        Some("acct_chatgpt")
    );
    assert_eq!(
        extract_account_id(&fallback_account_token).as_deref(),
        Some("acct_fallback")
    );
    assert_eq!(
        extract_email(&chatgpt_account_token).as_deref(),
        Some("user@example.com")
    );
    assert!(extract_account_id("not-a-jwt").is_none());
    assert!(extract_email("not-a-jwt").is_none());
}
