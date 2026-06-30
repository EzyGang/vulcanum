use chrono::Utc;
use uuid::Uuid;

use crate::state::journal::{Journal, JournalResultUpdate, JournalStatus};

fn open_journal() -> Journal {
    let tmp = std::env::temp_dir().join(format!("vulcanum-journal-test-{}", Uuid::new_v4()));
    Journal::open(&tmp).expect("should open journal")
}

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

#[test]
fn update_result_persists_granular_tokens() {
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
            tokens_used: 1_200,
            input_tokens: 700,
            output_tokens: 300,
            cache_read_tokens: 150,
            cache_write_tokens: 50,
            pr_url: Some("https://github.com/EzyGang/vulcanum/pull/1"),
            duration_ms: 12_345,
            status: JournalStatus::Running,
        })
        .expect("should update");

    let running = journal.list_running().expect("should list");
    let entry = running
        .into_iter()
        .find(|e| e.job_id == job_id)
        .expect("entry found");

    assert_eq!(entry.tokens_used, Some(1_200));
    assert_eq!(entry.input_tokens, Some(700));
    assert_eq!(entry.output_tokens, Some(300));
    assert_eq!(entry.cache_read_tokens, Some(150));
    assert_eq!(entry.cache_write_tokens, Some(50));
}

#[test]
fn multiple_jobs_with_mixed_statuses() {
    let journal = open_journal();
    let id1 = Uuid::new_v4();
    let id2 = Uuid::new_v4();
    let id3 = Uuid::new_v4();

    journal
        .insert_job(crate::state::journal::JournalInsert {
            job_id: id1,
            workdir: "/tmp/a",
            container_name: None,
            harness_type: "host",
            started_at: Utc::now(),
            max_turns: 1,
            agent_backend: "opencode",
        })
        .expect("insert 1");
    journal
        .insert_job(crate::state::journal::JournalInsert {
            job_id: id2,
            workdir: "/tmp/b",
            container_name: None,
            harness_type: "host",
            started_at: Utc::now(),
            max_turns: 1,
            agent_backend: "opencode",
        })
        .expect("insert 2");
    journal
        .insert_job(crate::state::journal::JournalInsert {
            job_id: id3,
            workdir: "/tmp/c",
            container_name: None,
            harness_type: "host",
            started_at: Utc::now(),
            max_turns: 1,
            agent_backend: "opencode",
        })
        .expect("insert 3");

    journal
        .update_result(JournalResultUpdate {
            job_id: id1,
            exit_code: 0,
            tokens_used: 500,
            input_tokens: 250,
            output_tokens: 150,
            cache_read_tokens: 75,
            cache_write_tokens: 25,
            pr_url: None,
            duration_ms: 10_000,
            status: JournalStatus::Completed,
        })
        .expect("complete 1");

    let running = journal.list_running().expect("should list");
    assert_eq!(running.len(), 2);
}

#[test]
fn journal_persists_host_info() {
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
        .expect("insert job");

    journal
        .set_host_info(job_id, 12_345, 5555)
        .expect("set host info");

    let running = journal.list_running().expect("list running");
    let entry = running
        .into_iter()
        .find(|e| e.job_id == job_id)
        .expect("entry found");

    assert_eq!(entry.host_pid, Some(12_345));
    assert_eq!(entry.host_port, Some(5555));
}

#[test]
fn journal_persists_agent_metadata() {
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
            agent_backend: "omp_rpc",
        })
        .expect("insert job");

    journal
        .set_agent_metadata(
            job_id,
            Some("/tmp/work/home/.omp/sessions/session.jsonl"),
            Some("/tmp/work/home/.omp"),
            Some("/tmp/work/home/.local/state/omp"),
            Some("omp_rpc"),
            Some(1234),
        )
        .expect("set metadata");

    let entry = journal
        .find_by_id(job_id)
        .expect("find succeeds")
        .expect("entry exists");
    assert_eq!(entry.agent_backend.as_deref(), Some("omp_rpc"));
    assert_eq!(
        entry.agent_session_path.as_deref(),
        Some("/tmp/work/home/.omp/sessions/session.jsonl")
    );
    assert_eq!(entry.agent_pid, Some(1234));
}
