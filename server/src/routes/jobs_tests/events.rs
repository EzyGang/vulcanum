use super::{
    build_instance_token, build_state, build_worker_token, json, routes, test, test_helpers, web,
    App,
};

#[sqlx::test]
async fn post_events_appends_batch_and_returns_next_expected(pool: sqlx::PgPool) {
    let state = build_state(pool.clone()).await;
    let worker_id = test_helpers::workers::insert_worker(&pool, "events-1-worker").await;
    let project_id = test_helpers::project_configs::insert_project_config(&pool, "events-1").await;
    let wr_id = test_helpers::work_runs::insert_running_work_run(
        &pool,
        project_id,
        "events-1-task",
        worker_id,
    )
    .await;

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
    let worker_id = test_helpers::workers::insert_worker(&pool, "events-2-worker").await;
    let project_id = test_helpers::project_configs::insert_project_config(&pool, "events-2").await;
    let wr_id = test_helpers::work_runs::insert_running_work_run(
        &pool,
        project_id,
        "events-2-task",
        worker_id,
    )
    .await;

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
    let owner = test_helpers::workers::insert_worker(&pool, "events-3-owner").await;
    let attacker = test_helpers::workers::insert_worker(&pool, "events-3-attacker").await;
    let project_id = test_helpers::project_configs::insert_project_config(&pool, "events-3").await;
    let wr_id =
        test_helpers::work_runs::insert_running_work_run(&pool, project_id, "events-3-task", owner)
            .await;

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
    let worker_id = test_helpers::workers::insert_worker(&pool, "events-4-worker").await;
    let project_id = test_helpers::project_configs::insert_project_config(&pool, "events-4").await;
    let wr_id = test_helpers::work_runs::insert_running_work_run(
        &pool,
        project_id,
        "events-4-task",
        worker_id,
    )
    .await;

    state
        .events
        .append_events(
            wr_id,
            worker_id,
            vec![vulcanum_shared::api::wire::WireEvent {
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
