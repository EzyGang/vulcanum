use chrono::{DateTime, Duration, Utc};
use jsonwebtoken::{decode, DecodingKey, Validation};

use crate::models::auth::errors::AuthError;

use super::{auth_service, auth_service_with_password};

#[sqlx::test]
async fn instance_login_issues_thirty_day_refresh_and_fifteen_minute_access(pool: sqlx::PgPool) {
    let service = auth_service(pool).await;
    let earliest_issue = Utc::now();
    let token_pair = service
        .instance_login("test-password")
        .await
        .expect("instance login");
    let latest_issue = Utc::now();
    let claims = decode::<serde_json::Value>(
        &token_pair.access_token,
        &DecodingKey::from_secret(b"test-secret"),
        &Validation::default(),
    )
    .expect("decode instance access token")
    .claims;

    assert_eq!(claims["sub"], "instance");
    assert_eq!(claims["typ"], "instance");
    assert_eq!(
        claims["exp"].as_i64().expect("numeric expiry")
            - claims["iat"].as_i64().expect("numeric issued-at"),
        15 * 60
    );
    assert!(token_pair.refresh_token.starts_with("irt1_"));
    assert_thirty_day_expiry(token_pair.refresh_expires_at, earliest_issue, latest_issue);
}

#[sqlx::test]
async fn instance_refresh_rotates_for_thirty_days_and_rejects_predecessor(pool: sqlx::PgPool) {
    let service = auth_service(pool).await;
    let initial = service
        .instance_login("test-password")
        .await
        .expect("instance login");
    let earliest_refresh = Utc::now();
    let refreshed = service
        .refresh_token(&initial.refresh_token)
        .await
        .expect("refresh instance session");
    let latest_refresh = Utc::now();

    assert_ne!(refreshed.access_token, initial.access_token);
    assert_ne!(refreshed.refresh_token, initial.refresh_token);
    assert_thirty_day_expiry(
        refreshed.refresh_expires_at,
        earliest_refresh,
        latest_refresh,
    );
    assert!(matches!(
        service.refresh_token(&initial.refresh_token).await,
        Err(AuthError::InvalidRefreshToken)
    ));
}

#[sqlx::test]
async fn instance_refresh_remains_valid_after_more_than_one_day(pool: sqlx::PgPool) {
    let service = auth_service(pool.clone()).await;
    let initial = service
        .instance_login("test-password")
        .await
        .expect("instance login");
    sqlx::query!(
        r#"UPDATE instance_refresh_tokens
           SET created_at = NOW() - INTERVAL '2 days',
               expires_at = NOW() + INTERVAL '28 days'"#
    )
    .execute(&pool)
    .await
    .expect("age instance refresh session");

    let refreshed = service
        .refresh_token(&initial.refresh_token)
        .await
        .expect("refresh session older than one day");

    assert_ne!(refreshed.refresh_token, initial.refresh_token);
}

#[sqlx::test]
async fn instance_refresh_rejects_expired_token(pool: sqlx::PgPool) {
    let service = auth_service(pool.clone()).await;
    let initial = service
        .instance_login("test-password")
        .await
        .expect("instance login");
    sqlx::query!(
        r#"UPDATE instance_refresh_tokens
           SET expires_at = NOW() - INTERVAL '1 second'"#
    )
    .execute(&pool)
    .await
    .expect("expire instance refresh session");

    assert!(matches!(
        service.refresh_token(&initial.refresh_token).await,
        Err(AuthError::InvalidRefreshToken)
    ));
}

#[sqlx::test]
async fn instance_refresh_rejects_token_after_logout(pool: sqlx::PgPool) {
    let service = auth_service(pool).await;
    let initial = service
        .instance_login("test-password")
        .await
        .expect("instance login");
    service
        .revoke_refresh_token(&initial.refresh_token)
        .await
        .expect("logout instance session");

    assert!(matches!(
        service.refresh_token(&initial.refresh_token).await,
        Err(AuthError::InvalidRefreshToken)
    ));
}

#[sqlx::test]
async fn instance_refresh_rejects_password_change_and_restoration(pool: sqlx::PgPool) {
    let service = auth_service(pool.clone()).await;
    let initial = service
        .instance_login("test-password")
        .await
        .expect("instance login");
    let changed_password_service =
        auth_service_with_password(pool.clone(), "changed-password").await;

    assert!(matches!(
        changed_password_service
            .refresh_token(&initial.refresh_token)
            .await,
        Err(AuthError::InvalidRefreshToken)
    ));
    let restored_password_service = auth_service_with_password(pool, "test-password").await;
    assert!(matches!(
        restored_password_service
            .refresh_token(&initial.refresh_token)
            .await,
        Err(AuthError::InvalidRefreshToken)
    ));
}

#[sqlx::test]
async fn concurrent_instance_refresh_allows_one_rotation(pool: sqlx::PgPool) {
    let service = auth_service(pool).await;
    let initial = service
        .instance_login("test-password")
        .await
        .expect("instance login");

    let (first, second) = tokio::join!(
        service.refresh_token(&initial.refresh_token),
        service.refresh_token(&initial.refresh_token)
    );
    let success_count = usize::from(first.is_ok()) + usize::from(second.is_ok());
    let invalid_count = usize::from(matches!(first, Err(AuthError::InvalidRefreshToken)))
        + usize::from(matches!(second, Err(AuthError::InvalidRefreshToken)));

    assert_eq!(success_count, 1);
    assert_eq!(invalid_count, 1);
}

#[sqlx::test]
async fn instance_login_rejects_wrong_password_without_creating_session(pool: sqlx::PgPool) {
    let service = auth_service(pool.clone()).await;

    assert!(matches!(
        service.instance_login("wrong-password").await,
        Err(AuthError::InvalidPassword)
    ));
    let row_count: i64 =
        sqlx::query_scalar!(r#"SELECT COUNT(*) AS "count!" FROM instance_refresh_tokens"#)
            .fetch_one(&pool)
            .await
            .expect("count instance refresh sessions");
    assert_eq!(row_count, 0);
}

fn assert_thirty_day_expiry(
    expires_at: DateTime<Utc>,
    earliest_issue: DateTime<Utc>,
    latest_issue: DateTime<Utc>,
) {
    assert!(expires_at >= earliest_issue + Duration::days(30));
    assert!(expires_at <= latest_issue + Duration::days(30));
}
