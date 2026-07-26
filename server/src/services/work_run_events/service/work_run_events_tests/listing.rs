use super::{build_service, make_wire_event, test_helpers, TimeZone, Utc, WorkRunEventsError};

#[sqlx::test]
async fn list_events_admin_returns_events_for_any_run(pool: sqlx::PgPool) {
    let (svc, _cancel) = build_service(pool.clone());
    let project_id = test_helpers::project_configs::insert_project_config(&pool, "evt-svc-5").await;
    let worker_id = test_helpers::workers::insert_worker(&pool, "evt-svc-worker-5").await;
    let wr_id = test_helpers::work_runs::insert_running_work_run(
        &pool,
        project_id,
        "evt-svc-task-5",
        worker_id,
    )
    .await;

    svc.append_events(
        wr_id,
        worker_id,
        vec![
            make_wire_event(1, "turn.started"),
            make_wire_event(2, "session.completed"),
        ],
    )
    .await
    .expect("append");

    let result = svc
        .list_events_admin(
            wr_id,
            chrono::Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap(),
            0,
            100,
        )
        .await
        .expect("list");
    assert_eq!(result.events.len(), 2);
    assert!(!result.has_more);
}

#[sqlx::test]
async fn list_events_worker_rejects_cross_worker(pool: sqlx::PgPool) {
    let (svc, _cancel) = build_service(pool.clone());
    let project_id = test_helpers::project_configs::insert_project_config(&pool, "evt-svc-6").await;
    let owner = test_helpers::workers::insert_worker(&pool, "evt-svc-worker-6a").await;
    let attacker = test_helpers::workers::insert_worker(&pool, "evt-svc-worker-6b").await;
    let wr_id = test_helpers::work_runs::insert_running_work_run(
        &pool,
        project_id,
        "evt-svc-task-6",
        owner,
    )
    .await;

    let err = svc
        .list_events(wr_id, attacker, Utc::now(), 0, 10)
        .await
        .expect_err("must reject cross-worker list");
    assert!(matches!(err, WorkRunEventsError::NotFound));
}

#[sqlx::test]
async fn list_recent_returns_last_n_ascending(pool: sqlx::PgPool) {
    use chrono::TimeZone;

    let (svc, _cancel) = build_service(pool.clone());
    let project_id = test_helpers::project_configs::insert_project_config(&pool, "evt-svc-7").await;
    let worker_id = test_helpers::workers::insert_worker(&pool, "evt-svc-worker-7").await;
    let wr_id = test_helpers::work_runs::insert_running_work_run(
        &pool,
        project_id,
        "evt-svc-task-7",
        worker_id,
    )
    .await;

    for i in 1..=25 {
        let mut ev = make_wire_event(i, &format!("e{i}"));
        ev.occurred_at = Utc.with_ymd_and_hms(2025, 6, i as u32, 0, 0, 0).unwrap();
        svc.append_events(wr_id, worker_id, vec![ev])
            .await
            .expect("append");
    }

    let recent = svc
        .list_recent(wr_id, test_helpers::DEFAULT_TEAM_ID)
        .await
        .expect("list");
    let sequences: Vec<i64> = recent.iter().map(|e| e.sequence).collect();
    assert_eq!(sequences.len(), 20);
    assert_eq!(sequences[0], 6);
    assert_eq!(sequences[19], 25);
}

#[sqlx::test]
async fn list_recent_rejects_cross_team_run(pool: sqlx::PgPool) {
    let (svc, _cancel) = build_service(pool.clone());
    let team_b = test_helpers::teams::insert_team(&pool, "events-team-b").await;
    let project_id =
        test_helpers::project_configs::insert_project_config(&pool, "evt-cross-team").await;
    let worker_id = test_helpers::workers::insert_worker(&pool, "evt-cross-team-worker").await;
    let wr_id = test_helpers::work_runs::insert_running_work_run(
        &pool,
        project_id,
        "evt-cross-team-task",
        worker_id,
    )
    .await;

    let mut event = make_wire_event(1, "cross-team-event");
    event.occurred_at = chrono::Utc::now();
    svc.append_events(wr_id, worker_id, vec![event])
        .await
        .expect("append event");

    let recent = svc
        .list_recent(wr_id, team_b)
        .await
        .expect("cross-team recent list should not expose events");

    assert!(recent.is_empty());
}
