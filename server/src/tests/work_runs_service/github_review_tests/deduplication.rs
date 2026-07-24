use std::sync::Arc;

use uuid::Uuid;

use crate::models::work_runs::errors::WorkRunsError;

use crate::services::work_runs::service::request_github_review::{
    GithubReviewRequest, GithubReviewRequestOutcome,
};
use crate::test_helpers;

use super::support::{setup_review_project, MockReviewTicketCreator, INSTALLATION_ID, SENDER_ID};

#[sqlx::test]
async fn github_review_request_is_authorized_and_idempotent(pool: sqlx::PgPool) {
    setup_review_project(&pool).await;
    let mut state = test_helpers::build_state(pool.clone()).await;
    let creator = Arc::new(MockReviewTicketCreator::default());
    state.jobs = state
        .jobs
        .clone()
        .with_review_ticket_creator(creator.clone());

    let unauthorized = state
        .jobs
        .request_github_review(review_request("unauthorized", "999", "acme/widgets"))
        .await
        .expect("reject unauthorized sender");
    assert_eq!(unauthorized, GithubReviewRequestOutcome::Unauthorized);

    let first = state
        .jobs
        .request_github_review(review_request("delivery-1", SENDER_ID, "acme/widgets"))
        .await
        .expect("create first review");
    let active_duplicate = state
        .jobs
        .request_github_review(review_request("delivery-2", SENDER_ID, "Acme/Widgets"))
        .await
        .expect("deduplicate active review");
    assert_eq!(first, GithubReviewRequestOutcome::Spawned);
    assert_eq!(active_duplicate, GithubReviewRequestOutcome::AlreadyActive);

    sqlx::query!(
        "UPDATE work_runs SET status = 'completed'::work_run_status WHERE github_delivery_id = $1",
        "delivery-1",
    )
    .execute(&pool)
    .await
    .expect("complete first review");
    let delivery_retry = state
        .jobs
        .request_github_review(review_request("delivery-1", SENDER_ID, "acme/widgets"))
        .await
        .expect("deduplicate delivery retry");
    let new_delivery = state
        .jobs
        .request_github_review(GithubReviewRequest {
            pr_title: "Review latest head",
            ..review_request("delivery-3", SENDER_ID, "acme/widgets")
        })
        .await
        .expect("create review for new delivery");
    assert_eq!(delivery_retry, GithubReviewRequestOutcome::AlreadyActive);
    assert_eq!(new_delivery, GithubReviewRequestOutcome::Spawned);
    assert_eq!(creator.created_count(), 1);
    assert_eq!(creator.lookup_count(), 0);
    let task_refs = sqlx::query_scalar!(
        "SELECT external_task_ref FROM work_runs WHERE review_target_pr_url = $1 ORDER BY created_at",
        "https://github.com/acme/widgets/pull/42",
    )
    .fetch_all(&pool)
    .await
    .expect("review task references");
    assert_eq!(task_refs, vec!["review-ticket-acme/widgets-42"; 2]);
}

#[sqlx::test]
async fn linked_review_identity_authorizes_request_without_team_membership(pool: sqlx::PgPool) {
    setup_review_project(&pool).await;
    let linked_user_id = "789";
    sqlx::query!(
        "UPDATE github_installations SET review_identity_user_id = $1, review_identity_login = $2 WHERE github_installation_id = $3",
        linked_user_id,
        "single-user-owner",
        INSTALLATION_ID,
    )
    .execute(&pool)
    .await
    .expect("link review identity");
    let mut state = test_helpers::build_state(pool).await;
    state.jobs = state
        .jobs
        .clone()
        .with_review_ticket_creator(Arc::new(MockReviewTicketCreator::default()));

    let multiuser_outcome = state
        .jobs
        .request_github_review(review_request(
            "multiuser-delivery",
            linked_user_id,
            "acme/widgets",
        ))
        .await
        .expect("reject linked identity outside single-user mode");
    let single_user_outcome = state
        .jobs
        .request_github_review(GithubReviewRequest {
            single_user_mode: true,
            ..review_request("single-user-delivery", linked_user_id, "acme/widgets")
        })
        .await
        .expect("authorize linked review identity in single-user mode");

    assert_eq!(multiuser_outcome, GithubReviewRequestOutcome::Unauthorized);
    assert_eq!(single_user_outcome, GithubReviewRequestOutcome::Spawned);
}

#[sqlx::test]
async fn stale_reservation_recovers_remote_ticket_without_duplicate(pool: sqlx::PgPool) {
    let project_id = setup_review_project(&pool).await;
    sqlx::query!(
        r#"INSERT INTO github_review_tickets
           (project_config_id, repo_full_name, pr_number, creation_token, creation_started_at)
           VALUES ($1, $2, $3, $4, NOW() - INTERVAL '10 minutes')"#,
        project_id,
        "acme/widgets",
        42_i64,
        Uuid::new_v4(),
    )
    .execute(&pool)
    .await
    .expect("insert stale reservation");
    let creator = Arc::new(MockReviewTicketCreator::with_existing("recovered-ticket"));
    let mut state = test_helpers::build_state(pool.clone()).await;
    state.jobs = state
        .jobs
        .clone()
        .with_review_ticket_creator(creator.clone());

    let outcome = state
        .jobs
        .request_github_review(review_request(
            "recovery-delivery",
            SENDER_ID,
            "acme/widgets",
        ))
        .await
        .expect("recover review ticket");

    assert_eq!(outcome, GithubReviewRequestOutcome::Spawned);
    assert_eq!(creator.lookup_count(), 1);
    assert_eq!(creator.created_count(), 0);
    let task_ref = sqlx::query_scalar!(
        "SELECT external_task_ref FROM github_review_tickets WHERE project_config_id = $1 AND repo_full_name = $2 AND pr_number = $3",
        project_id,
        "acme/widgets",
        42_i64,
    )
    .fetch_one(&pool)
    .await
    .expect("finalized recovered ticket");
    assert_eq!(task_ref.as_deref(), Some("recovered-ticket"));
}

#[sqlx::test]
async fn fresh_reservation_does_not_duplicate_remote_creation(pool: sqlx::PgPool) {
    let project_id = setup_review_project(&pool).await;
    sqlx::query!(
        r#"INSERT INTO github_review_tickets
           (project_config_id, repo_full_name, pr_number, creation_token)
           VALUES ($1, $2, $3, $4)"#,
        project_id,
        "acme/widgets",
        42_i64,
        Uuid::new_v4(),
    )
    .execute(&pool)
    .await
    .expect("insert active reservation");
    let creator = Arc::new(MockReviewTicketCreator::default());
    let mut state = test_helpers::build_state(pool).await;
    state.jobs = state
        .jobs
        .clone()
        .with_review_ticket_creator(creator.clone());

    let error = state
        .jobs
        .request_github_review(review_request(
            "pending-delivery",
            SENDER_ID,
            "acme/widgets",
        ))
        .await
        .expect_err("active reservation must remain exclusive");

    assert!(matches!(error, WorkRunsError::ReviewTicketCreationPending));
    assert_eq!(creator.lookup_count(), 0);
    assert_eq!(creator.created_count(), 0);
}

fn review_request<'a>(
    delivery_id: &'a str,
    sender_id: &'a str,
    repo_full_name: &'a str,
) -> GithubReviewRequest<'a> {
    GithubReviewRequest {
        delivery_id,
        installation_id: INSTALLATION_ID,
        sender_id,
        single_user_mode: false,
        repo_full_name,
        pr_number: 42,
        pr_title: "Review me",
        project_selector: None,
    }
}
