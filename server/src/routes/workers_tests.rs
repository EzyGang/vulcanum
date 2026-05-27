use actix_web::{test, web, App};

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

#[sqlx::test]
async fn generate_code_returns_201(pool: sqlx::PgPool) {
    let state = build_state(pool);
    let token = state.auth.instance_login(TEST_PASSWORD).unwrap();

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(routes::configure),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/api/v1/workers/codes")
        .insert_header(auth_header(&token))
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), 201);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body["code"].as_str().unwrap().len() == 16);
    assert!(body["expires_at"].is_string());
}

#[sqlx::test]
async fn connect_with_valid_code_returns_200(pool: sqlx::PgPool) {
    let state = build_state(pool);
    let code = state
        .workers
        .generate_code()
        .await
        .expect("should generate");

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(routes::configure),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/api/v1/workers/connect")
        .set_json(serde_json::json!({
            "code": code.code,
            "worker_name": "handler-test"
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body["access_token"].is_string());
    assert!(body["expires_at"].is_string());
    assert_eq!(body["name"], "handler-test");
}

#[sqlx::test]
async fn connect_with_invalid_code_returns_400(pool: sqlx::PgPool) {
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(build_state(pool)))
            .configure(routes::configure),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/api/v1/workers/connect")
        .set_json(serde_json::json!({
            "code": "nope",
            "worker_name": "x"
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), 400);
}

#[sqlx::test]
async fn refresh_with_valid_token_returns_200(pool: sqlx::PgPool) {
    let state = build_state(pool);
    let code = state
        .workers
        .generate_code()
        .await
        .expect("should generate");
    let connect = state
        .workers
        .connect(crate::services::workers::model::ConnectRequest {
            code: code.code,
            worker_name: "rt-handler".to_owned(),
        })
        .await
        .unwrap();

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(routes::configure),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/api/v1/workers/refresh")
        .set_json(serde_json::json!({
            "refresh_token": connect.refresh_token
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body["access_token"].is_string());
    assert!(body["refresh_token"].is_string());
    assert!(body["expires_at"].is_string());
}

#[sqlx::test]
async fn refresh_with_invalid_token_returns_401(pool: sqlx::PgPool) {
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(build_state(pool)))
            .configure(routes::configure),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/api/v1/workers/refresh")
        .set_json(serde_json::json!({
            "refresh_token": "bad-token"
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), 401);
}

#[sqlx::test]
async fn delete_worker_returns_204(pool: sqlx::PgPool) {
    let state = build_state(pool);
    let code = state
        .workers
        .generate_code()
        .await
        .expect("should generate");
    let connect = state
        .workers
        .connect(crate::services::workers::model::ConnectRequest {
            code: code.code,
            worker_name: "delete-me".to_owned(),
        })
        .await
        .unwrap();
    let worker_id = connect.worker_id;

    let token = state.auth.instance_login(TEST_PASSWORD).unwrap();

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(routes::configure),
    )
    .await;

    let req = test::TestRequest::delete()
        .uri(&format!("/api/v1/workers/{worker_id}"))
        .insert_header(auth_header(&token))
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), 204);
}

#[sqlx::test]
async fn list_workers_returns_200(pool: sqlx::PgPool) {
    test_helpers::insert_worker(&pool, "list-test-1").await;
    test_helpers::insert_worker(&pool, "list-test-2").await;

    let state = build_state(pool);
    let token = state.auth.instance_login(TEST_PASSWORD).unwrap();

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(routes::configure),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/v1/workers")
        .insert_header(auth_header(&token))
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), 200);

    let body: Vec<serde_json::Value> = test::read_body_json(resp).await;
    assert!(body.len() >= 2);
}
