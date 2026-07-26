use std::sync::Arc;

use crate::services::work_runs::service::request_github_implementation::GithubImplementationRequestOutcome;
use crate::tests::work_runs_service::implementation_followup_tests::mock_client::{
    integration_task, MockFollowupTicketClient,
};
use crate::tests::work_runs_service::implementation_followup_tests::support::{
    map_task, request, service, setup_project,
};

#[sqlx::test]
async fn mapped_ticket_is_appended_replayed_and_active_run_rejected(pool: sqlx::PgPool) {
    let project_id = setup_project(&pool).await;
    map_task(&pool, project_id, "existing-ticket").await;
    let client = Arc::new(MockFollowupTicketClient::with_task(integration_task(
        "existing-ticket",
        "Existing title",
        Some("Original description".to_owned()),
        "in-review",
    )));
    let work_runs = service(pool.clone(), client.clone()).await;

    let first = work_runs
        .request_github_implementation(request("delivery-reuse", "Handle the retry case."))
        .await
        .expect("spawn follow-up");
    assert!(matches!(
        &first,
        GithubImplementationRequestOutcome::Spawned {
            external_task_ref,
            ticket_created: false,
            ..
        } if external_task_ref == "existing-ticket"
    ));
    let updated = client.task("existing-ticket");
    let description = updated.description.expect("updated description");
    assert_eq!(updated.title, "Existing title");
    assert!(description.starts_with("Original description\n\n"));
    assert!(description.contains("Handle the retry case."));
    assert!(description.contains("vulcanum:github-implementation-followup:delivery-reuse"));
    assert_eq!(client.update_count(), 1);

    let replay = work_runs
        .request_github_implementation(request("delivery-reuse", "Handle the retry case."))
        .await
        .expect("replay follow-up");
    assert!(matches!(
        replay,
        GithubImplementationRequestOutcome::Spawned { .. }
    ));
    assert_eq!(client.update_count(), 1);

    let active = work_runs
        .request_github_implementation(request("delivery-active", "Also add migration coverage."))
        .await
        .expect("reject active follow-up");
    assert!(matches!(
        &active,
        GithubImplementationRequestOutcome::AlreadyActive {
            external_task_ref,
            ..
        } if external_task_ref == "existing-ticket"
    ));
    let description = client
        .task("existing-ticket")
        .description
        .expect("unchanged description");
    assert!(description.contains("Handle the retry case."));
    assert!(!description.contains("Also add migration coverage."));
    assert_eq!(client.update_count(), 1);

    let persisted = sqlx::query!(
        r#"SELECT github_installation_id, repo_full_name, pr_number, project_config_id,
           external_task_ref, request_body, outcome
           FROM github_implementation_followup_requests WHERE delivery_id = 'delivery-reuse'"#,
    )
    .fetch_one(&pool)
    .await
    .expect("persisted request");
    assert_eq!(persisted.github_installation_id, 123);
    assert_eq!(persisted.repo_full_name, "acme/widgets");
    assert_eq!(persisted.pr_number, 42);
    assert_eq!(persisted.project_config_id, project_id);
    assert_eq!(
        persisted.external_task_ref.as_deref(),
        Some("existing-ticket")
    );
    assert_eq!(persisted.request_body, "Handle the retry case.");
    assert_eq!(persisted.outcome, "spawned");
}
