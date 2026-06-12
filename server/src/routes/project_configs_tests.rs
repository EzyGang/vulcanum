use actix_web::{test, web, App};
use uuid::Uuid;

use crate::app_state::AppState;
use crate::routes;
use crate::test_helpers;

const TEST_PASSWORD: &str = "test-password";

fn build_state(pool: sqlx::PgPool) -> AppState {
    test_helpers::build_state(pool)
}

fn auth_header(token: &str) -> (&str, String) {
    ("Authorization", format!("Bearer {token}"))
}

async fn insert_config(pool: &sqlx::PgPool, external_project_id: &str) -> Uuid {
    let id = Uuid::new_v4();

    sqlx::query!(
        "INSERT INTO project_configs (id, team_id, external_project_id, prompt_template, integration_type) VALUES ($1, $2, $3, $4, 'kaneo')",
        id,
        test_helpers::DEFAULT_TEAM_ID,
        external_project_id,
        "Review {{task_title}}",
    )
    .execute(pool)
    .await
    .expect("Should insert test config");

    id
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
    assert_eq!(body["external_project_id"], "test-get");
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
