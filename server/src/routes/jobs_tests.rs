use actix_web::{test, web, App};
use serde_json::json;
use uuid::Uuid;

use crate::app_state::AppState;
use crate::db::dispatcher::DispatchRepository;
use crate::routes;
use crate::test_helpers;

async fn build_state(pool: sqlx::PgPool) -> AppState {
    test_helpers::build_state(pool).await
}

fn build_worker_token(worker_id: Uuid) -> String {
    test_helpers::build_worker_token(worker_id)
}

#[sqlx::test]
async fn poll_returns_204_when_no_dispatch(pool: sqlx::PgPool) {
    let state = build_state(pool.clone()).await;
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
    let state = build_state(pool.clone()).await;
    let worker_id = test_helpers::insert_worker(&pool, "test-poll-work").await;
    let project_id = test_helpers::insert_project_config(&pool, "kaneo-poll-test").await;
    let wr_id = test_helpers::insert_pending_work_run(&pool, project_id, "task-poll-test").await;

    state
        .jobs
        .dispatch_store()
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
    let state = build_state(pool.clone()).await;
    let worker_id = test_helpers::insert_worker(&pool, "test-getter").await;
    let project_id = test_helpers::insert_project_config(&pool, "kaneo-get-test").await;
    let wr_id = test_helpers::insert_pending_work_run(&pool, project_id, "task-get-test").await;
    let dispatch_repo = DispatchRepository;
    dispatch_repo
        .dispatch_to_worker(&pool, wr_id, worker_id)
        .await
        .expect("Should assign worker");

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
    assert!(body["prompt_text"]
        .as_str()
        .unwrap_or_default()
        .contains("Task title:"));
    assert_eq!(body["repos"], serde_json::json!([]));
    assert_eq!(body["github_token"], serde_json::Value::Null);
    assert_eq!(body["github_token_expires_at"], serde_json::Value::Null);
}

#[sqlx::test]
async fn refresh_github_token_returns_nulls_for_no_repos(pool: sqlx::PgPool) {
    let state = build_state(pool.clone()).await;
    let worker_id = test_helpers::insert_worker(&pool, "test-token-no-repos").await;
    let project_id = test_helpers::insert_project_config(&pool, "kaneo-token-no-repos").await;
    let wr_id =
        test_helpers::insert_running_work_run(&pool, project_id, "task-token-no-repos", worker_id)
            .await;

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(routes::configure),
    )
    .await;

    let req = test::TestRequest::post()
        .uri(&format!("/api/v1/jobs/{wr_id}/github-token"))
        .insert_header(("Authorization", build_worker_token(worker_id).as_str()))
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["github_token"], serde_json::Value::Null);
    assert_eq!(body["github_token_expires_at"], serde_json::Value::Null);
}

#[sqlx::test]
async fn refresh_github_token_rejects_wrong_owner(pool: sqlx::PgPool) {
    let state = build_state(pool.clone()).await;
    let owner = test_helpers::insert_worker(&pool, "test-token-owner").await;
    let attacker = test_helpers::insert_worker(&pool, "test-token-attacker").await;
    let project_id = test_helpers::insert_project_config(&pool, "kaneo-token-owner").await;
    let wr_id =
        test_helpers::insert_running_work_run(&pool, project_id, "task-token-owner", owner).await;

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(routes::configure),
    )
    .await;

    let req = test::TestRequest::post()
        .uri(&format!("/api/v1/jobs/{wr_id}/github-token"))
        .insert_header(("Authorization", build_worker_token(attacker).as_str()))
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), 403);
}

#[sqlx::test]
async fn get_job_returns_404_for_missing(pool: sqlx::PgPool) {
    let state = build_state(pool.clone()).await;
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
    let state = build_state(pool.clone()).await;
    let worker_id = test_helpers::insert_worker(&pool, "test-acker").await;
    let project_id = test_helpers::insert_project_config(&pool, "kaneo-ack-test").await;
    let wr_id = test_helpers::insert_pending_work_run(&pool, project_id, "task-ack-test").await;

    let dispatch_repo = DispatchRepository;
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
        .set_json(serde_json::json!({}))
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["status"], "running");
    assert_eq!(body["worker_id"], worker_id.to_string());
}

