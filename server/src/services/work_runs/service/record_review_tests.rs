use chrono::Utc;
use uuid::Uuid;
use vulcanum_shared::api_types::SubmitResultRequest;

use crate::services::work_runs::model::{WorkRun, WorkRunStatus, WorkRunType};
use crate::services::work_runs::repository::queries::InsertWorkRunParams;
use crate::services::work_runs::repository::WorkRunsRepository;
use crate::services::work_runs::service::record_review::review_comment;
use crate::services::work_runs::service::review_feedback::review_requires_implementation;
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

#[test]
fn review_requires_implementation_for_critical_items() {
    let body = "## CRITICAL\n- Data loss on retry\n\n## WARNINGS\n- None\n\n## SUGGESTIONS\n- Rename helper";

    assert!(review_requires_implementation(body));
}

#[test]
fn review_requires_implementation_for_warning_items() {
    let body = "## CRITICAL\n- None\n\n## WARNINGS\n- Missing authorization check\n\n## SUGGESTIONS\n- None";

    assert!(review_requires_implementation(body));
}

#[test]
fn review_does_not_require_implementation_for_suggestions_only() {
    let body =
        "## CRITICAL\n- None\n\n## WARNINGS\n- No warnings\n\n## SUGGESTIONS\n- Add a helper later";

    assert!(!review_requires_implementation(body));
}

#[sqlx::test]
async fn actionable_review_enqueues_fix_run_for_existing_pr(pool: sqlx::PgPool) {
    let state = test_helpers::build_state(pool.clone()).await;
    let project_config_id = test_helpers::insert_project_config(&pool, "review-fix-project").await;
    let run = WorkRunsRepository::new()
        .insert_work_run(
            &pool,
            InsertWorkRunParams {
                team_id: test_helpers::DEFAULT_TEAM_ID,
                external_task_ref: "task-review-fix".to_owned(),
                project_config_id,
                prompt_text: "Review".to_owned(),
                repo_url: "https://github.com/acme/app".to_owned(),
                repo_full_names: vec!["acme/app".to_owned()],
                agents_md: String::new(),
                status: WorkRunStatus::Completed,
                work_type: WorkRunType::PullRequestReview,
                parent_work_run_id: None,
                task_body: "Fix the issue".to_owned(),
                task_title: Some("Fix issue".to_owned()),
                task_slug: Some("APP-1".to_owned()),
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
    params.review_body = Some(
        "## CRITICAL\n- None\n\n## WARNINGS\n- Missing authorization check\n\n## SUGGESTIONS\n- None"
            .to_owned(),
    );

    state.jobs.record_review_result(&run, &params).await;

    let fix_run = sqlx::query!(
        r#"SELECT status as "status: WorkRunStatus", work_type as "work_type: WorkRunType",
           parent_work_run_id, prompt_text, review_target_pr_url, review_target_repo_full_name
           FROM work_runs WHERE parent_work_run_id = $1"#,
        run.id,
    )
    .fetch_one(&pool)
    .await
    .expect("fix run should be inserted");

    assert!(matches!(fix_run.status, WorkRunStatus::Pending));
    assert!(matches!(fix_run.work_type, WorkRunType::Implementation));
    assert_eq!(fix_run.parent_work_run_id, Some(run.id));
    assert_eq!(
        fix_run.review_target_pr_url.as_deref(),
        Some("https://github.com/acme/app/pull/7")
    );
    assert_eq!(
        fix_run.review_target_repo_full_name.as_deref(),
        Some("acme/app")
    );
    assert!(fix_run.prompt_text.contains("Missing authorization check"));
    assert!(fix_run
        .prompt_text
        .contains("Do not create a new pull request"));
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
        review_url: review_url.map(str::to_owned),
        review_body: None,
        review_already_exists,
    }
}
