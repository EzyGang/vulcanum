use super::{build_service, make_wire_event, test_helpers, CancelStore, WorkRunEventsError};

#[sqlx::test]
async fn append_events_happy_path_returns_should_cancel_false(pool: sqlx::PgPool) {
    let (svc, _cancel) = build_service(pool.clone());
    let project_id = test_helpers::project_configs::insert_project_config(&pool, "evt-svc-1").await;
    let worker_id = test_helpers::workers::insert_worker(&pool, "evt-svc-worker-1").await;
    let wr_id = test_helpers::work_runs::insert_running_work_run(
        &pool,
        project_id,
        "evt-svc-task-1",
        worker_id,
    )
    .await;

    let result = svc
        .append_events(wr_id, worker_id, vec![make_wire_event(1, "turn.started")])
        .await
        .expect("append");

    assert_eq!(result.accepted, 1);
    assert!(!result.should_cancel);
}

#[sqlx::test]
async fn append_events_touches_active_run_updated_at(pool: sqlx::PgPool) {
    let (svc, _cancel) = build_service(pool.clone());
    let project_id =
        test_helpers::project_configs::insert_project_config(&pool, "evt-svc-touch").await;
    let worker_id = test_helpers::workers::insert_worker(&pool, "evt-svc-touch-worker").await;
    let wr_id = test_helpers::work_runs::insert_running_work_run(
        &pool,
        project_id,
        "evt-svc-touch-task",
        worker_id,
    )
    .await;

    sqlx::query!(
        "UPDATE work_runs SET updated_at = NOW() - INTERVAL '1 hour' WHERE id = $1",
        wr_id,
    )
    .execute(&pool)
    .await
    .expect("move updated_at into past");

    let before = sqlx::query!("SELECT updated_at FROM work_runs WHERE id = $1", wr_id)
        .fetch_one(&pool)
        .await
        .expect("fetch before")
        .updated_at
        .expect("updated_at exists");

    let result = svc
        .append_events(
            wr_id,
            worker_id,
            vec![make_wire_event(1, "worker.heartbeat")],
        )
        .await
        .expect("append heartbeat");

    assert_eq!(result.accepted, 0);

    let recent = svc
        .list_recent(wr_id, test_helpers::DEFAULT_TEAM_ID)
        .await
        .expect("list recent");
    assert!(recent.is_empty());

    let after = sqlx::query!("SELECT updated_at FROM work_runs WHERE id = $1", wr_id)
        .fetch_one(&pool)
        .await
        .expect("fetch after")
        .updated_at
        .expect("updated_at exists");

    assert!(after > before);
}

#[sqlx::test]
async fn append_events_hides_existing_heartbeats_from_recent_list(pool: sqlx::PgPool) {
    let (svc, _cancel) = build_service(pool.clone());
    let project_id =
        test_helpers::project_configs::insert_project_config(&pool, "evt-svc-heartbeat-hidden")
            .await;
    let worker_id =
        test_helpers::workers::insert_worker(&pool, "evt-svc-heartbeat-hidden-worker").await;
    let wr_id = test_helpers::work_runs::insert_running_work_run(
        &pool,
        project_id,
        "evt-svc-heartbeat-hidden-task",
        worker_id,
    )
    .await;

    sqlx::query!(
        r#"INSERT INTO work_run_events (work_run_id, sequence, event_type, payload, occurred_at)
           VALUES ($1, 1, 'worker.heartbeat', '{}'::jsonb, NOW())"#,
        wr_id,
    )
    .execute(&pool)
    .await
    .expect("insert legacy heartbeat");

    svc.append_events(wr_id, worker_id, vec![make_wire_event(2, "turn.started")])
        .await
        .expect("append visible event");

    let recent = svc
        .list_recent(wr_id, test_helpers::DEFAULT_TEAM_ID)
        .await
        .expect("list recent");

    assert_eq!(recent.len(), 1);
    assert_eq!(recent[0].event_type, "turn.started");
}