#[sqlx::test]
async fn ack_job_returns_409_when_already_claimed(pool: sqlx::PgPool) {
    let state = build_state(pool.clone()).await;
    let worker_a = test_helpers::insert_worker(&pool, "test-acker-a").await;
    let worker_b = test_helpers::insert_worker(&pool, "test-acker-b").await;
    let project_id = test_helpers::insert_project_config(&pool, "kaneo-ack-race").await;
    let wr_id = test_helpers::insert_pending_work_run(&pool, project_id, "task-ack-race").await;

    let dispatch_repo = DispatchRepository;
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
        .set_json(serde_json::json!({}))
        .to_request();
    let resp_a = test::call_service(&app, req_a).await;
    assert_eq!(resp_a.status(), 200);

    let req_b = test::TestRequest::post()
        .uri(&format!("/api/v1/jobs/{wr_id}/ack"))
        .insert_header(("Authorization", build_worker_token(worker_b).as_str()))
        .set_json(serde_json::json!({}))
        .to_request();
    let resp_b = test::call_service(&app, req_b).await;
    assert_eq!(resp_b.status(), 409);
}

#[sqlx::test]
async fn submit_result_returns_200_on_completed(pool: sqlx::PgPool) {
    let state = build_state(pool.clone()).await;
    let worker_id = test_helpers::insert_worker(&pool, "test-result").await;
    let project_id = test_helpers::insert_project_config(&pool, "kaneo-result-test").await;
    let wr_id = test_helpers::insert_pending_work_run(&pool, project_id, "task-result-test").await;

    let dispatch_repo = DispatchRepository;
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
        .set_json(serde_json::json!({}))
        .to_request();
    test::call_service(&app, ack_req).await;

    let req = test::TestRequest::post()
        .uri(&format!("/api/v1/jobs/{wr_id}/result"))
        .insert_header(("Authorization", build_worker_token(worker_id).as_str()))
        .set_json(json!({
            "pr_urls": ["https://github.com/test/pr/1"],
            "exit_code": 0,
            "tokens_used": 1000,
            "duration_ms": 60000,
            "input_tokens": 0,
            "output_tokens": 0,
            "cache_read_tokens": 0,
            "cache_write_tokens": 0,
            "model_used": null,
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
    let state = build_state(pool.clone()).await;
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
            "pr_urls": [],
            "exit_code": 0,
            "tokens_used": 0,
            "duration_ms": 0,
            "input_tokens": 0,
            "output_tokens": 0,
            "cache_read_tokens": 0,
            "cache_write_tokens": 0,
            "model_used": null,
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), 409);
}

#[sqlx::test]
async fn poll_rejects_missing_auth(pool: sqlx::PgPool) {
    let state = build_state(pool).await;

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
    let state = build_state(pool).await;

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

fn build_instance_token() -> String {
    let exp = chrono::Utc::now() + chrono::Duration::minutes(15);
    let claims = serde_json::json!({"sub": "instance", "exp": exp.timestamp()});
    let token = jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret("test-secret".as_bytes()),
    )
    .expect("should build instance token");
    format!("Bearer {token}")
}

#[sqlx::test]
async fn post_events_appends_batch_and_returns_next_expected(pool: sqlx::PgPool) {
    let state = build_state(pool.clone()).await;
    let worker_id = test_helpers::insert_worker(&pool, "events-1-worker").await;
    let project_id = test_helpers::insert_project_config(&pool, "events-1").await;
    let wr_id =
        test_helpers::insert_running_work_run(&pool, project_id, "events-1-task", worker_id).await;

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(routes::configure),
    )
    .await;

    let body = json!({
        "events": [
            {"sequence": 1, "event_type": "session.started", "payload": {}, "occurred_at": "2025-06-08T00:00:00Z"},
            {"sequence": 2, "event_type": "turn.completed", "payload": {"text": "ok"}, "occurred_at": "2025-06-08T00:00:01Z"}
        ]
    });
    let req = test::TestRequest::post()
        .uri(&format!("/api/v1/jobs/{wr_id}/events"))
        .insert_header(("Authorization", build_worker_token(worker_id).as_str()))
        .set_json(body)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["accepted"], 2);
    assert_eq!(body["should_cancel"], false);
}

#[sqlx::test]
async fn post_events_accepts_out_of_order_sequences(pool: sqlx::PgPool) {
    let state = build_state(pool.clone()).await;
    let worker_id = test_helpers::insert_worker(&pool, "events-2-worker").await;
    let project_id = test_helpers::insert_project_config(&pool, "events-2").await;
    let wr_id =
        test_helpers::insert_running_work_run(&pool, project_id, "events-2-task", worker_id).await;

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(routes::configure),
    )
    .await;

    let body = json!({"events": [{"sequence": 5, "event_type": "x", "payload": {}, "occurred_at": "2025-06-08T00:00:00Z"}]});
    let req = test::TestRequest::post()
        .uri(&format!("/api/v1/jobs/{wr_id}/events"))
        .insert_header(("Authorization", build_worker_token(worker_id).as_str()))
        .set_json(body)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["accepted"], 1);
}

