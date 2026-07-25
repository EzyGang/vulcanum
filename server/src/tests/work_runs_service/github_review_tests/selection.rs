use std::sync::Arc;

use crate::services::work_runs::service::request_github_review::{
    GithubReviewRequest, GithubReviewRequestOutcome,
};
use crate::test_helpers;

use super::support::{
    connect_repo, setup_review_project, MockReviewTicketCreator, INSTALLATION_ID, SENDER_ID,
};

#[sqlx::test]
async fn github_review_request_requires_deterministic_project_selection(pool: sqlx::PgPool) {
    let first_id = setup_review_project(&pool).await;
    let second_id = test_helpers::insert_project_config(&pool, "github-review-project-two").await;
    connect_repo(&pool, second_id).await;
    let mut state = test_helpers::build_state(pool.clone()).await;
    state.jobs = state
        .jobs
        .clone()
        .with_review_ticket_creator(Arc::new(MockReviewTicketCreator::default()));

    let ambiguous = state
        .jobs
        .request_github_review(review_request("ambiguous", None))
        .await
        .expect("require project selection");
    match ambiguous {
        GithubReviewRequestOutcome::ProjectSelectionRequired(options) => {
            assert_eq!(options.projects.len(), 2);
            assert_eq!(options.projects[0].project_config_id, first_id);
            assert_eq!(options.projects[1].project_config_id, second_id);
        }
        outcome => panic!("unexpected outcome: {outcome:?}"),
    }

    let selector = format!("project:{second_id}");
    let selected = state
        .jobs
        .request_github_review(review_request("selected", Some(&selector)))
        .await
        .expect("select project");
    assert_eq!(selected, GithubReviewRequestOutcome::Spawned);
    let selected_project = sqlx::query_scalar!(
        "SELECT project_config_id FROM work_runs WHERE github_delivery_id = $1",
        "selected",
    )
    .fetch_one(&pool)
    .await
    .expect("selected review row");
    assert_eq!(selected_project, second_id);
}

#[sqlx::test]
async fn github_review_request_explains_disabled_invalid_and_missing_projects(pool: sqlx::PgPool) {
    let project_id = setup_review_project(&pool).await;
    sqlx::query!(
        "UPDATE project_configs SET review_enabled = false WHERE id = $1",
        project_id,
    )
    .execute(&pool)
    .await
    .expect("disable project review");
    let mut state = test_helpers::build_state(pool).await;
    state.jobs = state
        .jobs
        .clone()
        .with_review_ticket_creator(Arc::new(MockReviewTicketCreator::default()));

    let selector = format!("project:{project_id}");
    assert!(matches!(
        state
            .jobs
            .request_github_review(review_request("disabled", Some(&selector)))
            .await
            .expect("disabled outcome"),
        GithubReviewRequestOutcome::ReviewDisabled(_)
    ));
    assert!(matches!(
        state
            .jobs
            .request_github_review(review_request("invalid", Some("project:not-a-uuid")))
            .await
            .expect("invalid outcome"),
        GithubReviewRequestOutcome::InvalidProjectSelection(_)
    ));
    assert!(matches!(
        state
            .jobs
            .request_github_review(GithubReviewRequest {
                delivery_id: "missing",
                installation_id: INSTALLATION_ID,
                sender_id: SENDER_ID,
                single_user_mode: false,
                repo_full_name: "acme/other",
                pr_number: 42,
                pr_title: "Review me",
                project_selector: None,
            })
            .await
            .expect("missing outcome"),
        GithubReviewRequestOutcome::NoMatchingProject { .. }
    ));
}

fn review_request<'a>(
    delivery_id: &'a str,
    project_selector: Option<&'a str>,
) -> GithubReviewRequest<'a> {
    GithubReviewRequest {
        delivery_id,
        installation_id: INSTALLATION_ID,
        sender_id: SENDER_ID,
        single_user_mode: false,
        repo_full_name: "acme/widgets",
        pr_number: 42,
        pr_title: "Review me",
        project_selector,
    }
}
