use super::{open_journal, JournalResultUpdate, JournalStatus, Utc, Uuid};

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
            work_type: vulcanum_shared::api::wire::WorkRunType::Implementation,
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
            review_url: Some("https://github.com/EzyGang/vulcanum/pull/1#pullrequestreview-1"),
            review_body: Some("Looks good"),
            review_already_exists: true,
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
    assert_eq!(
        entry.review_url.as_deref(),
        Some("https://github.com/EzyGang/vulcanum/pull/1#pullrequestreview-1")
    );
    assert_eq!(entry.review_body.as_deref(), Some("Looks good"));
    assert!(entry.review_already_exists);
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
            work_type: vulcanum_shared::api::wire::WorkRunType::Implementation,
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
            work_type: vulcanum_shared::api::wire::WorkRunType::Implementation,
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
            work_type: vulcanum_shared::api::wire::WorkRunType::Implementation,
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
            review_url: None,
            review_body: None,
            review_already_exists: false,
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
            work_type: vulcanum_shared::api::wire::WorkRunType::Implementation,
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
            work_type: vulcanum_shared::api::wire::WorkRunType::Implementation,
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

#[test]
fn journal_persists_review_work_type_for_recovery() {
    let journal = open_journal();
    let job_id = Uuid::new_v4();
    journal
        .insert_job(crate::state::journal::JournalInsert {
            job_id,
            workdir: "/tmp/review-work",
            container_name: None,
            harness_type: "host",
            started_at: Utc::now(),
            max_turns: 1,
            agent_backend: "opencode",
            work_type: vulcanum_shared::api::wire::WorkRunType::PullRequestReview,
        })
        .expect("insert review job");

    let entry = journal
        .find_by_id(job_id)
        .expect("find succeeds")
        .expect("entry exists");

    assert_eq!(
        entry.work_type,
        vulcanum_shared::api::wire::WorkRunType::PullRequestReview
    );
}
