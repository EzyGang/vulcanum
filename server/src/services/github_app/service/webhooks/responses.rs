use uuid::Uuid;

use crate::models::github_app::errors::GithubAppError;
use crate::services::github_app::service::pull_requests::PullRequestCommentWriter;
use crate::services::work_runs::service::github_commands::GithubProjectOption;
use crate::services::work_runs::service::request_github_review::GithubReviewRequestOutcome;

#[derive(Clone, Copy)]
pub(super) struct GithubResponseTarget<'a> {
    pub delivery_id: &'a str,
    pub installation_id: i64,
    pub repo_full_name: &'a str,
    pub pr_number: i64,
}

pub(super) async fn respond_to_outcome(
    writer: &dyn PullRequestCommentWriter,
    app_slug: &str,
    target: GithubResponseTarget<'_>,
    outcome: &GithubReviewRequestOutcome,
) -> Result<(), GithubAppError> {
    let response = match outcome {
        GithubReviewRequestOutcome::ProjectSelectionRequired(options) => Some((
            options.team_id,
            project_choices(
                "Vulcanum found multiple review-enabled projects for this repository. Re-run exactly one command:",
                app_slug,
                &options.projects,
            ),
        )),
        GithubReviewRequestOutcome::InvalidProjectSelection(options) => Some((
            options.team_id,
            project_choices(
                "The project selector is invalid for this repository. Re-run exactly one command:",
                app_slug,
                &options.projects,
            ),
        )),
        GithubReviewRequestOutcome::ReviewDisabled(options) => Some((
            options.team_id,
            "Vulcanum review automation is disabled for the selected project.".to_owned(),
        )),
        GithubReviewRequestOutcome::NoMatchingProject { team_id } => Some((
            *team_id,
            "This repository is not connected to an enabled Vulcanum project.".to_owned(),
        )),
        GithubReviewRequestOutcome::Spawned
        | GithubReviewRequestOutcome::AlreadyActive
        | GithubReviewRequestOutcome::Unauthorized
        | GithubReviewRequestOutcome::UnknownInstallation => None,
    };

    match response {
        Some((team_id, body)) => ensure_response(writer, Some(team_id), target, &body, "").await,
        None => Ok(()),
    }
}

fn project_choices(heading: &str, app_slug: &str, projects: &[GithubProjectOption]) -> String {
    if projects.is_empty() {
        return format!("{heading}\n\nNo review-enabled project is available.");
    }

    let choices = projects
        .iter()
        .map(|project| {
            format!(
                "- `@{app_slug} review project:{}` — {}",
                project.project_config_id,
                markdown_escape(&project.display_name),
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    format!("{heading}\n\n{choices}")
}

pub(super) async fn ensure_response(
    writer: &dyn PullRequestCommentWriter,
    team_id: Option<Uuid>,
    target: GithubResponseTarget<'_>,
    body: &str,
    marker_suffix: &str,
) -> Result<(), GithubAppError> {
    let marker = format!(
        "<!-- vulcanum:github-delivery:{}{marker_suffix} -->",
        target.delivery_id
    );
    match team_id {
        Some(team_id) => {
            writer
                .ensure_pull_request_comment(
                    team_id,
                    target.installation_id,
                    target.repo_full_name,
                    target.pr_number,
                    &marker,
                    body,
                )
                .await
        }
        None => {
            writer
                .ensure_pull_request_comment_for_installation(
                    target.installation_id,
                    target.repo_full_name,
                    target.pr_number,
                    &marker,
                    body,
                )
                .await
        }
    }
}

pub(super) fn markdown_escape(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for character in value.chars() {
        if matches!(
            character,
            '\\' | '`'
                | '*'
                | '_'
                | '{'
                | '}'
                | '['
                | ']'
                | '<'
                | '>'
                | '('
                | ')'
                | '#'
                | '+'
                | '-'
                | '.'
                | '!'
                | '|'
        ) {
            escaped.push('\\');
        }
        escaped.push(character);
    }
    escaped
}
