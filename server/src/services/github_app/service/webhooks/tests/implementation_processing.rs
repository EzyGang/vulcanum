use std::sync::Arc;

use crate::models::work_runs::model::WorkRunType;
use crate::services::github_app::service::webhooks::tests::{
    issue_comment_payload, RecordingWriter, APP_SLUG,
};
use crate::services::github_app::service::webhooks::GithubWebhookService;
use crate::services::github_app::webhook_store::GithubWebhookStore;
use crate::test_helpers;
use crate::tests::work_runs_service::implementation_followup_tests::mock_client::MockFollowupTicketClient;
use crate::tests::work_runs_service::implementation_followup_tests::support::setup_project;

#[sqlx::test]
async fn signed_implement_command_creates_auditable_implementation_run(pool: sqlx::PgPool) {
    let project_id = setup_project(&pool).await;
    let state = test_helpers::build_state(pool.clone()).await;
    let writer = Arc::new(RecordingWriter::default());
    let work_runs = state
        .jobs
        .clone()
        .with_implementation_followup_ticket_client(Arc::new(MockFollowupTicketClient::default()));
    let service = GithubWebhookService::new(
        Some(Arc::from(test_helpers::GITHUB_WEBHOOK_SECRET)),
        Some(Arc::from(APP_SLUG)),
        state.is_single_user,
        GithubWebhookStore::in_memory(),
        work_runs,
        writer.clone(),
    );
    let request = "Handle the retry case.\n\nAlso add migration coverage.";
    let payload = issue_comment_payload(
        "created",
        "open",
        Some(serde_json::json!({})),
        &format!("@vulcanum-app implement {request}"),
        "octocat",
    );
    let signature = test_helpers::sign_github_webhook(&payload);

    service
        .handle(
            &signature,
            "issue_comment",
            "implement-smoke-delivery",
            &payload,
        )
        .await
        .expect("queue signed implement command");
    assert!(service
        .process_pending_once()
        .await
        .expect("process signed implement command"));

    let run = sqlx::query!(
        r#"SELECT id, project_config_id, work_type AS "work_type: WorkRunType",
           github_installation_id, github_delivery_id
           FROM work_runs WHERE github_delivery_id = 'implement-smoke-delivery'"#,
    )
    .fetch_one(&pool)
    .await
    .expect("implementation run");
    assert_eq!(run.project_config_id, project_id);
    assert_eq!(run.work_type, WorkRunType::Implementation);
    assert_eq!(run.github_installation_id, Some(123));
    let context = sqlx::query!(
        r#"SELECT repo_full_name, pr_number, request_body, external_task_ref, work_run_id
           FROM github_implementation_followup_requests
           WHERE delivery_id = 'implement-smoke-delivery'"#,
    )
    .fetch_one(&pool)
    .await
    .expect("durable implementation context");
    assert_eq!(context.repo_full_name, "acme/widgets");
    assert_eq!(context.pr_number, 42);
    assert_eq!(context.request_body, request);
    assert_eq!(context.work_run_id, Some(run.id));
    assert_eq!(
        context.external_task_ref.as_deref(),
        Some("created-followup-ticket")
    );
    let calls = writer.calls.lock().await;
    assert_eq!(calls.len(), 1);
    assert!(calls[0]
        .0
        .contains("implement-smoke-delivery:implementation"));
    assert!(calls[0].1.contains("created implementation ticket"));
    assert!(calls[0].1.contains(&run.id.to_string()));
}
