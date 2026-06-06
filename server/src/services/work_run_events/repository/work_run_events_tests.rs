use serde_json::json;
use uuid::Uuid;

use crate::services::work_run_events::errors::WorkRunEventsError;
use crate::services::work_run_events::model::WorkRunEvent;
use crate::services::work_run_events::repository::queries::InsertEventParams;
use crate::services::work_run_events::repository::WorkRunEventsRepository;
use crate::test_helpers;

fn make_event(seq: i64, event_type: &str) -> InsertEventParams {
    InsertEventParams {
        sequence: seq,
        event_type: event_type.to_owned(),
        payload: json!({"i": seq}),
    }
}

#[sqlx::test]
async fn max_sequence_starts_at_zero(pool: sqlx::PgPool) {
    let repo = WorkRunEventsRepository::new();
    let project_id = test_helpers::insert_project_config(&pool, "evt-empty").await;
    let wr_id = test_helpers::insert_pending_work_run(&pool, project_id, "evt-empty-task").await;

    let max = repo.max_sequence(&pool, wr_id).await.expect("max");
    assert_eq!(max, 0);
}

#[sqlx::test]
async fn insert_batch_first_event_must_be_one(pool: sqlx::PgPool) {
    let repo = WorkRunEventsRepository::new();
    let project_id = test_helpers::insert_project_config(&pool, "evt-fir").await;
    let wr_id = test_helpers::insert_pending_work_run(&pool, project_id, "evt-fir-task").await;

    let events = vec![make_event(2, "session.started")];
    let err = repo
        .insert_batch(&pool, wr_id, &events)
        .await
        .expect_err("must reject sequence 2 when nothing exists");
    match err {
        WorkRunEventsError::OutOfOrderSequence {
            next_expected_sequence,
        } => {
            assert_eq!(next_expected_sequence, 1);
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[sqlx::test]
async fn insert_batch_happy_path(pool: sqlx::PgPool) {
    let repo = WorkRunEventsRepository::new();
    let project_id = test_helpers::insert_project_config(&pool, "evt-happy").await;
    let wr_id = test_helpers::insert_pending_work_run(&pool, project_id, "evt-happy-task").await;

    let events = vec![
        make_event(1, "session.started"),
        make_event(2, "turn.completed"),
        make_event(3, "session.completed"),
    ];

    let result = repo
        .insert_batch(&pool, wr_id, &events)
        .await
        .expect("insert");

    assert_eq!(result.accepted, 3);
    assert_eq!(result.next_expected_sequence, 4);
}

#[sqlx::test]
async fn insert_batch_rejects_duplicate(pool: sqlx::PgPool) {
    let repo = WorkRunEventsRepository::new();
    let project_id = test_helpers::insert_project_config(&pool, "evt-dup").await;
    let wr_id = test_helpers::insert_pending_work_run(&pool, project_id, "evt-dup-task").await;

    repo.insert_batch(&pool, wr_id, &[make_event(1, "session.started")])
        .await
        .expect("first insert");

    let err = repo
        .insert_batch(&pool, wr_id, &[make_event(1, "session.started")])
        .await
        .expect_err("duplicate must be rejected");
    assert!(matches!(err, WorkRunEventsError::OutOfOrderSequence { .. }));
}

#[sqlx::test]
async fn insert_batch_rejects_backwards_sequence_in_payload(pool: sqlx::PgPool) {
    let repo = WorkRunEventsRepository::new();
    let project_id = test_helpers::insert_project_config(&pool, "evt-back").await;
    let wr_id = test_helpers::insert_pending_work_run(&pool, project_id, "evt-back-task").await;

    let err = repo
        .insert_batch(&pool, wr_id, &[make_event(1, "a"), make_event(1, "b")])
        .await
        .expect_err("non-strictly-increasing must be rejected");
    assert!(matches!(err, WorkRunEventsError::OutOfOrderSequence { .. }));
}

#[sqlx::test]
async fn insert_batch_empty_returns_next_expected(pool: sqlx::PgPool) {
    let repo = WorkRunEventsRepository::new();
    let project_id = test_helpers::insert_project_config(&pool, "evt-empty2").await;
    let wr_id = test_helpers::insert_pending_work_run(&pool, project_id, "evt-empty2-task").await;

    let result = repo.insert_batch(&pool, wr_id, &[]).await.expect("empty");

    assert_eq!(result.accepted, 0);
    assert_eq!(result.next_expected_sequence, 1);
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

    let after = repo.find_after(&pool, wr_id, 2, 100).await.expect("find");
    let sequences: Vec<i64> = after.iter().map(|e: &WorkRunEvent| e.sequence).collect();
    assert_eq!(sequences, vec![3, 4, 5]);
}

#[sqlx::test]
async fn find_last_n_returns_ascending(pool: sqlx::PgPool) {
    let repo = WorkRunEventsRepository::new();
    let project_id = test_helpers::insert_project_config(&pool, "evt-last").await;
    let wr_id = test_helpers::insert_pending_work_run(&pool, project_id, "evt-last-task").await;

    let events: Vec<InsertEventParams> =
        (1..=10).map(|i| make_event(i, &format!("e{i}"))).collect();
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
        .find_after(&pool, Uuid::new_v4(), 0, 10)
        .await
        .expect("find");
    assert!(events.is_empty());
}
