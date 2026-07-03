use chrono::Utc;
use uuid::Uuid;
use vulcanum_shared::api_types::SubmitResultRequest;

use crate::db::work_runs::queries::InsertWorkRunParams;
use crate::db::work_runs::WorkRunsRepository;
use crate::models::work_runs::model::{WorkRun, WorkRunStatus, WorkRunType};
use crate::services::work_runs::service::record_review::review_comment;
use crate::test_helpers;

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

#[sqlx::test]
async fn actionable_review_records_result_without_spawning_fix_run(pool: sqlx::PgPool) {
    let state = test_helpers::build_state(pool.clone()).await;
    let project_config_id = test_helpers::insert_project_config(&pool, "review-fix-project").await;
    let run = WorkRunsRepository::new()
        .insert_work_run(
            &pool,
            InsertWorkRunParams {
                team_id: test_helpers::DEFAULT_TEAM_ID,
                external_task_ref: "task-review-fix".to_owned(),
                project_config_id,
                repo_full_names: vec!["acme/app".to_owned()],
                status: WorkRunStatus::Completed,
                work_type: WorkRunType::PullRequestReview,
                parent_work_run_id: None,
                review_target_pr_url: Some("https://github.com/acme/app/pull/7".to_owned()),
                review_target_repo_full_name: Some("acme/app".to_owned()),
            },
        )
        .await
        .expect("review run should insert");
    let mut params = submit_params(
        false,
        Some("https://github.com/acme/app/pull/7#pullrequestreview-1"),
    );
    let review_body =
        "## CRITICAL\n- None\n\n## WARNINGS\n- Missing authorization check\n\n## SUGGESTIONS\n- None"
            .to_owned();
    params
        .review_result
        .as_mut()
        .expect("review result should exist")
        .review_body = Some(review_body.clone());

    state.jobs.record_review_result(&run, &params).await;

    let review = sqlx::query!(
        r#"SELECT review_url, review_body, review_already_exists
           FROM work_run_reviews WHERE work_run_id = $1 AND pr_url = $2"#,
        run.id,
        "https://github.com/acme/app/pull/7",
    )
    .fetch_one(&pool)
    .await
    .expect("review result should be recorded");

    assert_eq!(
        review.review_url.as_deref(),
        Some("https://github.com/acme/app/pull/7#pullrequestreview-1")
    );
    assert_eq!(review.review_body, Some(review_body));
    assert!(!review.review_already_exists);

    let child_count = sqlx::query!(
        "SELECT COUNT(*) as count FROM work_runs WHERE parent_work_run_id = $1",
        run.id,
    )
    .fetch_one(&pool)
    .await
    .expect("child count should load");

    assert_eq!(child_count.count, Some(0));
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
        review_target_pr_url: review_target_pr_url.map(str::to_owned),
        review_target_repo_full_name: None,
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
        result_summary: None,
        finish_blocked_reason: None,
        finish_next_column: None,
        created_at: now,
        updated_at: now,
    }
}

fn submit_params(review_already_exists: bool, review_url: Option<&str>) -> SubmitResultRequest {
    SubmitResultRequest {
        pr_urls: Vec::new(),
        exit_code: 0,
        tokens_used: 0,
        duration_ms: 0,
        input_tokens: 0,
        output_tokens: 0,
        cache_read_tokens: 0,
        cache_write_tokens: 0,
        model_used: None,
        finish_status: None,
        result_summary: None,
        review_result: Some(vulcanum_shared::api_types::SubmitReviewResult {
            review_url: review_url.map(str::to_owned),
            review_body: None,
            review_already_exists,
        }),
    }
}
