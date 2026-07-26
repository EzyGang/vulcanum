use super::*;

#[test]
fn update_result_preserves_null_pr_url_and_reads_legacy_empty_pr_url_as_none() {
    let path = journal_path();
    let journal = open_journal_at(&path);
    let job_id = Uuid::new_v4();
    insert_running_job(&journal, job_id);

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
        .expect("result should update");

    {
        let conn = Connection::open(&path).expect("journal db should open");
        let raw_pr_url: Option<String> = conn
            .query_row(
                "SELECT pr_url FROM job_journal WHERE job_id = ?1",
                [job_id.to_string()],
                |row| row.get(0),
            )
            .expect("raw pr_url should be readable");
        assert!(raw_pr_url.is_none());
        conn.execute(
            "UPDATE job_journal SET pr_url = '' WHERE job_id = ?1",
            [job_id.to_string()],
        )
        .expect("legacy pr_url should be written");
    }

    let entry = journal
        .find_by_id(job_id)
        .expect("entry should be readable")
        .expect("entry should exist");

    assert_eq!(entry.pr_url, None);
}

#[test]
fn list_running_errors_when_running_row_uuid_is_corrupt() {
    let path = journal_path();
    let journal = open_journal_at(&path);
    let job_id = Uuid::new_v4();
    insert_running_job(&journal, job_id);
    let conn = Connection::open(&path).expect("journal db should open");
    conn.execute(
        "UPDATE job_journal SET job_id = 'not-a-uuid' WHERE job_id = ?1",
        [job_id.to_string()],
    )
    .expect("corrupt job_id should be written");

    let err = journal
        .list_running()
        .expect_err("corrupt running row UUID should be surfaced");

    assert!(err.to_string().contains("Conversion error"));
}

#[test]
fn find_by_id_errors_when_journal_status_is_corrupt() {
    let path = journal_path();
    let journal = open_journal_at(&path);
    let job_id = Uuid::new_v4();
    insert_running_job(&journal, job_id);
    let conn = Connection::open(&path).expect("journal db should open");
    conn.execute(
        "UPDATE job_journal SET status = 'wedged' WHERE job_id = ?1",
        [job_id.to_string()],
    )
    .expect("corrupt status should be written");

    let err = journal
        .find_by_id(job_id)
        .expect_err("corrupt journal status should be surfaced");

    assert!(err.to_string().contains("invalid journal status"));
}

#[test]
fn list_running_errors_when_running_row_started_at_is_corrupt() {
    let path = journal_path();
    let journal = open_journal_at(&path);
    let job_id = Uuid::new_v4();
    insert_running_job(&journal, job_id);
    let conn = Connection::open(&path).expect("journal db should open");
    conn.execute(
        "UPDATE job_journal SET started_at = 'not-a-timestamp' WHERE job_id = ?1",
        [job_id.to_string()],
    )
    .expect("corrupt started_at should be written");

    let err = journal
        .list_running()
        .expect_err("corrupt running row timestamp should be surfaced");

    assert!(err.to_string().contains("Conversion error"));
}
