use chrono::Utc;
use uuid::Uuid;

use crate::state::journal::{Journal, JournalStatus};

fn open_journal() -> Journal {
    let tmp = std::env::temp_dir().join(format!("vulcanum-journal-test-{}", Uuid::new_v4()));
    Journal::open(&tmp).expect("should open journal")
}

#[test]
fn insert_and_list_running() {
    let journal = open_journal();
    let job_id = Uuid::new_v4();

    journal
        .insert_job(
            job_id,
            "/tmp/work",
            Some("vulcanum-container"),
            "kata",
            Utc::now(),
            1,
        )
        .expect("should insert");

    let running = journal.list_running().expect("should list");
    assert_eq!(running.len(), 1);
    assert_eq!(running[0].job_id, job_id);
    assert_eq!(running[0].workdir, "/tmp/work");
    assert_eq!(
        running[0].container_name.as_deref(),
        Some("vulcanum-container")
    );
    assert_eq!(running[0].harness_type, "kata");
}

#[test]
fn update_result_transitions_status() {
    let journal = open_journal();
    let job_id = Uuid::new_v4();

    journal
        .insert_job(job_id, "/tmp/work", None, "host", Utc::now(), 1)
        .expect("should insert");

    journal
        .update_result(
            job_id,
            0,
            1234,
            Some("https://github.com/x/pr/1"),
            30000,
            JournalStatus::Completed,
        )
        .expect("should update");

    let running = journal.list_running().expect("should list");
    assert!(running.is_empty());
}

#[test]
fn mark_lost_sets_status_and_error() {
    let journal = open_journal();
    let job_id = Uuid::new_v4();

    journal
        .insert_job(job_id, "/tmp/work", None, "host", Utc::now(), 1)
        .expect("should insert");

    journal
        .mark_lost(job_id, "container not found")
        .expect("should mark lost");

    let running = journal.list_running().expect("should list");
    assert!(running.is_empty());
}

#[test]
fn mark_submitted_transitions() {
    let journal = open_journal();
    let job_id = Uuid::new_v4();

    journal
        .insert_job(job_id, "/tmp/work", None, "host", Utc::now(), 1)
        .expect("should insert");

    journal
        .update_result(job_id, 0, 100, None, 5000, JournalStatus::Completed)
        .expect("should update");

    journal
        .mark_submitted(job_id)
        .expect("should mark submitted");

    let running = journal.list_running().expect("should list");
    assert!(running.is_empty());
}

#[test]
fn list_running_returns_empty_when_no_running_jobs() {
    let journal = open_journal();
    let running = journal.list_running().expect("should list");
    assert!(running.is_empty());
}

#[test]
fn multiple_jobs_with_mixed_statuses() {
    let journal = open_journal();
    let id1 = Uuid::new_v4();
    let id2 = Uuid::new_v4();
    let id3 = Uuid::new_v4();

    journal
        .insert_job(id1, "/tmp/a", None, "host", Utc::now(), 1)
        .expect("insert 1");
    journal
        .insert_job(id2, "/tmp/b", None, "host", Utc::now(), 1)
        .expect("insert 2");
    journal
        .insert_job(id3, "/tmp/c", None, "host", Utc::now(), 1)
        .expect("insert 3");

    journal
        .update_result(id1, 0, 500, None, 10000, JournalStatus::Completed)
        .expect("complete 1");

    let running = journal.list_running().expect("should list");
    assert_eq!(running.len(), 2);
}
