use std::sync::Arc;

use crate::services::work_runs::service::request_github_implementation::GithubImplementationRequestOutcome;
use crate::tests::work_runs_service::implementation_followup_tests::mock_client::MockFollowupTicketClient;
use crate::tests::work_runs_service::implementation_followup_tests::support::{
    map_task, request, service, setup_project,
};

#[sqlx::test]
async fn multiple_mapped_tickets_are_rejected_without_guessing(pool: sqlx::PgPool) {
    let project_id = setup_project(&pool).await;
    map_task(&pool, project_id, "ticket-a").await;
    map_task(&pool, project_id, "ticket-b").await;
    let client = Arc::new(MockFollowupTicketClient::default());
    let work_runs = service(pool.clone(), client.clone()).await;

    let outcome = work_runs
        .request_github_implementation(request("delivery-ambiguous", "Handle retries."))
        .await
        .expect("reject ambiguity");

    assert!(matches!(
        &outcome,
        GithubImplementationRequestOutcome::AmbiguousTickets {
            external_task_refs,
            ..
        } if external_task_refs == &["ticket-a".to_owned(), "ticket-b".to_owned()]
    ));
    assert_eq!(client.create_count(), 0);
    assert_eq!(client.update_count(), 0);
    let run_count = sqlx::query_scalar!("SELECT COUNT(*) FROM work_runs")
        .fetch_one(&pool)
        .await
        .expect("count runs");
    assert_eq!(run_count, Some(0));
    let outcome = sqlx::query_scalar!(
        "SELECT outcome FROM github_implementation_followup_requests WHERE delivery_id = 'delivery-ambiguous'",
    )
    .fetch_one(&pool)
    .await
    .expect("persist ambiguity");
    assert_eq!(outcome, "ambiguous_ticket");
}
