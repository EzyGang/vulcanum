use uuid::Uuid;

use crate::models::github_app::errors::GithubAppError;
use crate::services::github_app::service::pull_requests::PullRequestCommentWriter;
use crate::services::github_app::service::webhooks::responses::{ensure_response, markdown_escape};
use crate::services::work_runs::service::github_commands::{
    GithubCommandResponseOptions, GithubProjectOption,
};
use crate::services::work_runs::service::request_github_implementation::{
    GithubImplementationRequestOutcome, ImplementationCommandError,
};

#[allow(clippy::too_many_arguments)]
pub(super) async fn respond_to_implementation_outcome(
    writer: &dyn PullRequestCommentWriter,
    app_slug: &str,
    delivery_id: &str,
    installation_id: i64,
    repo_full_name: &str,
    pr_number: i64,
    request_body: Option<&str>,
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
            external_task_refs,
        } => (
            Some(*team_id),
            ambiguous_ticket_message(external_task_refs),
        ),
        GithubImplementationRequestOutcome::MalformedCommand { team_id, error } => (
            Some(*team_id),
            malformed_message(app_slug, *error),
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
                app_slug,
                request_body,
                options,
            ),
        ),
        GithubImplementationRequestOutcome::InvalidProjectSelection(options) => (
            Some(options.team_id),
            project_choices(
                "The project selector is invalid or the selected project is disabled. Re-run exactly one command:",
                app_slug,
                request_body,
                options,
            ),
        ),
    };
    ensure_response(
        writer,
        team_id,
        delivery_id,
        installation_id,
        repo_full_name,
        pr_number,
        &body,
        ":implementation",
    )
    .await
}

#[allow(clippy::too_many_arguments)]
pub(super) async fn respond_to_provider_failure(
    writer: &dyn PullRequestCommentWriter,
    team_id: Uuid,
    delivery_id: &str,
    installation_id: i64,
    repo_full_name: &str,
    pr_number: i64,
) -> Result<(), GithubAppError> {
    ensure_response(
        writer,
        Some(team_id),
        delivery_id,
        installation_id,
        repo_full_name,
        pr_number,
        "Vulcanum could not create or update the implementation ticket. The request remains retryable; check the task-tracker connection and try again.",
        ":implementation-provider-failure",
    )
    .await
}

fn ticket_action(created: bool) -> &'static str {
    match created {
        true => "created",
        false => "reused",
    }
}

fn malformed_message(app_slug: &str, error: ImplementationCommandError) -> String {
    let reason = match error {
        ImplementationCommandError::Malformed => {
            "The implementation command is malformed or its request is empty."
        }
        ImplementationCommandError::Ambiguous => {
            "The comment contains multiple conflicting Vulcanum commands."
        }
    };
    format!(
        "{reason}\n\nUse `@{app_slug} implement [project:<project-config-uuid>] <request>`. The request is required and may span multiple lines."
    )
}

fn ambiguous_ticket_message(external_task_refs: &[String]) -> String {
    let tickets = match external_task_refs {
        [] => String::new(),
        refs => format!(
            "\n\nMapped tickets: {}.",
            refs.iter()
                .map(|task_ref| format!("`{}`", markdown_escape(task_ref)))
                .collect::<Vec<String>>()
                .join(", ")
        ),
    };
    format!(
        "Vulcanum found multiple implementation tickets mapped to this pull request in the selected project and will not guess.{tickets}\n\nRemove the obsolete PR-to-ticket mappings so exactly one ticket remains, then re-run the command. Ticket selectors are not supported."
    )
}

fn project_choices(
    heading: &str,
    app_slug: &str,
    request_body: Option<&str>,
    options: &GithubCommandResponseOptions,
) -> String {
    let request_body = request_body.unwrap_or("<request>");
    if options.projects.is_empty() {
        return format!("{heading}\n\nNo enabled project is available.");
    }
    let choices = options
        .projects
        .iter()
        .map(|project| implementation_choice(app_slug, request_body, project))
        .collect::<Vec<String>>()
        .join("\n\n");
    format!("{heading}\n\n{choices}")
}

fn implementation_choice(
    app_slug: &str,
    request_body: &str,
    project: &GithubProjectOption,
) -> String {
    let command = format!(
        "@{app_slug} implement project:{} {request_body}",
        project.project_config_id,
    )
    .replace('\n', "\n    ");
    format!(
        "{}\n\n    {command}",
        markdown_escape(&project.display_name),
    )
}
