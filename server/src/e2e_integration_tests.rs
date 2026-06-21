use actix_web::{test, web, App};
use serde_json::json;
use uuid::Uuid;

use crate::routes;
use crate::services::dispatcher::repository::DispatchRepository;
use crate::test_helpers;

#[sqlx::test]
async fn connect_refresh_poll_no_jobs(pool: sqlx::PgPool) {
    let state = test_helpers::build_state(pool.clone()).await;
    let token = state.auth.instance_login("test-password").unwrap();

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state.clone()))
            .configure(routes::configure),
    )
    .await;

    let code_req = test::TestRequest::post()
        .uri("/api/v1/workers/codes")
        .insert_header(("Authorization", format!("Bearer {token}")))
        .to_request();
    let code_resp = test::call_service(&app, code_req).await;
    assert_eq!(code_resp.status(), 201);
    let code_body: serde_json::Value = test::read_body_json(code_resp).await;
    let code = code_body["code"].as_str().unwrap();

    let connect_req = test::TestRequest::post()
        .uri("/api/v1/workers/connect")
        .set_json(json!({"code": code, "worker_name": "e2e-worker"}))
        .to_request();
    let connect_resp = test::call_service(&app, connect_req).await;
    assert_eq!(connect_resp.status(), 200);
    let connect_body: serde_json::Value = test::read_body_json(connect_resp).await;
    let refresh_token = connect_body["refresh_token"].as_str().unwrap();
    let worker_id = connect_body["worker_id"].as_str().unwrap();
    let worker_uuid = Uuid::parse_str(worker_id).unwrap();

    let refresh_req = test::TestRequest::post()
        .uri("/api/v1/workers/refresh")
        .set_json(json!({"refresh_token": refresh_token}))
        .to_request();
    let refresh_resp = test::call_service(&app, refresh_req).await;
    assert_eq!(refresh_resp.status(), 200);
    let refresh_body: serde_json::Value = test::read_body_json(refresh_resp).await;
    let access_token = refresh_body["access_token"].as_str().unwrap();

    let poll_req = test::TestRequest::get()
        .uri("/api/v1/poll")
        .insert_header(("Authorization", format!("Bearer {access_token}")))
        .to_request();
    let poll_resp = test::call_service(&app, poll_req).await;
    assert_eq!(poll_resp.status(), 204);

    let row = sqlx::query!("SELECT last_seen FROM workers WHERE id = $1", worker_uuid)
        .fetch_one(&pool)
        .await
        .expect("Should query worker");
    assert!(
        row.last_seen.is_some(),
        "last_seen should be updated after poll"
    );
}

