use super::{
    build_instance_token, build_state, build_worker_token, json, routes, test, test_helpers, web,
    App,
};

#[sqlx::test]
async fn post_runs_cancel_returns_204(pool: sqlx::PgPool) {
    let state = build_state(pool.clone()).await;
    let worker_id = test_helpers::workers::insert_worker(&pool, "cancel-1-worker").await;
    let project_id = test_helpers::project_configs::insert_project_config(&pool, "cancel-1").await;
    let wr_id = test_helpers::work_runs::insert_running_work_run(
        &pool,
        project_id,
        "cancel-1-task",
        worker_id,
    )
    .await;

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
async fn post_runs_cancel_makes_empty_worker_append_observe_should_cancel(pool: sqlx::PgPool) {
    let state = build_state(pool.clone()).await;
    let worker_id = test_helpers::workers::insert_worker(&pool, "cancel-poll-worker").await;
    let project_id =
        test_helpers::project_configs::insert_project_config(&pool, "cancel-poll").await;
    let wr_id = test_helpers::work_runs::insert_running_work_run(
        &pool,
        project_id,
        "cancel-poll-task",
        worker_id,
    )
    .await;

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(routes::configure),
    )
    .await;

    let cancel_req = test::TestRequest::post()
        .uri(&format!("/api/v1/runs/{wr_id}/cancel"))
        .insert_header(("Authorization", build_instance_token().as_str()))
        .to_request();
    let cancel_resp = test::call_service(&app, cancel_req).await;
    assert_eq!(cancel_resp.status(), 204);

    let append_req = test::TestRequest::post()
        .uri(&format!("/api/v1/jobs/{wr_id}/events"))
        .insert_header(("Authorization", build_worker_token(worker_id).as_str()))
        .set_json(json!({"events": []}))
        .to_request();
    let append_resp = test::call_service(&app, append_req).await;
    assert_eq!(append_resp.status(), 200);

    let body: serde_json::Value = test::read_body_json(append_resp).await;
    assert_eq!(body["accepted"], 0);
    assert_eq!(body["should_cancel"], true);
}

#[sqlx::test]
async fn post_runs_cancel_rejects_terminal_status_with_409(pool: sqlx::PgPool) {
    let state = build_state(pool.clone()).await;
    let project_id = test_helpers::project_configs::insert_project_config(&pool, "cancel-2").await;
    let wr_id =
        test_helpers::work_runs::insert_pending_work_run(&pool, project_id, "cancel-2-task").await;

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
