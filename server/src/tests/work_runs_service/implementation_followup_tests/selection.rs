use std::sync::Arc;

use crate::models::work_runs::errors::WorkRunsError;
use crate::services::work_runs::service::request_github_implementation::GithubImplementationRequestOutcome;
use crate::test_helpers;
use crate::tests::work_runs_service::implementation_followup_tests::mock_client::MockFollowupTicketClient;
use crate::tests::work_runs_service::implementation_followup_tests::support::{
    map_task, request, service, setup_project,
};

#[sqlx::test]
async fn unauthorized_and_unknown_installation_commands_are_rejected(pool: sqlx::PgPool) {
    setup_project(&pool).await;
    let work_runs = service(pool, Arc::new(MockFollowupTicketClient::default())).await;
    let mut unauthorized = request("delivery-unauthorized", "Handle retries.");
    unauthorized.sender_id = "not-a-member";
    assert!(matches!(
        work_runs
            .request_github_implementation(unauthorized)
            .await
            .expect("reject unauthorized user"),
        GithubImplementationRequestOutcome::Unauthorized { .. }
    ));

    let mut unknown = request("delivery-unknown-install", "Handle retries.");
    unknown.installation_id = 999;
    assert_eq!(
        work_runs
            .request_github_implementation(unknown)
            .await
            .expect("reject unknown installation"),
        GithubImplementationRequestOutcome::UnknownInstallation,
    );
}

#[sqlx::test]
async fn zero_and_multiple_enabled_projects_require_actionable_selection(pool: sqlx::PgPool) {
    let first_project_id = setup_project(&pool).await;
    let provider_id = sqlx::query_scalar!(
        "SELECT provider_id FROM project_configs WHERE id = $1",
        first_project_id,
    )
    .fetch_one(&pool)
    .await
    .expect("project provider")
    .expect("configured provider");
    let second_project_id = test_helpers::insert_project_config_with_provider(
        &pool,
        "second-followup-project",
        provider_id,
    )
    .await;
    sqlx::query!(
        "INSERT INTO project_config_repos (project_config_id, repo_full_name, repo_url, position) VALUES ($1, 'acme/widgets', 'https://github.com/acme/widgets', 0)",
        second_project_id,
    )
    .execute(&pool)
    .await
    .expect("connect second project");
    let work_runs = service(pool.clone(), Arc::new(MockFollowupTicketClient::default())).await;

    let selection = work_runs
        .request_github_implementation(request("delivery-selection", "Handle retries."))
        .await
        .expect("require project selection");
    assert!(matches!(
        &selection,
        GithubImplementationRequestOutcome::ProjectSelectionRequired(options)
            if options.projects.len() == 2
    ));
    let mut invalid = request("delivery-invalid-project", "Handle retries.");
    invalid.project_selector = Some("project:00000000-0000-0000-0000-000000000000");
    assert!(matches!(
        work_runs
            .request_github_implementation(invalid)
            .await
            .expect("reject project selector"),
        GithubImplementationRequestOutcome::InvalidProjectSelection(_)
    ));

    sqlx::query!("UPDATE project_configs SET enabled = false")
        .execute(&pool)
        .await
        .expect("disable projects");
    assert!(matches!(
        work_runs
            .request_github_implementation(request("delivery-disabled", "Handle retries."))
            .await
            .expect("reject disabled projects"),
        GithubImplementationRequestOutcome::NoMatchingProject { .. }
    ));
}

#[sqlx::test]
async fn malformed_command_is_rejected_before_ticket_persistence(pool: sqlx::PgPool) {
    setup_project(&pool).await;
    let work_runs = service(pool.clone(), Arc::new(MockFollowupTicketClient::default())).await;
    let mut malformed = request("delivery-malformed", "placeholder");
    malformed.request_body = None;

    assert!(matches!(
        work_runs
            .request_github_implementation(malformed)
            .await
            .expect("reject malformed command"),
        GithubImplementationRequestOutcome::MalformedCommand { .. }
    ));
    let count = sqlx::query_scalar!("SELECT COUNT(*) FROM github_implementation_followup_requests")
        .fetch_one(&pool)
        .await
        .expect("count persisted requests");
    assert_eq!(count, Some(0));
}

#[sqlx::test]
async fn delivery_replay_rejects_changed_ticket_selector(pool: sqlx::PgPool) {
    let project_id = setup_project(&pool).await;
    map_task(&pool, project_id, "ticket-a").await;
    map_task(&pool, project_id, "ticket-b").await;
    let work_runs = service(pool, Arc::new(MockFollowupTicketClient::default())).await;
    work_runs
        .request_github_implementation(request("delivery-selector-conflict", "Handle retries."))
        .await
        .expect("persist ambiguous delivery");
    let mut conflicting = request("delivery-selector-conflict", "Handle retries.");
    conflicting.ticket_selector = Some("ticket-a");

    assert!(matches!(
        work_runs.request_github_implementation(conflicting).await,
        Err(WorkRunsError::GithubDeliveryConflict)
    ));
}
