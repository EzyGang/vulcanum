use actix_web::{test, web, App};
use uuid::Uuid;

use crate::app_state::AppState;
use crate::routes;

fn build_state(pool: sqlx::PgPool) -> AppState {
    let kaneo = crate::services::kaneo::client::KaneoClient::new(
        "cloud.kaneo.app".to_owned(),
        String::new(),
    );

    let cfg = crate::config::AppConfig {
        db_url: String::new(),
        max_conns: 1,
        poll_period_secs: 30,
        jwt_secret: "test-secret".to_owned(),
    };

    AppState {
        auth: crate::services::auth::service::AuthService::new(
            crate::services::users::service::UsersService::new(
                crate::services::users::repository::UsersRepository::new(),
                pool.clone(),
            ),
        ),
        project_configs: crate::services::project_configs::service::ProjectConfigsService::new(
            crate::services::project_configs::repository::ProjectConfigsRepository::new(),
            pool.clone(),
            kaneo.clone(),
        ),
        workers: crate::services::workers::service::WorkersService::new(
            crate::services::workers::repository::WorkersRepository::new(),
            pool.clone(),
            &cfg,
        ),
        db_pool: pool,
        kaneo,
        work_runs: crate::services::work_runs::repository::WorkRunsRepository::new(),
        work_notifier: crate::services::poller::notifier::WorkNotifier::new(),
    }
}

#[sqlx::test]
async fn generate_code_returns_201(pool: sqlx::PgPool) {
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(build_state(pool)))
            .configure(routes::configure),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/api/v1/workers/codes")
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
    let code = state.workers.generate_code().await;

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
    let code = state.workers.generate_code().await;
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
    let code = state.workers.generate_code().await;
    let connect = state
        .workers
        .connect(crate::services::workers::model::ConnectRequest {
            code: code.code,
            worker_name: "delete-me".to_owned(),
        })
        .await
        .unwrap();
    let worker_id = connect.worker_id;

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(routes::configure),
    )
    .await;

    let req = test::TestRequest::delete()
        .uri(&format!("/api/v1/workers/{worker_id}"))
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), 204);
}
