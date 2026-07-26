use uuid::Uuid;

use crate::models::github_app::errors::GithubAppError;
use crate::services::github_app::service::pull_requests::PullRequestCommentWriter;
use crate::services::github_app::service::webhooks::implementation_response_messages::{
    malformed_message, project_choices, ticket_action, ticket_selection_message,
    ImplementationResponseContext,
};
use crate::services::github_app::service::webhooks::responses::{
    ensure_response, markdown_escape, GithubResponseTarget,
};
use crate::services::work_runs::service::request_github_implementation::GithubImplementationRequestOutcome;

pub(super) async fn respond_to_implementation_outcome(
    writer: &dyn PullRequestCommentWriter,
    context: ImplementationResponseContext<'_>,
    outcome: &GithubImplementationRequestOutcome,
) -> Result<(), GithubAppError> {
    let (team_id, body) = match outcome {
        GithubImplementationRequestOutcome::Spawned {
            team_id,
            external_task_ref,
            work_run_id,
            ticket_created,
        } => (
            Some(*team_id),
            format!(
                "Vulcanum {} implementation ticket `{}` and queued implementation run `{work_run_id}`.",
                ticket_action(*ticket_created),
                markdown_escape(external_task_ref),
            ),
        ),
        GithubImplementationRequestOutcome::AlreadyActive {
            team_id,
            external_task_ref,
            ticket_created,
        } => (
            Some(*team_id),
            format!(
                "Vulcanum {} implementation ticket `{}`, but that ticket already has an active implementation run. Retry after the current run finishes.",
                ticket_action(*ticket_created),
                markdown_escape(external_task_ref),
            ),
        ),
        GithubImplementationRequestOutcome::AmbiguousTickets {
            team_id,
            project_config_id,
            external_task_refs,
        } => (
            Some(*team_id),
            ticket_selection_message(
                "Vulcanum found multiple implementation tickets mapped to this pull request in the selected project and will not guess. Re-run exactly one command:",
                context,
                *project_config_id,
                external_task_refs,
            ),
        ),
        GithubImplementationRequestOutcome::InvalidTicketSelection {
            team_id,
            project_config_id,
            external_task_refs,
        } => (
            Some(*team_id),
            ticket_selection_message(
                "The ticket selector is not mapped to this pull request in the selected project. Re-run exactly one command:",
                context,
                *project_config_id,
                external_task_refs,
            ),
        ),
        GithubImplementationRequestOutcome::MalformedCommand { team_id, error } => (
            Some(*team_id),
            malformed_message(context.app_slug, *error),
        ),
        GithubImplementationRequestOutcome::Unauthorized { team_id } => (
            Some(*team_id),
            "This GitHub identity is not authorized to start Vulcanum implementation work for this team.".to_owned(),
        ),
        GithubImplementationRequestOutcome::UnknownInstallation => (
            None,
            "Vulcanum could not match this GitHub App installation to a team. Reconnect the installation in Vulcanum settings.".to_owned(),
        ),
        GithubImplementationRequestOutcome::NoMatchingProject { team_id } => (
            Some(*team_id),
            "This repository is not connected to an enabled Vulcanum project.".to_owned(),
        ),
        GithubImplementationRequestOutcome::ProjectSelectionRequired(options) => (
            Some(options.team_id),
            project_choices(
                "Vulcanum found multiple enabled projects for this repository. Re-run exactly one command:",
                context,
                options,
            ),
        ),
        GithubImplementationRequestOutcome::InvalidProjectSelection(options) => (
            Some(options.team_id),
            project_choices(
                "The project selector is invalid or the selected project is disabled. Re-run exactly one command:",
                context,
                options,
            ),
        ),
    };
    ensure_response(writer, team_id, context.target, &body, ":implementation").await
}

pub(super) async fn respond_to_provider_failure(
    writer: &dyn PullRequestCommentWriter,
    team_id: Uuid,
    target: GithubResponseTarget<'_>,
) -> Result<(), GithubAppError> {
    ensure_response(
        writer,
        Some(team_id),
        target,
        "Vulcanum could not create or update the implementation ticket. The request remains retryable; check the task-tracker connection and try again.",
        ":implementation-provider-failure",
    )
    .await
}
