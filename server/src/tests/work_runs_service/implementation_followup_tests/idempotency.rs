use std::sync::Arc;

use crate::services::work_runs::service::request_github_implementation::GithubImplementationRequestOutcome;
use crate::test_helpers;
use crate::tests::work_runs_service::implementation_followup_tests::mock_client::MockFollowupTicketClient;
use crate::tests::work_runs_service::implementation_followup_tests::support::{
    request, service, setup_project,
};

#[sqlx::test]
async fn deleting_spawned_run_does_not_reopen_delivery(pool: sqlx::PgPool) {
    setup_project(&pool).await;
    let client = Arc::new(MockFollowupTicketClient::default());
    let work_runs = service(pool.clone(), client.clone()).await;
    let first = work_runs
        .request_github_implementation(request("delivery-deleted-run", "Handle retries."))
        .await
        .expect("spawn follow-up");
    let work_run_id = match first {
        GithubImplementationRequestOutcome::Spawned { work_run_id, .. } => work_run_id,
        outcome => panic!("expected spawned follow-up, got {outcome:?}"),
    };

    work_runs
        .delete_run(work_run_id, test_helpers::DEFAULT_TEAM_ID)
        .await
        .expect("delete spawned run");
    let replay = work_runs
        .request_github_implementation(request("delivery-deleted-run", "Handle retries."))
        .await
        .expect("replay deleted run delivery");

    assert!(matches!(
        replay,
        GithubImplementationRequestOutcome::Spawned {
            work_run_id: replayed_id,
            ..
        } if replayed_id == work_run_id
    ));
    assert_eq!(client.create_count(), 1);
    let run_count = sqlx::query_scalar!("SELECT COUNT(*) FROM work_runs")
        .fetch_one(&pool)
        .await
        .expect("count deleted runs");
    assert_eq!(run_count, Some(0));
}
