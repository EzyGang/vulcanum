use std::sync::Arc;

use chrono::Utc;
use rusqlite::Connection;
use tokio::sync::RwLock;
use uuid::Uuid;

use vulcanum_shared::client::ApiClient;
use vulcanum_shared::runtime::types::{FinishRunArtifact, FinishStatus};
use vulcanum_shared::state::worker::WorkerState;

use crate::daemon::job::review::review_loop::ReviewLoopCheckpoint;
use crate::daemon::queue::JobTracker;
use crate::recovery::{reconcile_running_jobs, should_resume_review_artifact};
use crate::state::journal::{Journal, JournalInsert};

#[test]
fn actionable_review_artifact_resumes_reconciliation() {
    let artifact = FinishRunArtifact {
        status: FinishStatus::Completed,
        pr_urls: Vec::new(),
        summary: None,
        blocked_reason: None,
        review_url: Some("https://github.com/acme/app/pull/1#pullrequestreview-1".to_owned()),
        review_body: Some(
            "## CRITICAL\n- None\n\n## WARNINGS\n- Missing validation\n\n## SUGGESTIONS\n- None"
                .to_owned(),
        ),
        review_already_exists: true,
    };

    assert!(should_resume_review_artifact(
        &artifact,
        1,
        ReviewLoopCheckpoint::default(),
    ));
    assert!(!should_resume_review_artifact(
        &artifact,
        1,
        ReviewLoopCheckpoint {
            fix_pass: 1,
            fixing: false,
        },
    ));
}

#[tokio::test]
async fn corrupt_running_state_blocks_reconciliation() {
    let temporary = tempfile::tempdir().expect("temporary directory should be created");
    let journal_path = temporary.path().join("worker.db");
    let journal = Arc::new(Journal::open(&journal_path).expect("journal should open"));
    let job_id = Uuid::new_v4();
    journal
        .insert_job(JournalInsert {
            job_id,
            workdir: "/tmp/work",
            container_name: None,
            harness_type: "host",
            started_at: Utc::now(),
            max_turns: 1,
            agent_backend: "opencode",
            work_type: vulcanum_shared::api::wire::WorkRunType::Implementation,
        })
        .expect("running job should be inserted");
    let connection = Connection::open(&journal_path).expect("journal database should open");
    connection
        .execute(
            "UPDATE job_journal SET job_id = 'not-a-uuid' WHERE job_id = ?1",
            [job_id.to_string()],
        )
        .expect("running job should be corrupted");

    let client = Arc::new(ApiClient::new("http://127.0.0.1:1".to_owned()));
    let worker_state = Arc::new(RwLock::new(WorkerState {
        worker_id: Uuid::new_v4(),
        instance_url: "http://127.0.0.1:1".to_owned(),
        access_token: "access".to_owned(),
        refresh_token: "refresh".to_owned(),
        expires_at: Utc::now(),
        max_concurrent_jobs: 1,
    }));
    let tracker = Arc::new(JobTracker::default());

    let error = reconcile_running_jobs(&journal, &client, &worker_state, &tracker)
        .await
        .expect_err("unknown running state must block startup");

    assert!(error
        .to_string()
        .contains("failed to list running jobs for recovery"));
}
