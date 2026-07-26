use super::*;

#[sqlx::test]
async fn poll_returns_204_when_no_dispatch(pool: sqlx::PgPool) {
    let state = build_state(pool.clone()).await;
    let worker_id = test_helpers::workers::insert_worker(&pool, "test-poll-noop").await;

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
    let worker_id = test_helpers::workers::insert_worker(&pool, "test-poll-work").await;
    let project_id =
        test_helpers::project_configs::insert_project_config(&pool, "kaneo-poll-test").await;
    let wr_id =
        test_helpers::work_runs::insert_pending_work_run(&pool, project_id, "task-poll-test").await;

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
    let worker_id = test_helpers::workers::insert_worker(&pool, "test-getter").await;
    let project_id =
        test_helpers::project_configs::insert_project_config(&pool, "kaneo-get-test").await;
    let wr_id =
        test_helpers::work_runs::insert_pending_work_run(&pool, project_id, "task-get-test").await;
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
        .unwrap()
        .contains("Debian-based container"));
    assert_eq!(body["repos"], serde_json::json!([]));
    assert_eq!(body["github_token"], serde_json::Value::Null);
    assert_eq!(body["github_token_expires_at"], serde_json::Value::Null);
}

#[sqlx::test]
async fn refresh_github_token_returns_nulls_for_no_repos(pool: sqlx::PgPool) {
    let state = build_state(pool.clone()).await;
    let worker_id = test_helpers::workers::insert_worker(&pool, "test-token-no-repos").await;
    let project_id =
        test_helpers::project_configs::insert_project_config(&pool, "kaneo-token-no-repos").await;
    let wr_id = test_helpers::work_runs::insert_running_work_run(
        &pool,
        project_id,
        "task-token-no-repos",
        worker_id,
    )
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
    let owner = test_helpers::workers::insert_worker(&pool, "test-token-owner").await;
    let attacker = test_helpers::workers::insert_worker(&pool, "test-token-attacker").await;
    let project_id =
        test_helpers::project_configs::insert_project_config(&pool, "kaneo-token-owner").await;
    let wr_id = test_helpers::work_runs::insert_running_work_run(
        &pool,
        project_id,
        "task-token-owner",
        owner,
    )
    .await;

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
    let worker_id = test_helpers::workers::insert_worker(&pool, "test-missing-getter").await;

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
