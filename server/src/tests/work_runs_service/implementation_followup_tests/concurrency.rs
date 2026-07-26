use std::sync::Arc;
use std::time::Duration;

use crate::db::work_runs::WorkRunsRepository;
use crate::models::work_runs::errors::WorkRunsError;
use crate::services::work_runs::service::request_github_implementation::GithubImplementationRequestOutcome;
use crate::tests::work_runs_service::implementation_followup_tests::mock_client::{
    integration_task, MockFollowupTicketClient,
};
use crate::tests::work_runs_service::implementation_followup_tests::support::{
    request, service, setup_project,
};

#[sqlx::test]
async fn concurrent_deliveries_create_one_ticket_and_one_active_run(pool: sqlx::PgPool) {
    setup_project(&pool).await;
    let client = Arc::new(MockFollowupTicketClient::default());
    let work_runs = service(pool.clone(), client.clone()).await;

    let (first, second) = tokio::join!(
        work_runs.request_github_implementation(request("delivery-concurrent-a", "Request A")),
        work_runs.request_github_implementation(request("delivery-concurrent-b", "Request B")),
    );
    let first = settle_pending(&work_runs, first, "delivery-concurrent-a", "Request A").await;
    let second = settle_pending(&work_runs, second, "delivery-concurrent-b", "Request B").await;

    let spawned = [first, second]
        .iter()
        .filter(|outcome| matches!(outcome, GithubImplementationRequestOutcome::Spawned { .. }))
        .count();
    assert_eq!(spawned, 1);
    assert_eq!(client.create_count(), 1);
    let run_count = sqlx::query_scalar!("SELECT COUNT(*) FROM work_runs")
        .fetch_one(&pool)
        .await
        .expect("count runs");
    assert_eq!(run_count, Some(1));
    let mapping_count = sqlx::query_scalar!("SELECT COUNT(*) FROM task_prs")
        .fetch_one(&pool)
        .await
        .expect("count task mappings");
    assert_eq!(mapping_count, Some(1));
}

#[sqlx::test]
async fn concurrent_mapping_is_rechecked_before_ticket_creation(pool: sqlx::PgPool) {
    let project_id = setup_project(&pool).await;
    let client = Arc::new(MockFollowupTicketClient::with_task(integration_task(
        "existing-ticket",
        "Existing ticket",
        Some("Original description".to_owned()),
        "review",
    )));
    let work_runs = service(pool.clone(), client.clone()).await;
    let repo = WorkRunsRepository::new();
    let mut mapping = pool.begin().await.expect("begin mapping transaction");
    repo.lock_task_pr_target(&mut mapping, project_id, "acme/widgets", 42)
        .await
        .expect("lock PR mapping");

    let concurrent_service = work_runs.clone();
    let followup = tokio::spawn(async move {
        concurrent_service
            .request_github_implementation(request("delivery-mapping-race", "Handle mapping race."))
            .await
    });
    tokio::time::sleep(Duration::from_millis(25)).await;
    assert!(!followup.is_finished());
    repo.upsert_github_followup_task_pr(
        &mut *mapping,
        project_id,
        "existing-ticket",
        "https://github.com/acme/widgets/pull/42",
        "acme/widgets",
        42,
    )
    .await
    .expect("persist original mapping");
    mapping.commit().await.expect("commit original mapping");

    let outcome = tokio::time::timeout(Duration::from_secs(5), followup)
        .await
        .expect("follow-up completes after mapping commit")
        .expect("follow-up task joins")
        .expect("follow-up succeeds");
    assert!(matches!(
        outcome,
        GithubImplementationRequestOutcome::Spawned {
            external_task_ref,
            ticket_created: false,
            ..
        } if external_task_ref == "existing-ticket"
    ));
    assert_eq!(client.create_count(), 0);
}

async fn settle_pending(
    work_runs: &crate::services::work_runs::service::WorkRunsService,
    outcome: Result<GithubImplementationRequestOutcome, WorkRunsError>,
    delivery_id: &str,
    body: &str,
) -> GithubImplementationRequestOutcome {
    match outcome {
        Ok(outcome) => outcome,
        Err(WorkRunsError::ImplementationFollowupPending) => work_runs
            .request_github_implementation(request(delivery_id, body))
            .await
            .expect("retry reserved follow-up"),
        Err(error) => panic!("unexpected concurrent follow-up error: {error}"),
    }
}
