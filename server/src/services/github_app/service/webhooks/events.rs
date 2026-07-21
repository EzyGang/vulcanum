use serde::Deserialize;

use crate::services::github_app::service::webhooks::GithubWebhookError;
use crate::services::github_app::webhook_store::{GithubWebhookDelivery, GithubWebhookKind};

pub(super) fn parse_event(
    event: &str,
    delivery_id: &str,
    app_slug: Option<&str>,
    body: &[u8],
) -> Result<Option<GithubWebhookDelivery>, GithubWebhookError> {
    match event {
        "pull_request" => closed_pull_request(delivery_id, body),
        "issue_comment" => review_request(delivery_id, app_slug, body),
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
        sender_id: None,
        pr_title: None,
        project_selector: None,
        attempts: 0,
    }))
}

fn review_request(
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
        || !contains_mention(&payload.comment.body, app_slug)
    {
        return Ok(None);
    }

    Ok(Some(GithubWebhookDelivery {
        delivery_id: delivery_id.to_owned(),
        kind: GithubWebhookKind::ReviewRequested,
        installation_id: payload.installation.id,
        repo_full_name: payload.repository.full_name,
        pr_number: payload.issue.number,
        sender_id: Some(payload.sender.id.to_string()),
        pr_title: Some(payload.issue.title),
        project_selector: project_selector(&payload.comment.body),
        attempts: 0,
    }))
}

fn contains_mention(body: &str, app_slug: &str) -> bool {
    let body = body.as_bytes();
    let mention = format!("@{app_slug}").to_ascii_lowercase();
    let mention = mention.as_bytes();
    if mention.len() > body.len() {
        return false;
    }

    (0..=body.len() - mention.len()).any(|index| {
        body[index..index + mention.len()].eq_ignore_ascii_case(mention)
            && boundary_before(body, index)
            && boundary_after(body, index + mention.len())
    })
}

fn boundary_before(body: &[u8], index: usize) -> bool {
    index == 0 || !is_login_byte(body[index - 1])
}

fn boundary_after(body: &[u8], index: usize) -> bool {
    index == body.len() || !is_login_byte(body[index])
}

fn is_login_byte(value: u8) -> bool {
    value.is_ascii_alphanumeric() || matches!(value, b'-' | b'_')
}

fn is_app_sender(login: &str, app_slug: &str) -> bool {
    login.eq_ignore_ascii_case(app_slug) || login.eq_ignore_ascii_case(&format!("{app_slug}[bot]"))
}

fn project_selector(body: &str) -> Option<String> {
    let selectors = body
        .split_whitespace()
        .map(|token| {
            token.trim_matches(|character: char| {
                matches!(
                    character,
                    '`' | ',' | ';' | '(' | ')' | '[' | ']' | '{' | '}'
                )
            })
        })
        .filter(|token| {
            token
                .get(..8)
                .is_some_and(|prefix| prefix.eq_ignore_ascii_case("project:"))
        })
        .collect::<Vec<_>>();

    match selectors.as_slice() {
        [] => None,
        [selector] => Some((*selector).to_owned()),
        _ => Some(selectors.join(" ")),
    }
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
    body: String,
}

#[derive(Deserialize)]
struct Sender {
    id: i64,
    login: String,
}
