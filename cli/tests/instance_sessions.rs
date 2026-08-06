#![cfg(not(windows))]

use std::fs;
use std::net::TcpListener;

use actix_web::{web, App, HttpResponse, HttpServer};
use chrono::{Duration, Utc};
use tempfile::TempDir;
use tokio::process::Command;
use uuid::Uuid;
use vulcanum_server::config::AppConfig;
use vulcanum_server::db::auth::AuthRepository;
use vulcanum_server::db::github_app::GithubAppRepository;
use vulcanum_server::db::teams::TeamsRepository;
use vulcanum_server::db::users::UsersRepository;
use vulcanum_server::models::auth::model::RefreshRequest;
use vulcanum_server::services::auth::service::AuthService;
use vulcanum_server::services::teams::service::TeamsService;
use vulcanum_server::services::users::service::UsersService;
use vulcanum_shared::state::app::AppSession;

const TEAM_ID: Uuid = Uuid::from_u128(42);

#[sqlx::test(migrations = "../server/migrations")]
async fn persisted_session_older_than_one_day_refreshes_through_cli(pool: sqlx::PgPool) {
    let auth = auth_service(pool.clone());
    let initial = auth
        .instance_login("test-password")
        .await
        .expect("issue instance session");
    sqlx::query!(
        r#"UPDATE instance_refresh_tokens
           SET created_at = NOW() - INTERVAL '2 days',
               expires_at = NOW() + INTERVAL '28 days'"#
    )
    .execute(&pool)
    .await
    .expect("age instance refresh session");

    let listener = TcpListener::bind(("127.0.0.1", 0)).expect("bind integration server");
    let address = listener.local_addr().expect("read integration address");
    let server_auth = auth.clone();
    let server = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(server_auth.clone()))
            .route("/api/v1/auth/refresh", web::post().to(refresh))
            .route("/api/v1/teams/{team_id}", web::get().to(team))
            .route("/api/v1/workers", web::get().to(workers))
    })
    .workers(1)
    .listen(listener)
    .expect("listen for integration requests")
    .run();
    let server_handle = server.handle();
    let server_task = tokio::spawn(server);

    let home = TempDir::new().expect("create isolated CLI home");
    let state_path = home.path().join(".vulcanum/app.json");
    fs::create_dir_all(state_path.parent().expect("state path parent"))
        .expect("create CLI state directory");
    let initial_refresh_token = initial.refresh_token.clone();
    let session = AppSession {
        instance_url: format!("http://{address}"),
        access_token: initial.access_token,
        refresh_token: initial.refresh_token,
        refresh_expires_at: initial.refresh_expires_at - Duration::days(2),
        team_id: Some(TEAM_ID),
    };
    fs::write(
        &state_path,
        serde_json::to_vec_pretty(&session).expect("encode initial CLI session"),
    )
    .expect("persist initial CLI session");

    let output = Command::new(env!("CARGO_BIN_EXE_vulcanum"))
        .env("HOME", home.path())
        .args(["workers", "list"])
        .output()
        .await
        .expect("run authenticated CLI command");

    server_handle.stop(true).await;
    server_task
        .await
        .expect("join integration server")
        .expect("stop integration server");
    assert!(
        output.status.success(),
        "CLI command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8(output.stdout).expect("CLI output should be UTF-8"),
        format!("No workers found for team Core ({TEAM_ID}).\n")
    );

    let persisted: AppSession =
        serde_json::from_slice(&fs::read(&state_path).expect("read refreshed CLI session"))
            .expect("decode refreshed CLI session");
    assert_ne!(persisted.refresh_token, initial_refresh_token);
    assert!(persisted.refresh_expires_at > Utc::now() + Duration::days(29));
    drop(auth);
    pool.close().await;
}

async fn refresh(auth: web::Data<AuthService>, request: web::Json<RefreshRequest>) -> HttpResponse {
    match auth.refresh_token(&request.refresh_token).await {
        Ok(tokens) => HttpResponse::Ok().json(tokens),
        Err(_) => HttpResponse::Unauthorized().finish(),
    }
}

async fn team() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "id": TEAM_ID,
        "name": "Core",
        "primary_model_provider_key": null,
        "primary_model_id": null,
        "small_model_provider_key": null,
        "small_model_id": null,
    }))
}

async fn workers() -> HttpResponse {
    HttpResponse::Ok().json(Vec::<serde_json::Value>::new())
}

fn auth_service(pool: sqlx::PgPool) -> AuthService {
    let config = AppConfig {
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
        GithubAppRepository::new(),
        pool.clone(),
        UsersService::new(UsersRepository::new(), pool.clone()),
        TeamsService::new(TeamsRepository::new(), pool),
        &config,
    )
    .expect("build auth service")
}
