use chrono::Utc;
use uuid::Uuid;

use crate::daemon::job::execution::submit::{
    submit_result_from_journal, submit_result_request, SubmitResultParams,
};
use crate::state::journal::{JournalEntry, JournalStatus};

#[test]
fn submit_result_includes_review_fields() {
    let request = submit_result_request(SubmitResultParams {
        pr_urls: Vec::new(),
        exit_code: 0,
        tokens_used: 10,
        duration_ms: 100,
        input_tokens: 1,
        output_tokens: 2,
        cache_read_tokens: 3,
        cache_write_tokens: 4,
        model_used: None,
        finish_status: None,
        finish_summary: None,
        review_url: Some("https://github.com/acme/widgets/pull/42#pullrequestreview-1".to_owned()),
        review_body: Some("Looks good".to_owned()),
        review_already_exists: true,
    });

    assert_eq!(
        request.review_url.as_deref(),
        Some("https://github.com/acme/widgets/pull/42#pullrequestreview-1")
    );
    assert_eq!(request.review_body.as_deref(), Some("Looks good"));
    assert!(request.review_already_exists);
}

#[test]
fn submit_result_from_journal_replays_stored_result() {
    let entry = JournalEntry {
        job_id: Uuid::new_v4(),
        workdir: "/tmp/vulcanum-work-test".to_owned(),
        container_name: None,
        harness_type: "host".to_owned(),
        status: JournalStatus::Submitted,
        started_at: Utc::now(),
        finished_at: Some(Utc::now()),
        exit_code: Some(0),
        tokens_used: Some(100),
        input_tokens: Some(60),
        output_tokens: Some(30),
        cache_read_tokens: Some(7),
        cache_write_tokens: Some(3),
        pr_url: Some("https://github.com/EzyGang/vulcanum/pull/101".to_owned()),
        duration_ms: Some(5_000),
        error_message: None,
        turn_count: Some(1),
        session_id: Some("ses_1".to_owned()),
        max_turns: Some(3),
        host_pid: None,
        host_port: None,
    };

    let request = submit_result_from_journal(&entry);

    assert_eq!(request.exit_code, 0);
    assert_eq!(request.tokens_used, 100);
    assert_eq!(request.input_tokens, 60);
    assert_eq!(request.output_tokens, 30);
    assert_eq!(request.cache_read_tokens, 7);
    assert_eq!(request.cache_write_tokens, 3);
    assert_eq!(request.duration_ms, 5_000);
    assert_eq!(
        request.pr_urls,
        vec!["https://github.com/EzyGang/vulcanum/pull/101".to_owned()]
    );
}
