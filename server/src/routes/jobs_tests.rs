use std::sync::Arc;

use actix_web::{test, web, App};
use serde_json::json;
use uuid::Uuid;

use crate::app_state::AppState;
use crate::routes;
use crate::services::dispatcher::flag_store::InMemoryDispatchStore;
use crate::test_helpers;

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
        stale_worker_threshold_secs: 120,
        instance_password: "test-password".to_owned(),
        kaneo_instance: "cloud.kaneo.app".to_owned(),
        kaneo_api_key: String::new(),
        redis_url: String::new(),
    };

    let workers_repo = crate::services::workers::repository::WorkersRepository::new();
    let work_runs_repo = crate::services::work_runs::repository::WorkRunsRepository::new();
    let project_configs_repo =
        crate::services::project_configs::repository::ProjectConfigsRepository::new();
    let dispatch_store = Arc::new(InMemoryDispatchStore::default());

    AppState {
        auth: crate::services::auth::service::AuthService::new(
            crate::services::users::service::UsersService::new(
                crate::services::users::repository::UsersRepository::new(),
                pool.clone(),
            ),
            "test-password".to_owned(),
            "test-secret".to_owned(),
        ),
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
        kaneo,
        work_runs: work_runs_repo,
        dispatch_store,
        jwt_secret: cfg.jwt_secret.clone(),
    }
}

fn build_worker_token(worker_id: Uuid) -> String {
    let exp = chrono::Utc::now() + chrono::Duration::minutes(15);
    let claims = json!({"sub": worker_id.to_string(), "exp": exp.timestamp()});
    let token = jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret("test-secret".as_bytes()),
    )
    .expect("should build token");
    format!("Bearer {token}")
}

#[sqlx::test]
async fn poll_returns_204_when_no_dispatch(pool: sqlx::PgPool) {
    let state = build_state(pool.clone());
    let worker_id = test_helpers::insert_worker(&pool, "test-poll-noop").await;

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(routes::configure),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/v1/poll")
        .insert_header(("Authorization", build_worker_token(worker_id).as_str()))
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), 204);
}

#[sqlx::test]
async fn poll_returns_job_id_when_dispatched(pool: sqlx::PgPool) {
    let state = build_state(pool.clone());
    let worker_id = test_helpers::insert_worker(&pool, "test-poll-work").await;
    let project_id = test_helpers::insert_project_config(&pool, "kaneo-poll-test").await;
    let wr_id = test_helpers::insert_pending_work_run(&pool, project_id, "task-poll-test").await;

    state
        .dispatch_store
        .set_dispatched(worker_id, wr_id)
        .await
        .expect("Should set dispatched");

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(routes::configure),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/v1/poll")
        .insert_header(("Authorization", build_worker_token(worker_id).as_str()))
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["job_id"], wr_id.to_string());
}

#[sqlx::test]
async fn get_job_returns_200(pool: sqlx::PgPool) {
    let state = build_state(pool.clone());
    let worker_id = test_helpers::insert_worker(&pool, "test-getter").await;
    let project_id = test_helpers::insert_project_config(&pool, "kaneo-get-test").await;
    let wr_id = test_helpers::insert_pending_work_run(&pool, project_id, "task-get-test").await;

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(routes::configure),
    )
    .await;

    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/jobs/{wr_id}"))
        .insert_header(("Authorization", build_worker_token(worker_id).as_str()))
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["external_task_ref"], "task-get-test");
    assert_eq!(body["prompt_text"], "Review the PR");
    assert_eq!(body["kaneo_instance"], "cloud.kaneo.app");
    assert_eq!(body["repo_url"], "");
}

#[sqlx::test]
async fn get_job_returns_404_for_missing(pool: sqlx::PgPool) {
    let state = build_state(pool.clone());
    let worker_id = test_helpers::insert_worker(&pool, "test-missing-getter").await;

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(routes::configure),
    )
    .await;

    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/jobs/{}", Uuid::new_v4()))
        .insert_header(("Authorization", build_worker_token(worker_id).as_str()))
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), 404);
}

#[sqlx::test]
async fn ack_job_returns_200(pool: sqlx::PgPool) {
    let state = build_state(pool.clone());
    let worker_id = test_helpers::insert_worker(&pool, "test-acker").await;
    let project_id = test_helpers::insert_project_config(&pool, "kaneo-ack-test").await;
    let wr_id = test_helpers::insert_pending_work_run(&pool, project_id, "task-ack-test").await;

    let dispatch_repo = crate::services::dispatcher::repository::DispatchRepository::new();
    dispatch_repo
        .dispatch_to_worker(&pool, wr_id, worker_id)
        .await
        .expect("Should dispatch");

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(routes::configure),
    )
    .await;

    let req = test::TestRequest::post()
        .uri(&format!("/api/v1/jobs/{wr_id}/ack"))
        .insert_header(("Authorization", build_worker_token(worker_id).as_str()))
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["status"], "running");
    assert_eq!(body["worker_id"], worker_id.to_string());
}

