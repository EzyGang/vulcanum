use super::*;

#[test]
fn insert_and_list_running() {
    let journal = open_journal();
    let job_id = Uuid::new_v4();

    journal
        .insert_job(crate::state::journal::JournalInsert {
            job_id,
            workdir: "/tmp/work",
            container_name: Some("vulcanum-container"),
            harness_type: "kata",
            started_at: Utc::now(),
            max_turns: 1,
            agent_backend: "opencode",
        })
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
fn find_by_id_returns_existing_entry() {
    let journal = open_journal();
    let job_id = Uuid::new_v4();

    journal
        .insert_job(crate::state::journal::JournalInsert {
            job_id,
            workdir: "/tmp/work",
            container_name: None,
            harness_type: "host",
            started_at: Utc::now(),
            max_turns: 2,
            agent_backend: "opencode",
        })
        .expect("should insert");

    let entry = journal
        .find_by_id(job_id)
        .expect("should find")
        .expect("entry exists");

    assert_eq!(entry.job_id, job_id);
    assert_eq!(entry.workdir, "/tmp/work");
    assert_eq!(entry.status, JournalStatus::Running);
    assert_eq!(entry.max_turns, Some(2));
}

#[test]
fn find_by_id_returns_none_for_missing_entry() {
    let journal = open_journal();

    let entry = journal
        .find_by_id(Uuid::new_v4())
        .expect("should query missing entry");

    assert!(entry.is_none());
}

#[test]
fn update_result_transitions_status() {
    let journal = open_journal();
    let job_id = Uuid::new_v4();

    journal
        .insert_job(crate::state::journal::JournalInsert {
            job_id,
            workdir: "/tmp/work",
            container_name: None,
            harness_type: "host",
            started_at: Utc::now(),
            max_turns: 1,
            agent_backend: "opencode",
        })
        .expect("should insert");

    journal
        .update_result(JournalResultUpdate {
            job_id,
            exit_code: 0,
            tokens_used: 100,
            input_tokens: 60,
            output_tokens: 30,
            cache_read_tokens: 8,
            cache_write_tokens: 2,
            pr_url: None,
            duration_ms: 5_000,
            review_url: None,
            review_body: None,
            review_already_exists: false,
            status: JournalStatus::Completed,
        })
        .expect("should update");

    let running = journal.list_running().expect("should list");
    assert!(running.is_empty());
}

#[test]
fn mark_lost_sets_status_and_error() {
    let journal = open_journal();
    let job_id = Uuid::new_v4();

    journal
        .insert_job(crate::state::journal::JournalInsert {
            job_id,
            workdir: "/tmp/work",
            container_name: None,
            harness_type: "host",
            started_at: Utc::now(),
            max_turns: 1,
            agent_backend: "opencode",
        })
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
        .insert_job(crate::state::journal::JournalInsert {
            job_id,
            workdir: "/tmp/work",
            container_name: None,
            harness_type: "host",
            started_at: Utc::now(),
            max_turns: 1,
            agent_backend: "opencode",
        })
        .expect("should insert");

    journal
        .update_result(JournalResultUpdate {
            job_id,
            exit_code: 0,
            tokens_used: 100,
            input_tokens: 60,
            output_tokens: 30,
            cache_read_tokens: 8,
            cache_write_tokens: 2,
            pr_url: None,
            duration_ms: 5_000,
            review_url: None,
            review_body: None,
            review_already_exists: false,
            status: JournalStatus::Completed,
        })
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
