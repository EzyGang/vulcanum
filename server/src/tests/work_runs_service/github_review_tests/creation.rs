use std::sync::Arc;

use crate::models::work_runs::model::{WorkRunStatus, WorkRunType};
use crate::services::work_runs::service::request_github_review::{
    GithubReviewRequest, GithubReviewRequestOutcome,
};
use crate::services::work_runs::service::review_ticket::{
    review_ticket_input, review_ticket_marker,
};
use crate::test_helpers;

use super::support::{setup_review_project, MockReviewTicketCreator, INSTALLATION_ID, SENDER_ID};

#[sqlx::test]
async fn github_review_request_creates_standalone_review(pool: sqlx::PgPool) {
    let project_id = setup_review_project(&pool).await;
    let mut state = test_helpers::build_state(pool.clone()).await;
    state.jobs = state
        .jobs
        .clone()
        .with_review_ticket_creator(Arc::new(MockReviewTicketCreator::default()));

    let project = state
        .jobs
        .project_configs
        .find_by_id(project_id)
        .await
        .expect("review project");
    let ticket_input = review_ticket_input(&project, "acme/widgets", 42, "Review me");
    assert_eq!(ticket_input.project_id, "github-review-project");
    assert_eq!(ticket_input.status, "in review");
    assert_eq!(ticket_input.title, "Review PR #42: Review me");
    assert!(ticket_input
        .body
        .contains("Review pull request: https://github.com/acme/widgets/pull/42"));
    assert!(ticket_input
        .body
        .contains(&review_ticket_marker(project_id, "acme/widgets", 42)));

    let outcome = state
        .jobs
        .request_github_review(GithubReviewRequest {
            delivery_id: "delivery-1",
            installation_id: INSTALLATION_ID,
            sender_id: SENDER_ID,
            single_user_mode: false,
            repo_full_name: "Acme/Widgets",
            pr_number: 42,
            pr_title: "Review me",
            project_selector: None,
        })
        .await
        .expect("request review");

    assert_eq!(outcome, GithubReviewRequestOutcome::Spawned);
    let run = sqlx::query!(
        r#"SELECT team_id, external_task_ref, task_title, task_slug, project_config_id,
           status as "status: WorkRunStatus", work_type as "work_type: WorkRunType",
           parent_work_run_id, review_target_pr_url, review_target_repo_full_name,
           github_installation_id
           FROM work_runs WHERE github_delivery_id = $1"#,
        "delivery-1",
    )
    .fetch_one(&pool)
    .await
    .expect("standalone review row");
    assert_eq!(run.team_id, test_helpers::DEFAULT_TEAM_ID);
    assert_eq!(run.project_config_id, project_id);
    assert_eq!(run.external_task_ref, "review-ticket-acme/widgets-42");
    assert_eq!(run.task_title.as_deref(), Some("Review PR #42: Review me"));
    assert_eq!(run.task_slug.as_deref(), Some("Acme/Widgets#42"));
    assert_eq!(run.status, WorkRunStatus::Pending);
    assert_eq!(run.work_type, WorkRunType::PullRequestReview);
    assert_eq!(run.parent_work_run_id, None);
    assert_eq!(
        run.review_target_pr_url.as_deref(),
        Some("https://github.com/acme/widgets/pull/42")
    );
    assert_eq!(
        run.review_target_repo_full_name.as_deref(),
        Some("Acme/Widgets")
    );
    assert_eq!(run.github_installation_id, Some(INSTALLATION_ID));
}
