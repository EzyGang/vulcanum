use std::sync::Arc;

use chrono::{TimeZone, Utc};

use crate::db::work_run_events::WorkRunEventsRepository;
use crate::db::work_runs::WorkRunsRepository;
use crate::models::work_run_events::errors::WorkRunEventsError;
use crate::services::dispatcher::cancel_store::{CancelStore, InMemoryCancelStore};
use crate::services::work_run_events::service::WorkRunEventsService;
use crate::test_helpers;
use vulcanum_shared::api_types::WireEvent;

fn make_wire_event(seq: u64, event_type: &str) -> WireEvent {
    WireEvent {
        sequence: seq,
        event_type: event_type.to_owned(),
        payload: serde_json::json!({"i": seq}),
        occurred_at: Utc::now(),
    }
}

fn build_service(pool: sqlx::PgPool) -> (WorkRunEventsService, Arc<InMemoryCancelStore>) {
    let cancel = Arc::new(InMemoryCancelStore::new());
    let svc = WorkRunEventsService::new(
        WorkRunEventsRepository::new(),
        WorkRunsRepository::new(),
        cancel.clone(),
        pool,
    );
    (svc, cancel)
}

#[sqlx::test]
async fn append_events_happy_path_returns_should_cancel_false(pool: sqlx::PgPool) {
    let (svc, _cancel) = build_service(pool.clone());
    let project_id = test_helpers::insert_project_config(&pool, "evt-svc-1").await;
    let worker_id = test_helpers::insert_worker(&pool, "evt-svc-worker-1").await;
    let wr_id =
        test_helpers::insert_running_work_run(&pool, project_id, "evt-svc-task-1", worker_id).await;

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
    let project_id = test_helpers::insert_project_config(&pool, "evt-svc-touch").await;
    let worker_id = test_helpers::insert_worker(&pool, "evt-svc-touch-worker").await;
    let wr_id =
        test_helpers::insert_running_work_run(&pool, project_id, "evt-svc-touch-task", worker_id)
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
    let project_id = test_helpers::insert_project_config(&pool, "evt-svc-heartbeat-hidden").await;
    let worker_id = test_helpers::insert_worker(&pool, "evt-svc-heartbeat-hidden-worker").await;
    let wr_id = test_helpers::insert_running_work_run(
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
    let project_id = test_helpers::insert_project_config(&pool, "evt-svc-2").await;
    let owner = test_helpers::insert_worker(&pool, "evt-svc-worker-2a").await;
    let attacker = test_helpers::insert_worker(&pool, "evt-svc-worker-2b").await;
    let wr_id =
        test_helpers::insert_running_work_run(&pool, project_id, "evt-svc-task-2", owner).await;

    let err = svc
        .append_events(wr_id, attacker, vec![make_wire_event(1, "x")])
        .await
        .expect_err("must reject cross-worker append");
    assert!(matches!(err, WorkRunEventsError::NotFound));
}

#[sqlx::test]
async fn append_events_returns_should_cancel_when_flag_set(pool: sqlx::PgPool) {
    let (svc, cancel) = build_service(pool.clone());
    let project_id = test_helpers::insert_project_config(&pool, "evt-svc-3").await;
    let worker_id = test_helpers::insert_worker(&pool, "evt-svc-worker-3").await;
    let wr_id =
        test_helpers::insert_running_work_run(&pool, project_id, "evt-svc-task-3", worker_id).await;

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
    let project_id = test_helpers::insert_project_config(&pool, "evt-svc-empty-cancel").await;
    let worker_id = test_helpers::insert_worker(&pool, "evt-svc-empty-cancel-worker").await;
    let wr_id = test_helpers::insert_running_work_run(
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
    let project_id = test_helpers::insert_project_config(&pool, "evt-svc-4").await;
    let worker_id = test_helpers::insert_worker(&pool, "evt-svc-worker-4").await;
    let wr_id =
        test_helpers::insert_running_work_run(&pool, project_id, "evt-svc-task-4", worker_id).await;

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
    let project_id = test_helpers::insert_project_config(&pool, "evt-svc-8").await;
    let worker_id = test_helpers::insert_worker(&pool, "evt-svc-worker-8").await;
    let wr_id =
        test_helpers::insert_running_work_run(&pool, project_id, "evt-svc-task-8", worker_id).await;

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

#[sqlx::test]
async fn list_events_admin_returns_events_for_any_run(pool: sqlx::PgPool) {
    let (svc, _cancel) = build_service(pool.clone());
    let project_id = test_helpers::insert_project_config(&pool, "evt-svc-5").await;
    let worker_id = test_helpers::insert_worker(&pool, "evt-svc-worker-5").await;
    let wr_id =
        test_helpers::insert_running_work_run(&pool, project_id, "evt-svc-task-5", worker_id).await;

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
    let project_id = test_helpers::insert_project_config(&pool, "evt-svc-6").await;
    let owner = test_helpers::insert_worker(&pool, "evt-svc-worker-6a").await;
    let attacker = test_helpers::insert_worker(&pool, "evt-svc-worker-6b").await;
    let wr_id =
        test_helpers::insert_running_work_run(&pool, project_id, "evt-svc-task-6", owner).await;

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
    let project_id = test_helpers::insert_project_config(&pool, "evt-svc-7").await;
    let worker_id = test_helpers::insert_worker(&pool, "evt-svc-worker-7").await;
    let wr_id =
        test_helpers::insert_running_work_run(&pool, project_id, "evt-svc-task-7", worker_id).await;

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
    let team_b = test_helpers::insert_team(&pool, "events-team-b").await;
    let project_id = test_helpers::insert_project_config(&pool, "evt-cross-team").await;
    let worker_id = test_helpers::insert_worker(&pool, "evt-cross-team-worker").await;
    let wr_id =
        test_helpers::insert_running_work_run(&pool, project_id, "evt-cross-team-task", worker_id)
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
