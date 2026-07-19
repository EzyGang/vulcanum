use jsonwebtoken::{decode, DecodingKey, Validation};

use crate::config::AppConfig;
use crate::db::auth::AuthRepository;
use crate::db::teams::TeamsRepository;
use crate::db::users::UsersRepository;
use crate::models::auth::errors::AuthError;
use crate::models::auth::model::{LoginRequest, VerifyQuery};
use crate::services::auth::service::github_oauth::validate_return_to;
use crate::services::auth::service::AuthService;
use crate::services::teams::service::TeamsService;
use crate::services::users::service::UsersService;

#[test]
fn validates_safe_oauth_return_paths() {
    assert_eq!(
        validate_return_to(Some("/invites/token")),
        Some("/invites/token")
    );
    assert_eq!(
        validate_return_to(Some("/invites/token?source=login")),
        Some("/invites/token?source=login")
    );
}

#[test]
fn rejects_unsafe_oauth_return_paths() {
    assert_eq!(validate_return_to(None), None);
    assert_eq!(
        validate_return_to(Some("https://evil.test/invites/token")),
        None
    );
    assert_eq!(validate_return_to(Some("//evil.test/invites/token")), None);
    assert_eq!(validate_return_to(Some("invites/token")), None);
    assert_eq!(validate_return_to(Some("/../api/some-endpoint")), None);
    assert_eq!(validate_return_to(Some("/invites/../settings")), None);
    assert_eq!(
        validate_return_to(Some("/invites/token?next=..")),
        Some("/invites/token?next=..")
    );
    assert_eq!(validate_return_to(Some("/\\evil.test")), None);
    assert_eq!(validate_return_to(Some("/invites/token#fragment")), None);
    assert_eq!(
        validate_return_to(Some("/invites/token\nLocation: //evil.test")),
        None
    );
}

#[sqlx::test]
async fn verify_magic_link_returns_refreshable_token_pair(pool: sqlx::PgPool) {
    let service = auth_service(pool).await;
    let magic_token = service
        .login(LoginRequest {
            email: "magic-link-user@example.com".to_owned(),
        })
        .await
        .expect("create magic link");

    let (user, token_pair) = service
        .verify(VerifyQuery { token: magic_token })
        .await
        .expect("verify magic link");

    assert_eq!(user.email, "magic-link-user@example.com");
    assert!(!token_pair.access_token.is_empty());
    assert!(!token_pair.refresh_token.is_empty());
    assert!(token_pair.refresh_expires_at > chrono::Utc::now());

    let refreshed = service
        .refresh_user_token(&token_pair.refresh_token)
        .await
        .expect("refresh token pair issued by verify");
    assert!(!refreshed.access_token.is_empty());
    assert_ne!(refreshed.refresh_token, token_pair.refresh_token);
    assert!(refreshed.refresh_expires_at > chrono::Utc::now());
}

#[sqlx::test]
async fn instance_login_returns_refreshable_fifteen_minute_access_token(pool: sqlx::PgPool) {
    let service = auth_service(pool).await;
    let token_pair = service
        .instance_login("test-password")
        .await
        .expect("instance login");
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
    assert!(token_pair.refresh_expires_at > chrono::Utc::now() + chrono::Duration::hours(23));
}

#[sqlx::test]
async fn instance_refresh_rotates_once_and_rejects_password_change(pool: sqlx::PgPool) {
    let service = auth_service(pool.clone()).await;
    let initial = service
        .instance_login("test-password")
        .await
        .expect("instance login");
    let refreshed = service
        .refresh_token(&initial.refresh_token)
        .await
        .expect("refresh instance session");

    assert_ne!(refreshed.access_token, initial.access_token);
    assert_ne!(refreshed.refresh_token, initial.refresh_token);
    assert!(matches!(
        service.refresh_token(&initial.refresh_token).await,
        Err(AuthError::InvalidRefreshToken)
    ));

    let changed_password_service =
        auth_service_with_password(pool.clone(), "changed-password").await;
    assert!(matches!(
        changed_password_service
            .refresh_token(&refreshed.refresh_token)
            .await,
        Err(AuthError::InvalidRefreshToken)
    ));
    let restored_password_service = auth_service_with_password(pool, "test-password").await;
    assert!(matches!(
        restored_password_service
            .refresh_token(&refreshed.refresh_token)
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

async fn auth_service(pool: sqlx::PgPool) -> AuthService {
    auth_service_with_password(pool, "test-password").await
}

async fn auth_service_with_password(pool: sqlx::PgPool, password: &str) -> AuthService {
    let cfg = AppConfig {
        db_url: String::new(),
        max_conns: 1,
        poll_period_secs: 30,
        jwt_secret: "test-secret".to_owned(),
        stale_worker_threshold_secs: 120,
        unhealthy_threshold: 3,
        stalled_running_threshold_secs: 1800,
        instance_password: password.to_owned(),
        is_single_user: true,
        redis_url: "redis://127.0.0.1:6379".to_owned(),
        model_provider_secret_key: "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=".to_owned(),
        github_app_id: None,
        github_app_private_key: None,
        github_app_slug: None,
        github_webhook_secret: None,
        github_oauth_client_id: None,
        github_oauth_client_secret: None,
        github_oauth_redirect_url: None,
    };

    AuthService::new(
        AuthRepository::new(),
        pool.clone(),
        UsersService::new(UsersRepository::new(), pool.clone()),
        TeamsService::new(TeamsRepository::new(), pool),
        password.to_owned(),
        "test-secret".to_owned(),
        &cfg,
    )
    .expect("build auth service")
}