#[sqlx::test]
async fn append_events_rejects_wrong_owner(pool: sqlx::PgPool) {
    let (svc, _cancel) = build_service(pool.clone());
    let project_id = test_helpers::project_configs::insert_project_config(&pool, "evt-svc-2").await;
    let owner = test_helpers::workers::insert_worker(&pool, "evt-svc-worker-2a").await;
    let attacker = test_helpers::workers::insert_worker(&pool, "evt-svc-worker-2b").await;
    let wr_id = test_helpers::work_runs::insert_running_work_run(
        &pool,
        project_id,
        "evt-svc-task-2",
        owner,
    )
    .await;

    let err = svc
        .append_events(wr_id, attacker, vec![make_wire_event(1, "x")])
        .await
        .expect_err("must reject cross-worker append");
    assert!(matches!(err, WorkRunEventsError::NotFound));
}

#[sqlx::test]
async fn append_events_returns_should_cancel_when_flag_set(pool: sqlx::PgPool) {
    let (svc, cancel) = build_service(pool.clone());
    let project_id = test_helpers::project_configs::insert_project_config(&pool, "evt-svc-3").await;
    let worker_id = test_helpers::workers::insert_worker(&pool, "evt-svc-worker-3").await;
    let wr_id = test_helpers::work_runs::insert_running_work_run(
        &pool,
        project_id,
        "evt-svc-task-3",
        worker_id,
    )
    .await;

    cancel.request_cancel(wr_id).await.expect("set cancel");

    let result = svc
        .append_events(wr_id, worker_id, vec![make_wire_event(1, "x")])
        .await
        .expect("append");
    assert!(result.should_cancel);
}

#[sqlx::test]
async fn append_empty_events_returns_should_cancel_when_flag_set(pool: sqlx::PgPool) {
    let (svc, cancel) = build_service(pool.clone());
    let project_id =
        test_helpers::project_configs::insert_project_config(&pool, "evt-svc-empty-cancel").await;
    let worker_id =
        test_helpers::workers::insert_worker(&pool, "evt-svc-empty-cancel-worker").await;
    let wr_id = test_helpers::work_runs::insert_running_work_run(
        &pool,
        project_id,
        "evt-svc-empty-cancel-task",
        worker_id,
    )
    .await;

    cancel.request_cancel(wr_id).await.expect("set cancel");

    let result = svc
        .append_events(wr_id, worker_id, Vec::new())
        .await
        .expect("append empty event batch");

    assert_eq!(result.accepted, 0);
    assert!(result.should_cancel);
}

#[sqlx::test]
async fn append_events_accepts_duplicate_sequences(pool: sqlx::PgPool) {
    let (svc, _cancel) = build_service(pool.clone());
    let project_id = test_helpers::project_configs::insert_project_config(&pool, "evt-svc-4").await;
    let worker_id = test_helpers::workers::insert_worker(&pool, "evt-svc-worker-4").await;
    let wr_id = test_helpers::work_runs::insert_running_work_run(
        &pool,
        project_id,
        "evt-svc-task-4",
        worker_id,
    )
    .await;

    let r1 = svc
        .append_events(wr_id, worker_id, vec![make_wire_event(1, "a")])
        .await
        .expect("first");

    assert_eq!(r1.accepted, 1);

    let r2 = svc
        .append_events(wr_id, worker_id, vec![make_wire_event(1, "a")])
        .await
        .expect("duplicate silently skipped");

    assert_eq!(r2.accepted, 0);
}

#[sqlx::test]
async fn append_events_accepts_out_of_order_sequences(pool: sqlx::PgPool) {
    let (svc, _cancel) = build_service(pool.clone());
    let project_id = test_helpers::project_configs::insert_project_config(&pool, "evt-svc-8").await;
    let worker_id = test_helpers::workers::insert_worker(&pool, "evt-svc-worker-8").await;
    let wr_id = test_helpers::work_runs::insert_running_work_run(
        &pool,
        project_id,
        "evt-svc-task-8",
        worker_id,
    )
    .await;

    let r1 = svc
        .append_events(wr_id, worker_id, vec![make_wire_event(2, "late.arrival")])
        .await
        .expect("first");

    assert_eq!(r1.accepted, 1);

    let r2 = svc
        .append_events(wr_id, worker_id, vec![make_wire_event(1, "early.arrival")])
        .await
        .expect("second");

    assert_eq!(r2.accepted, 1);
}
