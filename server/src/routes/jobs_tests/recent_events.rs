use super::*;

#[sqlx::test]
async fn get_events_recent_returns_last_n_ascending(pool: sqlx::PgPool) {
    let state = build_state(pool.clone()).await;
    let worker_id = test_helpers::workers::insert_worker(&pool, "recent-worker").await;
    let project_id = test_helpers::project_configs::insert_project_config(&pool, "recent").await;
    let wr_id = test_helpers::work_runs::insert_running_work_run(
        &pool,
        project_id,
        "recent-task",
        worker_id,
    )
    .await;

    let events: Vec<vulcanum_shared::api::wire::WireEvent> = (1..=25)
        .map(|i| vulcanum_shared::api::wire::WireEvent {
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
