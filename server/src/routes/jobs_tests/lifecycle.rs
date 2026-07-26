use super::{
    build_state, build_worker_token, json, routes, test, test_helpers, web, App, DispatchRepository,
};

#[sqlx::test]
async fn ack_job_returns_200(pool: sqlx::PgPool) {
    let state = build_state(pool.clone()).await;
    let worker_id = test_helpers::workers::insert_worker(&pool, "test-acker").await;
    let project_id =
        test_helpers::project_configs::insert_project_config(&pool, "kaneo-ack-test").await;
    let wr_id =
        test_helpers::work_runs::insert_pending_work_run(&pool, project_id, "task-ack-test").await;

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
    let worker_a = test_helpers::workers::insert_worker(&pool, "test-acker-a").await;
    let worker_b = test_helpers::workers::insert_worker(&pool, "test-acker-b").await;
    let project_id =
        test_helpers::project_configs::insert_project_config(&pool, "kaneo-ack-race").await;
    let wr_id =
        test_helpers::work_runs::insert_pending_work_run(&pool, project_id, "task-ack-race").await;

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
    let worker_id = test_helpers::workers::insert_worker(&pool, "test-result").await;
    let project_id =
        test_helpers::project_configs::insert_project_config(&pool, "kaneo-result-test").await;
    let wr_id =
        test_helpers::work_runs::insert_pending_work_run(&pool, project_id, "task-result-test")
            .await;

    let dispatch_repo = DispatchRepository;
    dispatch_repo
        .dispatch_to_worker(&pool, wr_id, worker_id)
        .await
        .expect("Should dispatch");
    dispatch_repo
        .increment_worker_jobs(&pool, worker_id)
        .await
        .expect("Should reserve worker capacity");

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
    let worker_id = test_helpers::workers::insert_worker(&pool, "test-early-result").await;
    let project_id =
        test_helpers::project_configs::insert_project_config(&pool, "kaneo-early-result").await;
    let wr_id =
        test_helpers::work_runs::insert_pending_work_run(&pool, project_id, "task-early-result")
            .await;

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
