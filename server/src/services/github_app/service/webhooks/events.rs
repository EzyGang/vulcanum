use serde::Deserialize;

use crate::services::github_app::service::webhooks::commands::{comment_command, CommentCommand};
use crate::services::github_app::service::webhooks::GithubWebhookError;
use crate::services::github_app::webhook_store::{
    GithubWebhookCommandError, GithubWebhookDelivery, GithubWebhookKind,
};

pub(super) fn parse_event(
    event: &str,
    delivery_id: &str,
    app_slug: Option<&str>,
    body: &[u8],
) -> Result<Option<GithubWebhookDelivery>, GithubWebhookError> {
    match event {
        "pull_request" => closed_pull_request(delivery_id, body),
        "issue_comment" => issue_comment_command(delivery_id, app_slug, body),
        _ => Ok(None),
    }
}

fn closed_pull_request(
    delivery_id: &str,
    body: &[u8],
) -> Result<Option<GithubWebhookDelivery>, GithubWebhookError> {
    let payload = serde_json::from_slice::<PullRequestEvent>(body)?;
    if payload.action != "closed" {
        return Ok(None);
    }

    Ok(Some(GithubWebhookDelivery {
        delivery_id: delivery_id.to_owned(),
        kind: GithubWebhookKind::PullRequestClosed,
        installation_id: payload.installation.id,
        repo_full_name: payload.repository.full_name,
        pr_number: payload.number,
        comment_id: None,
        sender_id: None,
        pr_title: None,
        project_selector: None,
        ticket_selector: None,
        request_body: None,
        command_error: None,
        attempts: 0,
    }))
}

fn issue_comment_command(
    delivery_id: &str,
    app_slug: Option<&str>,
    body: &[u8],
) -> Result<Option<GithubWebhookDelivery>, GithubWebhookError> {
    let app_slug = app_slug.ok_or(GithubWebhookError::MissingAppSlug)?;
    let payload = serde_json::from_slice::<IssueCommentEvent>(body)?;
    if payload.action != "created"
        || payload.issue.state != "open"
        || payload.issue.pull_request.is_none()
        || is_app_sender(&payload.sender.login, app_slug)
    {
        return Ok(None);
    }

    let command = match comment_command(&payload.comment.body, app_slug) {
        Some(command) => command,
        None => return Ok(None),
    };
    let (kind, project_selector, ticket_selector, request_body, command_error) = match command {
        CommentCommand::Review(project_selector) => (
            GithubWebhookKind::ReviewRequested,
            project_selector,
            None,
            None,
            None,
        ),
        CommentCommand::Implement {
            project_selector,
            ticket_selector,
            request_body,
        } => (
            GithubWebhookKind::ImplementationFollowupRequested,
            project_selector,
            ticket_selector,
            Some(request_body),
            None,
        ),
        CommentCommand::MalformedImplementation => (
            GithubWebhookKind::ImplementationFollowupRequested,
            None,
            None,
            None,
            Some(GithubWebhookCommandError::Malformed),
        ),
        CommentCommand::AmbiguousImplementation => (
            GithubWebhookKind::ImplementationFollowupRequested,
            None,
            None,
            None,
            Some(GithubWebhookCommandError::Ambiguous),
        ),
    };

    Ok(Some(GithubWebhookDelivery {
        delivery_id: delivery_id.to_owned(),
        kind,
        installation_id: payload.installation.id,
        repo_full_name: payload.repository.full_name,
        pr_number: payload.issue.number,
        comment_id: Some(payload.comment.id),
        sender_id: Some(payload.sender.id.to_string()),
        pr_title: Some(payload.issue.title),
        project_selector,
        ticket_selector,
        request_body,
        command_error,
        attempts: 0,
    }))
}

fn is_app_sender(login: &str, app_slug: &str) -> bool {
    login.eq_ignore_ascii_case(app_slug) || login.eq_ignore_ascii_case(&format!("{app_slug}[bot]"))
}

#[derive(Deserialize)]
struct PullRequestEvent {
    action: String,
    number: i64,
    installation: Installation,
    repository: Repository,
}

#[derive(Deserialize)]
struct IssueCommentEvent {
    action: String,
    installation: Installation,
    repository: Repository,
    issue: Issue,
    comment: Comment,
    sender: Sender,
}

#[derive(Deserialize)]
struct Installation {
    id: i64,
}

#[derive(Deserialize)]
struct Repository {
    full_name: String,
}

#[derive(Deserialize)]
struct Issue {
    number: i64,
    title: String,
    state: String,
    pull_request: Option<serde_json::Value>,
}

#[derive(Deserialize)]
struct Comment {
    id: i64,
    body: String,
}

#[derive(Deserialize)]
struct Sender {
    id: i64,
    login: String,
}
