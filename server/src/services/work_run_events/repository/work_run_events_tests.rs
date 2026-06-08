use chrono::{TimeZone, Utc};
use serde_json::json;
use uuid::Uuid;

use crate::services::work_run_events::model::WorkRunEvent;
use crate::services::work_run_events::repository::queries::InsertEventParams;
use crate::services::work_run_events::repository::WorkRunEventsRepository;
use crate::test_helpers;

fn make_event(seq: i64, event_type: &str) -> InsertEventParams {
    InsertEventParams {
        sequence: seq,
        event_type: event_type.to_owned(),
        payload: json!({"i": seq}),
        occurred_at: Utc::now(),
    }
}

#[sqlx::test]
async fn insert_batch_happy_path(pool: sqlx::PgPool) {
    let repo = WorkRunEventsRepository::new();
    let project_id = test_helpers::insert_project_config(&pool, "evt-happy").await;
    let wr_id = test_helpers::insert_pending_work_run(&pool, project_id, "evt-happy-task").await;

    let events = vec![
        make_event(1, "turn.started"),
        make_event(2, "session.completed"),
    ];

    let result = repo
        .insert_batch(&pool, wr_id, &events)
        .await
        .expect("insert");

    assert_eq!(result.accepted, 2);
}

#[sqlx::test]
async fn insert_batch_accepts_out_of_order_sequences(pool: sqlx::PgPool) {
    let repo = WorkRunEventsRepository::new();
    let project_id = test_helpers::insert_project_config(&pool, "evt-ooo").await;
    let wr_id = test_helpers::insert_pending_work_run(&pool, project_id, "evt-ooo-task").await;

    let result = repo
        .insert_batch(&pool, wr_id, &[make_event(2, "late.arrival")])
        .await
        .expect("insert");

    assert_eq!(result.accepted, 1);

    let result = repo
        .insert_batch(&pool, wr_id, &[make_event(1, "early.arrival")])
        .await
        .expect("insert");

    assert_eq!(result.accepted, 1);
}

#[sqlx::test]
async fn insert_batch_duplicate_is_skipped(pool: sqlx::PgPool) {
    let repo = WorkRunEventsRepository::new();
    let project_id = test_helpers::insert_project_config(&pool, "evt-dup").await;
    let wr_id = test_helpers::insert_pending_work_run(&pool, project_id, "evt-dup-task").await;

    let r1 = repo
        .insert_batch(&pool, wr_id, &[make_event(1, "session.started")])
        .await
        .expect("first insert");

    assert_eq!(r1.accepted, 1);

    let r2 = repo
        .insert_batch(&pool, wr_id, &[make_event(1, "session.started")])
        .await
        .expect("duplicate should be silently ignored");

    assert_eq!(r2.accepted, 0);
}

#[sqlx::test]
async fn insert_batch_empty(pool: sqlx::PgPool) {
    let repo = WorkRunEventsRepository::new();
    let project_id = test_helpers::insert_project_config(&pool, "evt-empty").await;
    let wr_id = test_helpers::insert_pending_work_run(&pool, project_id, "evt-empty-task").await;

    let result = repo.insert_batch(&pool, wr_id, &[]).await.expect("empty");

    assert_eq!(result.accepted, 0);
}

#[sqlx::test]
async fn find_after_returns_ordered_results(pool: sqlx::PgPool) {
    let repo = WorkRunEventsRepository::new();
    let project_id = test_helpers::insert_project_config(&pool, "evt-find").await;
    let wr_id = test_helpers::insert_pending_work_run(&pool, project_id, "evt-find-task").await;

    let events: Vec<InsertEventParams> = (1..=5).map(|i| make_event(i, &format!("e{i}"))).collect();
    repo.insert_batch(&pool, wr_id, &events)
        .await
        .expect("insert");

    let first = repo
        .find_after(
            &pool,
            wr_id,
            Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap(),
            0,
            100,
        )
        .await
        .expect("find");
    let sequences: Vec<i64> = first.iter().map(|e: &WorkRunEvent| e.sequence).collect();
    assert_eq!(sequences, vec![1, 2, 3, 4, 5]);
}

#[sqlx::test]
async fn find_last_n_returns_ascending(pool: sqlx::PgPool) {
    use chrono::TimeZone;

    let repo = WorkRunEventsRepository::new();
    let project_id = test_helpers::insert_project_config(&pool, "evt-last").await;
    let wr_id = test_helpers::insert_pending_work_run(&pool, project_id, "evt-last-task").await;

    let mut events: Vec<InsertEventParams> = Vec::new();
    for i in 1..=10 {
        events.push(InsertEventParams {
            sequence: i,
            event_type: format!("e{i}"),
            payload: json!({"i": i}),
            occurred_at: Utc.with_ymd_and_hms(2025, 1, i as u32, 0, 0, 0).unwrap(),
        });
    }
    repo.insert_batch(&pool, wr_id, &events)
        .await
        .expect("insert");

    let last = repo.find_last_n(&pool, wr_id, 3).await.expect("last");
    let sequences: Vec<i64> = last.iter().map(|e: &WorkRunEvent| e.sequence).collect();
    assert_eq!(sequences, vec![8, 9, 10]);
}

#[sqlx::test]
async fn find_after_for_unknown_work_run_is_empty(pool: sqlx::PgPool) {
    let repo = WorkRunEventsRepository::new();
    let events = repo
        .find_after(&pool, Uuid::new_v4(), Utc::now(), 0, 10)
        .await
        .expect("find");
    assert!(events.is_empty());
}
