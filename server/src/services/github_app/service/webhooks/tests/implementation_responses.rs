use uuid::Uuid;

use crate::services::github_app::service::webhooks::implementation_response_messages::ImplementationResponseContext;
use crate::services::github_app::service::webhooks::implementation_responses::{
    respond_to_implementation_outcome, respond_to_provider_failure,
};
use crate::services::github_app::service::webhooks::responses::GithubResponseTarget;
use crate::services::github_app::service::webhooks::tests::{RecordingWriter, APP_SLUG};
use crate::services::work_runs::service::github_commands::{
    GithubCommandResponseOptions, GithubProjectOption,
};
use crate::services::work_runs::service::request_github_implementation::{
    GithubImplementationRequestOutcome, ImplementationCommandError,
};

#[tokio::test]
async fn project_selection_reply_preserves_multiline_request_in_exact_commands() {
    let writer = RecordingWriter::default();
    let project_id = Uuid::new_v4();
    let request = "Handle retries.\nAlso add migration coverage.";
    let outcome = GithubImplementationRequestOutcome::ProjectSelectionRequired(
        GithubCommandResponseOptions {
            team_id: Uuid::new_v4(),
            projects: vec![GithubProjectOption {
                project_config_id: project_id,
                display_name: "Widgets".to_owned(),
            }],
        },
    );

    respond_to_implementation_outcome(
        &writer,
        ImplementationResponseContext {
            app_slug: APP_SLUG,
            target: GithubResponseTarget {
                delivery_id: "delivery-choice",
                installation_id: 123,
                repo_full_name: "acme/widgets",
                pr_number: 42,
            },
            request_body: Some(request),
            ticket_selector: Some("task-123"),
        },
        &outcome,
    )
    .await
    .expect("write selection response");

    let calls = writer.calls.lock().await;
    assert_eq!(calls.len(), 1);
    assert_eq!(
        calls[0].0,
        "<!-- vulcanum:github-delivery:delivery-choice:implementation -->"
    );
    assert!(calls[0].1.contains(&format!(
        "@{APP_SLUG} implement project:{project_id} ticket:task-123 Handle retries.\n    Also add migration coverage."
    )));
}

#[tokio::test]
async fn rejection_and_provider_outcomes_return_secret_free_actionable_feedback() {
    let writer = RecordingWriter::default();
    let team_id = Uuid::new_v4();
    let project_id = Uuid::new_v4();
    let outcomes = [
        GithubImplementationRequestOutcome::MalformedCommand {
            team_id,
            error: ImplementationCommandError::Malformed,
        },
        GithubImplementationRequestOutcome::Unauthorized { team_id },
        GithubImplementationRequestOutcome::UnknownInstallation,
        GithubImplementationRequestOutcome::AlreadyActive {
            team_id,
            external_task_ref: "task-1".to_owned(),
            ticket_created: false,
        },
        GithubImplementationRequestOutcome::AmbiguousTickets {
            team_id,
            project_config_id: project_id,
            external_task_refs: vec!["task-1".to_owned(), "task-2".to_owned()],
        },
    ];
    for (index, outcome) in outcomes.iter().enumerate() {
        let delivery_id = format!("delivery-{index}");
        respond_to_implementation_outcome(
            &writer,
            ImplementationResponseContext {
                app_slug: APP_SLUG,
                target: GithubResponseTarget {
                    delivery_id: &delivery_id,
                    installation_id: 123,
                    repo_full_name: "acme/widgets",
                    pr_number: 42,
                },
                request_body: Some("Handle retries."),
                ticket_selector: None,
            },
            outcome,
        )
        .await
        .expect("write rejection response");
    }
    respond_to_provider_failure(
        &writer,
        team_id,
        GithubResponseTarget {
            delivery_id: "delivery-provider",
            installation_id: 123,
            repo_full_name: "acme/widgets",
            pr_number: 42,
        },
    )
    .await
    .expect("write provider failure response");

    let calls = writer.calls.lock().await;
    assert_eq!(calls.len(), 6);
    assert!(calls
        .iter()
        .any(|(_, body)| body.contains("request is empty")));
    assert!(calls
        .iter()
        .any(|(_, body)| body.contains("not authorized")));
    assert!(calls.iter().any(|(_, body)| body.contains("Reconnect")));
    assert!(calls
        .iter()
        .any(|(_, body)| body.contains("already has an active")));
    assert!(calls
        .iter()
        .any(|(_, body)| body.contains("will not guess")));
    assert!(calls
        .iter()
        .any(|(_, body)| body.contains("task-tracker connection")));
    assert!(!calls.iter().any(|(_, body)| body.contains("test-key")));
    assert!(calls
        .iter()
        .any(|(_, body)| body.contains("ticket:task-1 Handle retries.")));
}