#[sqlx::test]
async fn full_job_lifecycle(pool: sqlx::PgPool) {
    let state = test_helpers::build_state(pool.clone()).await;
    let token = state.auth.instance_login("test-password").unwrap();

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state.clone()))
            .configure(routes::configure),
    )
    .await;

    let code_req = test::TestRequest::post()
        .uri("/api/v1/workers/codes")
        .insert_header(("Authorization", format!("Bearer {token}")))
        .to_request();
    let code_resp = test::call_service(&app, code_req).await;
    let code_body: serde_json::Value = test::read_body_json(code_resp).await;
    let code = code_body["code"].as_str().unwrap();

    let connect_req = test::TestRequest::post()
        .uri("/api/v1/workers/connect")
        .set_json(json!({"code": code, "worker_name": "lifecycle-worker"}))
        .to_request();
    let connect_resp = test::call_service(&app, connect_req).await;
    let connect_body: serde_json::Value = test::read_body_json(connect_resp).await;
    let access_token = connect_body["access_token"].as_str().unwrap();
    let worker_id = connect_body["worker_id"].as_str().unwrap();
    let worker_uuid = Uuid::parse_str(worker_id).unwrap();

    let project_id = test_helpers::insert_project_config(&pool, "kaneo-lifecycle").await;
    let wr_id = test_helpers::insert_pending_work_run(&pool, project_id, "task-lifecycle").await;

    let dispatch_repo = DispatchRepository;
    dispatch_repo
        .dispatch_to_worker(&pool, wr_id, worker_uuid)
        .await
        .expect("Should dispatch");

    state
        .dispatch_store
        .set_dispatched(worker_uuid, wr_id)
        .await
        .expect("Should set dispatched");

    let poll_req = test::TestRequest::get()
        .uri("/api/v1/poll")
        .insert_header(("Authorization", format!("Bearer {access_token}")))
        .to_request();
    let poll_resp = test::call_service(&app, poll_req).await;
    assert_eq!(poll_resp.status(), 200);
    let poll_body: serde_json::Value = test::read_body_json(poll_resp).await;
    assert_eq!(poll_body["job_id"], wr_id.to_string());

    let get_req = test::TestRequest::get()
        .uri(&format!("/api/v1/jobs/{wr_id}"))
        .insert_header(("Authorization", format!("Bearer {access_token}")))
        .to_request();
    let get_resp = test::call_service(&app, get_req).await;
    assert_eq!(get_resp.status(), 200);
    let get_body: serde_json::Value = test::read_body_json(get_resp).await;
    assert_eq!(get_body["external_task_ref"], "task-lifecycle");
    assert_eq!(get_body["prompt_text"], "Review the PR");

    let ack_req = test::TestRequest::post()
        .uri(&format!("/api/v1/jobs/{wr_id}/ack"))
        .insert_header(("Authorization", format!("Bearer {access_token}")))
        .set_json(json!({}))
        .to_request();
    let ack_resp = test::call_service(&app, ack_req).await;
    assert_eq!(ack_resp.status(), 200);
    let ack_body: serde_json::Value = test::read_body_json(ack_resp).await;
    assert_eq!(ack_body["status"], "running");

    let result_req = test::TestRequest::post()
        .uri(&format!("/api/v1/jobs/{wr_id}/result"))
        .insert_header(("Authorization", format!("Bearer {access_token}")))
        .set_json(json!({
            "pr_urls": ["https://github.com/test/pr/42"],
            "exit_code": 0,
            "tokens_used": 1234,
            "duration_ms": 30000,
            "input_tokens": 0,
            "output_tokens": 0,
            "cache_read_tokens": 0,
            "cache_write_tokens": 0,
            "model_used": null,
        }))
        .to_request();
    let result_resp = test::call_service(&app, result_req).await;
    assert_eq!(result_resp.status(), 200);
    let result_body: serde_json::Value = test::read_body_json(result_resp).await;
    assert_eq!(result_body["status"], "completed");
    assert_eq!(
        result_body["result_pr_url"],
        "https://github.com/test/pr/42"
    );
    assert_eq!(result_body["tokens_used"], 1234);
    assert_eq!(result_body["duration_ms"], 30000);
}

