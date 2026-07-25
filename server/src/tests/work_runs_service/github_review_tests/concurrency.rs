use std::sync::Arc;
use std::time::Duration;

use crate::models::work_runs::errors::WorkRunsError;
use crate::services::work_runs::service::request_github_review::{
    GithubReviewRequest, GithubReviewRequestOutcome,
};
use crate::test_helpers;

use super::support::{setup_review_project, MockReviewTicketCreator, INSTALLATION_ID, SENDER_ID};

#[sqlx::test]
async fn slow_create_renews_ownership_and_prevents_takeover(pool: sqlx::PgPool) {
    let project_id = setup_review_project(&pool).await;
    let creator = Arc::new(MockReviewTicketCreator::slow());
    let mut state = test_helpers::build_state(pool.clone()).await;
    state.jobs = state
        .jobs
        .clone()
        .with_review_ticket_creator(creator.clone());
    let first_jobs = state.jobs.clone();
    let first = tokio::spawn(async move {
        first_jobs
            .request_github_review(review_request("slow-delivery"))
            .await
    });
    creator.wait_for_create().await;

    sqlx::query!(
        r#"UPDATE github_review_tickets
           SET creation_started_at = NOW() - INTERVAL '10 minutes'
           WHERE project_config_id = $1 AND repo_full_name = $2 AND pr_number = $3"#,
        project_id,
        "acme/widgets",
        42_i64,
    )
    .execute(&pool)
    .await
    .expect("age active reservation");
    tokio::time::sleep(Duration::from_millis(75)).await;

    let competing = state
        .jobs
        .request_github_review(review_request("competing-delivery"))
        .await
        .expect_err("renewed reservation must not be taken over");
    assert!(matches!(
        competing,
        WorkRunsError::ReviewTicketCreationPending
    ));

    creator.release_create();
    let first_outcome = first
        .await
        .expect("slow request task")
        .expect("slow review request");
    assert_eq!(first_outcome, GithubReviewRequestOutcome::Spawned);
    assert_eq!(creator.created_count(), 1);
    assert_eq!(creator.lookup_count(), 0);
}

fn review_request(delivery_id: &str) -> GithubReviewRequest<'_> {
    GithubReviewRequest {
        delivery_id,
        installation_id: INSTALLATION_ID,
        sender_id: SENDER_ID,
        single_user_mode: false,
        repo_full_name: "acme/widgets",
        pr_number: 42,
        pr_title: "Review me",
        project_selector: None,
    }
}
