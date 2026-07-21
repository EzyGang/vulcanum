use chrono::Utc;
use uuid::Uuid;
use vulcanum_shared::api::wire::SubmitResultRequest;

use crate::db::work_runs::queries::InsertWorkRunParams;
use crate::db::work_runs::WorkRunsRepository;
use crate::models::work_runs::model::{WorkRun, WorkRunStatus, WorkRunType};
use crate::services::work_runs::service::record_review::review_comment;
use crate::test_helpers;

#[test]
fn review_comment_reports_posted_review_url_with_target_pr() {
    let run = review_run(Some("https://github.com/acme/app/pull/7"));
    let params = submit_params(
        Some("Reviewed PR"),
        Some("https://github.com/acme/app/pull/7#pullrequestreview-1"),
        Some("Looks good"),
        false,
    );

    assert_eq!(
        review_comment(&run, &params),
        "Review posted for https://github.com/acme/app/pull/7: https://github.com/acme/app/pull/7#pullrequestreview-1"
    );
}

#[test]
fn review_comment_reports_existing_review_url() {
    let run = review_run(Some("https://github.com/acme/app/pull/7"));
    let params = submit_params(
        Some("Reviewed PR"),
        Some("https://github.com/acme/app/pull/7#pullrequestreview-1"),
        Some("Looks good"),
        true,
    );

    assert_eq!(
        review_comment(&run, &params),
        "Review already existed for https://github.com/acme/app/pull/7: https://github.com/acme/app/pull/7#pullrequestreview-1"
    );
}

#[test]
fn review_comment_reports_without_review_url() {
    let run = review_run(None);
    let params = submit_params(None, None, None, false);

    assert_eq!(
        review_comment(&run, &params),
        "Review posted for the pull request"
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
                task_title: None,
                task_slug: None,
                project_config_id,
                repo_full_names: vec!["acme/app".to_owned()],
                status: WorkRunStatus::Completed,
                work_type: WorkRunType::PullRequestReview,
                parent_work_run_id: None,
                review_target_pr_url: Some("https://github.com/acme/app/pull/7".to_owned()),
                review_target_repo_full_name: Some("acme/app".to_owned()),
                github_installation_id: None,
                github_delivery_id: None,
            },
        )
        .await
        .expect("review run should insert");
    let params = submit_params(
        Some("Reviewed PR"),
        Some("https://github.com/acme/app/pull/7#pullrequestreview-1"),
        Some("## CRITICAL\n- None\n\n## WARNINGS\n- Missing authorization check\n\n## SUGGESTIONS\n- None"),
        false,
    );

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
    assert_eq!(review.review_body, params.review_body);
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
        task_title: None,
        task_slug: None,
        project_config_id: Uuid::new_v4(),
        worker_id: None,
        status: WorkRunStatus::Running,
        work_type: WorkRunType::PullRequestReview,
        parent_work_run_id: None,
        review_target_pr_url: review_target_pr_url.map(str::to_owned),
        review_target_repo_full_name: None,
        github_installation_id: None,
        github_delivery_id: None,
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

fn submit_params(
    result_summary: Option<&str>,
    review_url: Option<&str>,
    review_body: Option<&str>,
    review_already_exists: bool,
) -> SubmitResultRequest {
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
        result_summary: result_summary.map(str::to_owned),
        review_url: review_url.map(str::to_owned),
        review_body: review_body.map(str::to_owned),
        review_already_exists,
    }
}