#[sqlx::test]
async fn ack_wrong_worker_returns_409(pool: sqlx::PgPool) {
    let state = test_helpers::build_state(pool.clone()).await;
    let token = state.auth.instance_login("test-password").unwrap();

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state.clone()))
            .configure(routes::configure),
    )
    .await;

    let code_req = test::TestRequest::post()
        .uri("/api/v1/workers/codes")
        .insert_header(("Authorization", format!("Bearer {token}")))
        .to_request();
    let code_resp = test::call_service(&app, code_req).await;
    let code_body: serde_json::Value = test::read_body_json(code_resp).await;
    let code_a = code_body["code"].as_str().unwrap();

    let connect_a_req = test::TestRequest::post()
        .uri("/api/v1/workers/connect")
        .set_json(json!({"code": code_a, "worker_name": "worker-a"}))
        .to_request();
    let connect_a_resp = test::call_service(&app, connect_a_req).await;
    let connect_a_body: serde_json::Value = test::read_body_json(connect_a_resp).await;
    let _token_a = connect_a_body["access_token"].as_str().unwrap();
    let worker_a_id = connect_a_body["worker_id"].as_str().unwrap();
    let worker_a_uuid = Uuid::parse_str(worker_a_id).unwrap();

    let code_b_req = test::TestRequest::post()
        .uri("/api/v1/workers/codes")
        .insert_header(("Authorization", format!("Bearer {token}")))
        .to_request();
    let code_b_resp = test::call_service(&app, code_b_req).await;
    let code_b_body: serde_json::Value = test::read_body_json(code_b_resp).await;
    let code_b = code_b_body["code"].as_str().unwrap();

    let connect_b_req = test::TestRequest::post()
        .uri("/api/v1/workers/connect")
        .set_json(json!({"code": code_b, "worker_name": "worker-b"}))
        .to_request();
    let connect_b_resp = test::call_service(&app, connect_b_req).await;
    let connect_b_body: serde_json::Value = test::read_body_json(connect_b_resp).await;
    let token_b = connect_b_body["access_token"].as_str().unwrap();

    let project_id = test_helpers::insert_project_config(&pool, "kaneo-wrong-ack").await;
    let wr_id = test_helpers::insert_pending_work_run(&pool, project_id, "task-wrong-ack").await;

    let dispatch_repo = DispatchRepository;
    dispatch_repo
        .dispatch_to_worker(&pool, wr_id, worker_a_uuid)
        .await
        .expect("Should dispatch to A");

    let ack_b_req = test::TestRequest::post()
        .uri(&format!("/api/v1/jobs/{wr_id}/ack"))
        .insert_header(("Authorization", format!("Bearer {token_b}")))
        .set_json(json!({}))
        .to_request();
    let ack_b_resp = test::call_service(&app, ack_b_req).await;
    assert_eq!(ack_b_resp.status(), 409);
}

#[sqlx::test]
async fn double_ack_returns_409(pool: sqlx::PgPool) {
    let state = test_helpers::build_state(pool.clone()).await;
    let token = state.auth.instance_login("test-password").unwrap();

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state.clone()))
            .configure(routes::configure),
    )
    .await;

    let code_req = test::TestRequest::post()
        .uri("/api/v1/workers/codes")
        .insert_header(("Authorization", format!("Bearer {token}")))
        .to_request();
    let code_resp = test::call_service(&app, code_req).await;
    let code_body: serde_json::Value = test::read_body_json(code_resp).await;
    let code = code_body["code"].as_str().unwrap();

    let connect_req = test::TestRequest::post()
        .uri("/api/v1/workers/connect")
        .set_json(json!({"code": code, "worker_name": "double-ack-worker"}))
        .to_request();
    let connect_resp = test::call_service(&app, connect_req).await;
    let connect_body: serde_json::Value = test::read_body_json(connect_resp).await;
    let access_token = connect_body["access_token"].as_str().unwrap();
    let worker_id = connect_body["worker_id"].as_str().unwrap();
    let worker_uuid = Uuid::parse_str(worker_id).unwrap();

    let project_id = test_helpers::insert_project_config(&pool, "kaneo-double-ack").await;
    let wr_id = test_helpers::insert_pending_work_run(&pool, project_id, "task-double-ack").await;

    let dispatch_repo = DispatchRepository;
    dispatch_repo
        .dispatch_to_worker(&pool, wr_id, worker_uuid)
        .await
        .expect("Should dispatch");

    let ack_req = test::TestRequest::post()
        .uri(&format!("/api/v1/jobs/{wr_id}/ack"))
        .insert_header(("Authorization", format!("Bearer {access_token}")))
        .set_json(json!({}))
        .to_request();
    let ack_resp = test::call_service(&app, ack_req).await;
    assert_eq!(ack_resp.status(), 200);

    let ack_again_req = test::TestRequest::post()
        .uri(&format!("/api/v1/jobs/{wr_id}/ack"))
        .insert_header(("Authorization", format!("Bearer {access_token}")))
        .set_json(json!({}))
        .to_request();
    let ack_again_resp = test::call_service(&app, ack_again_req).await;
    assert_eq!(ack_again_resp.status(), 409);
}
