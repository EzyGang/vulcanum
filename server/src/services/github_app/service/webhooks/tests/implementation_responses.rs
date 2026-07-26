use uuid::Uuid;

use crate::services::github_app::service::webhooks::implementation_responses::{
    respond_to_implementation_outcome, respond_to_provider_failure,
};
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
        APP_SLUG,
        "delivery-choice",
        123,
        "acme/widgets",
        42,
        Some(request),
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
        "@{APP_SLUG} implement project:{project_id} Handle retries.\n    Also add migration coverage."
    )));
}

#[tokio::test]
async fn rejection_and_provider_outcomes_return_secret_free_actionable_feedback() {
    let writer = RecordingWriter::default();
    let team_id = Uuid::new_v4();
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
            external_task_refs: vec!["task-1".to_owned(), "task-2".to_owned()],
        },
    ];
    for (index, outcome) in outcomes.iter().enumerate() {
        respond_to_implementation_outcome(
            &writer,
            APP_SLUG,
            &format!("delivery-{index}"),
            123,
            "acme/widgets",
            42,
            Some("Handle retries."),
            outcome,
        )
        .await
        .expect("write rejection response");
    }
    respond_to_provider_failure(
        &writer,
        team_id,
        "delivery-provider",
        123,
        "acme/widgets",
        42,
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
}