#[sqlx::test]
async fn post_events_rejects_wrong_owner_with_404(pool: sqlx::PgPool) {
    let state = build_state(pool.clone()).await;
    let owner = test_helpers::insert_worker(&pool, "events-3-owner").await;
    let attacker = test_helpers::insert_worker(&pool, "events-3-attacker").await;
    let project_id = test_helpers::insert_project_config(&pool, "events-3").await;
    let wr_id =
        test_helpers::insert_running_work_run(&pool, project_id, "events-3-task", owner).await;

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(routes::configure),
    )
    .await;

    let body = json!({"events": [{"sequence": 1, "event_type": "x", "payload": {}, "occurred_at": "2025-06-08T00:00:00Z"}]});
    let req = test::TestRequest::post()
        .uri(&format!("/api/v1/jobs/{wr_id}/events"))
        .insert_header(("Authorization", build_worker_token(attacker).as_str()))
        .set_json(body)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 404);
}

#[sqlx::test]
async fn get_events_admin_returns_full_list(pool: sqlx::PgPool) {
    let state = build_state(pool.clone()).await;
    let worker_id = test_helpers::insert_worker(&pool, "events-4-worker").await;
    let project_id = test_helpers::insert_project_config(&pool, "events-4").await;
    let wr_id =
        test_helpers::insert_running_work_run(&pool, project_id, "events-4-task", worker_id).await;

    state
        .events
        .append_events(
            wr_id,
            worker_id,
            vec![vulcanum_shared::api_types::WireEvent {
                sequence: 1,
                event_type: "session.started".to_owned(),
                payload: serde_json::json!({}),
                occurred_at: chrono::Utc::now(),
            }],
        )
        .await
        .expect("append");

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(routes::configure),
    )
    .await;

    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/jobs/{wr_id}/events"))
        .insert_header(("Authorization", build_instance_token().as_str()))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["events"].as_array().map(|a| a.len()), Some(1));
    assert_eq!(body["has_more"], false);
}

#[sqlx::test]
async fn post_runs_cancel_returns_204(pool: sqlx::PgPool) {
    let state = build_state(pool.clone()).await;
    let worker_id = test_helpers::insert_worker(&pool, "cancel-1-worker").await;
    let project_id = test_helpers::insert_project_config(&pool, "cancel-1").await;
    let wr_id =
        test_helpers::insert_running_work_run(&pool, project_id, "cancel-1-task", worker_id).await;

    let cancel_store = state.jobs.cancel_store();
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(routes::configure),
    )
    .await;

    let req = test::TestRequest::post()
        .uri(&format!("/api/v1/runs/{wr_id}/cancel"))
        .insert_header(("Authorization", build_instance_token().as_str()))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 204);

    assert!(cancel_store
        .is_cancel_requested(wr_id)
        .await
        .expect("is_cancel"));
}

#[sqlx::test]
async fn post_runs_cancel_rejects_terminal_status_with_409(pool: sqlx::PgPool) {
    let state = build_state(pool.clone()).await;
    let project_id = test_helpers::insert_project_config(&pool, "cancel-2").await;
    let wr_id = test_helpers::insert_pending_work_run(&pool, project_id, "cancel-2-task").await;

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(routes::configure),
    )
    .await;

    let req = test::TestRequest::post()
        .uri(&format!("/api/v1/runs/{wr_id}/cancel"))
        .insert_header(("Authorization", build_instance_token().as_str()))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 409);
}

#[sqlx::test]
async fn get_events_recent_returns_last_n_ascending(pool: sqlx::PgPool) {
    let state = build_state(pool.clone()).await;
    let worker_id = test_helpers::insert_worker(&pool, "recent-worker").await;
    let project_id = test_helpers::insert_project_config(&pool, "recent").await;
    let wr_id =
        test_helpers::insert_running_work_run(&pool, project_id, "recent-task", worker_id).await;

    let events: Vec<vulcanum_shared::api_types::WireEvent> = (1..=25)
        .map(|i| vulcanum_shared::api_types::WireEvent {
            sequence: i as u64,
            event_type: format!("e{i}"),
            payload: serde_json::json!({}),
            occurred_at: chrono::Utc::now(),
        })
        .collect();
    state
        .events
        .append_events(wr_id, worker_id, events)
        .await
        .expect("append");

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(routes::configure),
    )
    .await;

    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/runs/{wr_id}/events/recent"))
        .insert_header(("Authorization", build_instance_token().as_str()))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = test::read_body_json(resp).await;
    let arr = body.as_array().expect("array");
    assert_eq!(arr.len(), 20, "must return exactly last 20 events");
    assert_eq!(arr[0]["sequence"], 6, "first must be sequence 6");
    assert_eq!(arr[19]["sequence"], 25, "last must be sequence 25");
}
