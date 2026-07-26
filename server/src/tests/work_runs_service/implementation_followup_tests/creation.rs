use std::sync::Arc;

use crate::models::work_runs::errors::WorkRunsError;
use crate::services::work_runs::service::request_github_implementation::GithubImplementationRequestOutcome;
use crate::tests::work_runs_service::implementation_followup_tests::mock_client::MockFollowupTicketClient;
use crate::tests::work_runs_service::implementation_followup_tests::support::{
    request, service, setup_project,
};

#[sqlx::test]
async fn unmapped_pr_creates_one_executable_ticket_and_mapping(pool: sqlx::PgPool) {
    let project_id = setup_project(&pool).await;
    sqlx::query!(
        r#"INSERT INTO github_review_tickets
           (project_config_id, repo_full_name, pr_number, external_task_ref, creation_token)
           VALUES ($1, 'acme/widgets', 42, 'review-ticket', $2)"#,
        project_id,
        uuid::Uuid::new_v4(),
    )
    .execute(&pool)
    .await
    .expect("seed review ticket");
    let client = Arc::new(MockFollowupTicketClient::default());
    let work_runs = service(pool.clone(), client.clone()).await;

    let outcome = work_runs
        .request_github_implementation(request(
            "delivery-create",
            "Add migration coverage exactly as requested.",
        ))
        .await
        .expect("create follow-up ticket");

    assert!(matches!(
        &outcome,
        GithubImplementationRequestOutcome::Spawned {
            external_task_ref,
            ticket_created: true,
            ..
        } if external_task_ref == "created-followup-ticket"
    ));
    assert_eq!(client.create_count(), 1);
    assert_eq!(
        client.block_relations(),
        vec![(
            "created-followup-ticket".to_owned(),
            "review-ticket".to_owned()
        )]
    );
    let task = client.task("created-followup-ticket");
    assert_eq!(task.title, "Follow up PR #42: Improve retries");
    assert_eq!(task.status, "in progress");
    let description = task.description.expect("created description");
    assert!(description.contains("https://github.com/acme/widgets/pull/42"));
    assert!(description.contains("Add migration coverage exactly as requested."));
    assert!(description.contains("vulcanum:github-implementation-ticket"));

    let mapping = sqlx::query!(
        "SELECT external_task_ref, source_work_run_id FROM task_prs WHERE project_config_id = $1",
        project_id,
    )
    .fetch_one(&pool)
    .await
    .expect("created task mapping");
    assert_eq!(mapping.external_task_ref, "created-followup-ticket");
    assert_eq!(mapping.source_work_run_id, None);
    let run_task_slug = sqlx::query_scalar!(
        "SELECT task_slug FROM work_runs WHERE project_config_id = $1",
        project_id,
    )
    .fetch_one(&pool)
    .await
    .expect("created work run");
    assert_eq!(run_task_slug.as_deref(), Some("VLC-2"));
}

#[sqlx::test]
async fn provider_response_loss_recovers_created_ticket_without_duplication(pool: sqlx::PgPool) {
    setup_project(&pool).await;
    let client = Arc::new(MockFollowupTicketClient::default());
    client.fail_once_after_create();
    let work_runs = service(pool, client.clone()).await;

    let first = work_runs
        .request_github_implementation(request("delivery-recovery", "Handle retries."))
        .await;
    assert!(matches!(first, Err(WorkRunsError::Provider(_))));

    let recovered = work_runs
        .request_github_implementation(request("delivery-recovery", "Handle retries."))
        .await
        .expect("recover provider side effect");
    assert!(matches!(
        recovered,
        GithubImplementationRequestOutcome::Spawned {
            ticket_created: true,
            ..
        }
    ));
    assert_eq!(client.create_count(), 1);
}
