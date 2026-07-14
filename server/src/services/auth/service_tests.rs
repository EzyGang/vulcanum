use crate::config::AppConfig;
use crate::db::auth::AuthRepository;
use crate::db::teams::TeamsRepository;
use crate::db::users::UsersRepository;
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

async fn auth_service(pool: sqlx::PgPool) -> AuthService {
    let cfg = AppConfig {
        db_url: String::new(),
        max_conns: 1,
        poll_period_secs: 30,
        jwt_secret: "test-secret".to_owned(),
        stale_worker_threshold_secs: 120,
        unhealthy_threshold: 3,
        stalled_running_threshold_secs: 1800,
        instance_password: "test-password".to_owned(),
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
        "test-password".to_owned(),
        "test-secret".to_owned(),
        &cfg,
    )
    .expect("build auth service")
}
