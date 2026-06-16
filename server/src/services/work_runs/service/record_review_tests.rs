use chrono::Utc;
use uuid::Uuid;
use vulcanum_shared::api_types::SubmitResultRequest;

use crate::services::work_runs::model::{WorkRun, WorkRunStatus, WorkRunType};
use crate::services::work_runs::service::record_review::review_comment;

#[test]
fn review_comment_reports_posted_review_with_url() {
    let run = review_run(Some("https://github.com/acme/app/pull/7"));
    let params = submit_params(
        false,
        Some("https://github.com/acme/app/pull/7#pullrequestreview-1"),
    );

    assert_eq!(
        review_comment(&run, &params),
        "Review posted for https://github.com/acme/app/pull/7: https://github.com/acme/app/pull/7#pullrequestreview-1"
    );
}

#[test]
fn review_comment_reports_posted_review_without_url() {
    let run = review_run(Some("https://github.com/acme/app/pull/7"));
    let params = submit_params(false, None);

    assert_eq!(
        review_comment(&run, &params),
        "Review posted for https://github.com/acme/app/pull/7"
    );
}

#[test]
fn review_comment_reports_existing_review_with_url() {
    let run = review_run(Some("https://github.com/acme/app/pull/7"));
    let params = submit_params(
        true,
        Some("https://github.com/acme/app/pull/7#pullrequestreview-1"),
    );

    assert_eq!(
        review_comment(&run, &params),
        "Review already existed for https://github.com/acme/app/pull/7: https://github.com/acme/app/pull/7#pullrequestreview-1"
    );
}

#[test]
fn review_comment_reports_existing_review_without_target_pr() {
    let run = review_run(None);
    let params = submit_params(true, None);

    assert_eq!(
        review_comment(&run, &params),
        "Review already existed for the pull request"
    );
}

fn review_run(review_target_pr_url: Option<&str>) -> WorkRun {
    let now = Utc::now();

    WorkRun {
        id: Uuid::new_v4(),
        team_id: Uuid::new_v4(),
        external_task_ref: "task-1".to_owned(),
        project_config_id: Uuid::new_v4(),
        worker_id: None,
        status: WorkRunStatus::Running,
        work_type: WorkRunType::PullRequestReview,
        parent_work_run_id: None,
        prompt_text: String::new(),
        repo_url: String::new(),
        agents_md: String::new(),
        task_body: String::new(),
        task_title: None,
        task_slug: None,
        review_target_pr_url: review_target_pr_url.map(str::to_owned),
        review_target_repo_full_name: None,
        review_url: None,
        review_body: None,
        review_already_exists: false,
        result_pr_url: None,
        result_exit_code: None,
        tokens_used: None,
        duration_ms: None,
        input_tokens: None,
        output_tokens: None,
        cache_read_tokens: None,
        cache_write_tokens: None,
        model_used: None,
        finish_status: None,
        finish_summary: None,
        finish_blocked_reason: None,
        finish_next_column: None,
        created_at: now,
        updated_at: now,
    }
}

fn submit_params(review_already_exists: bool, review_url: Option<&str>) -> SubmitResultRequest {
    SubmitResultRequest {
        pr_urls: Vec::new(),
        pr_url: String::new(),
        exit_code: 0,
        tokens_used: 0,
        duration_ms: 0,
        input_tokens: 0,
        output_tokens: 0,
        cache_read_tokens: 0,
        cache_write_tokens: 0,
        model_used: None,
        finish_status: None,
        finish_summary: None,
        finish_blocked_reason: None,
        finish_next_column: None,
        review_url: review_url.map(str::to_owned),
        review_body: None,
        review_already_exists,
    }
}