#[sqlx::test]
async fn ack_job_returns_409_when_already_claimed(pool: sqlx::PgPool) {
    let state = build_state(pool.clone());
    let worker_a = test_helpers::insert_worker(&pool, "test-acker-a").await;
    let worker_b = test_helpers::insert_worker(&pool, "test-acker-b").await;
    let project_id = test_helpers::insert_project_config(&pool, "kaneo-ack-race").await;
    let wr_id = test_helpers::insert_pending_work_run(&pool, project_id, "task-ack-race").await;

    let dispatch_repo = crate::services::dispatcher::repository::DispatchRepository::new();
    dispatch_repo
        .dispatch_to_worker(&pool, wr_id, worker_a)
        .await
        .expect("Should dispatch");

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(routes::configure),
    )
    .await;

    let req_a = test::TestRequest::post()
        .uri(&format!("/api/v1/jobs/{wr_id}/ack"))
        .insert_header(("Authorization", build_worker_token(worker_a).as_str()))
        .to_request();
    let resp_a = test::call_service(&app, req_a).await;
    assert_eq!(resp_a.status(), 200);

    let req_b = test::TestRequest::post()
        .uri(&format!("/api/v1/jobs/{wr_id}/ack"))
        .insert_header(("Authorization", build_worker_token(worker_b).as_str()))
        .to_request();
    let resp_b = test::call_service(&app, req_b).await;
    assert_eq!(resp_b.status(), 409);
}

#[sqlx::test]
async fn submit_result_returns_200_on_completed(pool: sqlx::PgPool) {
    let state = build_state(pool.clone());
    let worker_id = test_helpers::insert_worker(&pool, "test-result").await;
    let project_id = test_helpers::insert_project_config(&pool, "kaneo-result-test").await;
    let wr_id = test_helpers::insert_pending_work_run(&pool, project_id, "task-result-test").await;

    let dispatch_repo = crate::services::dispatcher::repository::DispatchRepository::new();
    dispatch_repo
        .dispatch_to_worker(&pool, wr_id, worker_id)
        .await
        .expect("Should dispatch");

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(routes::configure),
    )
    .await;

    let ack_req = test::TestRequest::post()
        .uri(&format!("/api/v1/jobs/{wr_id}/ack"))
        .insert_header(("Authorization", build_worker_token(worker_id).as_str()))
        .to_request();
    test::call_service(&app, ack_req).await;

    let req = test::TestRequest::post()
        .uri(&format!("/api/v1/jobs/{wr_id}/result"))
        .insert_header(("Authorization", build_worker_token(worker_id).as_str()))
        .set_json(json!({
            "pr_url": "https://github.com/test/pr/1",
            "exit_code": 0,
            "tokens_used": 1000,
            "duration_ms": 60000,
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["status"], "completed");
    assert_eq!(body["result_pr_url"], "https://github.com/test/pr/1");
    assert_eq!(body["result_exit_code"], 0);
    assert_eq!(body["tokens_used"], 1000);
    assert_eq!(body["duration_ms"], 60000);
}

#[sqlx::test]
async fn submit_result_returns_409_when_not_running(pool: sqlx::PgPool) {
    let state = build_state(pool.clone());
    let worker_id = test_helpers::insert_worker(&pool, "test-early-result").await;
    let project_id = test_helpers::insert_project_config(&pool, "kaneo-early-result").await;
    let wr_id = test_helpers::insert_pending_work_run(&pool, project_id, "task-early-result").await;

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(routes::configure),
    )
    .await;

    let req = test::TestRequest::post()
        .uri(&format!("/api/v1/jobs/{wr_id}/result"))
        .insert_header(("Authorization", build_worker_token(worker_id).as_str()))
        .set_json(json!({
            "pr_url": "",
            "exit_code": 0,
            "tokens_used": 0,
            "duration_ms": 0,
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), 409);
}

#[sqlx::test]
async fn poll_rejects_missing_auth(pool: sqlx::PgPool) {
    let state = build_state(pool);

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(routes::configure),
    )
    .await;

    let req = test::TestRequest::get().uri("/api/v1/poll").to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), 401);
}

#[sqlx::test]
async fn poll_rejects_invalid_token(pool: sqlx::PgPool) {
    let state = build_state(pool);

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(routes::configure),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/v1/poll")
        .insert_header(("Authorization", "Bearer garbage"))
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), 401);
}
