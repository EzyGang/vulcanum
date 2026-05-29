use std::sync::Arc;

use actix_web::{test, web, App};
use uuid::Uuid;

use crate::app_state::AppState;
use crate::routes;
use crate::services::dispatcher::flag_store::InMemoryDispatchStore;
use crate::services::integrations::client::IntegrationClient;

const TEST_PASSWORD: &str = "test-password";

fn build_state(pool: sqlx::PgPool) -> AppState {
    let kaneo = IntegrationClient::new_kaneo("cloud.kaneo.app".to_owned(), String::new());

    let cfg = crate::config::AppConfig {
        db_url: String::new(),
        max_conns: 1,
        poll_period_secs: 30,
        jwt_secret: "test-secret".to_owned(),
        stale_worker_threshold_secs: 120,
        instance_password: TEST_PASSWORD.to_owned(),
        kaneo_instance: "cloud.kaneo.app".to_owned(),
        kaneo_api_key: String::new(),
        redis_url: String::new(),
    };

    let workers_repo = crate::services::workers::repository::WorkersRepository::new();
    let work_runs_repo = crate::services::work_runs::repository::WorkRunsRepository::new();
    let project_configs_repo =
        crate::services::project_configs::repository::ProjectConfigsRepository::new();
    let dispatch_store = Arc::new(InMemoryDispatchStore::default());

    let auth = crate::services::auth::service::AuthService::new(
        crate::services::users::service::UsersService::new(
            crate::services::users::repository::UsersRepository::new(),
            pool.clone(),
        ),
        TEST_PASSWORD.to_owned(),
        "test-secret".to_owned(),
    );

    AppState {
        auth,
        project_configs: crate::services::project_configs::service::ProjectConfigsService::new(
            project_configs_repo.clone(),
            pool.clone(),
            kaneo.clone(),
        ),
        workers: crate::services::workers::service::WorkersService::new(
            workers_repo.clone(),
            pool.clone(),
            &cfg,
            Arc::new(crate::services::workers::code_store::InMemoryCodeStore::new()),
        ),
        jobs: crate::services::work_runs::service::WorkRunsService::new(
            work_runs_repo.clone(),
            workers_repo,
            project_configs_repo,
            pool.clone(),
            dispatch_store.clone(),
            kaneo.clone(),
        ),
        db_pool: pool,
        integration: kaneo,
        work_runs: work_runs_repo,
        dispatch_store,
        jwt_secret: cfg.jwt_secret.clone(),
    }
}

async fn insert_config(pool: &sqlx::PgPool, kaneo_project_id: &str) -> Uuid {
    let id = Uuid::new_v4();

    sqlx::query!(
        "INSERT INTO project_configs (id, kaneo_project_id, prompt_template, integration_type) VALUES ($1, $2, $3, 'kaneo')",
        id,
        kaneo_project_id,
        "Review {{task_title}}",
    )
    .execute(pool)
    .await
    .expect("Should insert test config");

    id
}

fn auth_header(token: &str) -> (&str, String) {
    ("Authorization", format!("Bearer {token}"))
}

#[sqlx::test]
async fn list_returns_configs(pool: sqlx::PgPool) {
    insert_config(&pool, "test-list-1").await;
    insert_config(&pool, "test-list-2").await;

    let state = build_state(pool);
    let token = state.auth.instance_login(TEST_PASSWORD).unwrap();

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(routes::configure),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/v1/projects")
        .insert_header(auth_header(&token))
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert!(resp.status().is_success());

    let body: Vec<serde_json::Value> = test::read_body_json(resp).await;
    assert!(body.len() >= 2);
}

#[sqlx::test]
async fn get_returns_config(pool: sqlx::PgPool) {
    let id = insert_config(&pool, "test-get").await;

    let state = build_state(pool);
    let token = state.auth.instance_login(TEST_PASSWORD).unwrap();

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(routes::configure),
    )
    .await;

    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/projects/{id}"))
        .insert_header(auth_header(&token))
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert!(resp.status().is_success());

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["kaneo_project_id"], "test-get");
}

#[sqlx::test]
async fn get_nonexistent_returns_404(pool: sqlx::PgPool) {
    let nonexistent = Uuid::new_v4();

    let state = build_state(pool);
    let token = state.auth.instance_login(TEST_PASSWORD).unwrap();

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(routes::configure),
    )
    .await;

    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/projects/{nonexistent}"))
        .insert_header(auth_header(&token))
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), 404);
}

#[sqlx::test]
async fn delete_removes_config(pool: sqlx::PgPool) {
    let id = insert_config(&pool, "test-delete").await;

    let state = build_state(pool.clone());
    let token = state.auth.instance_login(TEST_PASSWORD).unwrap();

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(routes::configure),
    )
    .await;

    let req = test::TestRequest::delete()
        .uri(&format!("/api/v1/projects/{id}"))
        .insert_header(auth_header(&token))
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), 204);

    let row = sqlx::query!(
        "SELECT COUNT(*) as count FROM project_configs WHERE id = $1",
        id,
    )
    .fetch_one(&pool)
    .await
    .expect("Should query count");
    assert_eq!(row.count.unwrap(), 0);
}

#[sqlx::test]
async fn delete_nonexistent_returns_404(pool: sqlx::PgPool) {
    let nonexistent = Uuid::new_v4();

    let state = build_state(pool);
    let token = state.auth.instance_login(TEST_PASSWORD).unwrap();

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(routes::configure),
    )
    .await;

    let req = test::TestRequest::delete()
        .uri(&format!("/api/v1/projects/{nonexistent}"))
        .insert_header(auth_header(&token))
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), 404);
}
