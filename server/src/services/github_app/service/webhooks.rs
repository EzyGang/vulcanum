use std::sync::Arc;

use hmac::{Hmac, Mac};
use serde::Deserialize;
use sha2::Sha256;
use thiserror::Error;

use crate::models::work_runs::errors::WorkRunsError;
use crate::services::work_runs::service::WorkRunsService;

#[derive(Clone)]
pub struct GithubWebhookService {
    secret: Option<Arc<str>>,
    work_runs: WorkRunsService,
}

#[derive(Debug, Error)]
pub enum GithubWebhookError {
    #[error("github webhook secret is not configured")]
    NotConfigured,
    #[error("invalid github webhook signature")]
    InvalidSignature,
    #[error("invalid github webhook payload: {0}")]
    InvalidPayload(#[from] serde_json::Error),
    #[error("pull request reconciliation failed: {0}")]
    Reconciliation(#[from] WorkRunsError),
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum GithubWebhookOutcome {
    Ignored,
    Reconciled { moved: usize },
}

impl GithubWebhookService {
    #[must_use]
    pub fn new(secret: Option<&str>, work_runs: WorkRunsService) -> Self {
        Self {
            secret: secret.map(Arc::from),
            work_runs,
        }
    }

    pub async fn handle(
        &self,
        signature: &str,
        event: &str,
        delivery: Option<&str>,
        body: &[u8],
    ) -> Result<GithubWebhookOutcome, GithubWebhookError> {
        verify_signature(self.secret.as_deref(), signature, body)?;
        let payload = match closed_pull_request(event, body)? {
            Some(payload) => payload,
            None => return Ok(GithubWebhookOutcome::Ignored),
        };

        let moved = self
            .work_runs
            .reconcile_pull_request_completion(
                payload.installation.id,
                &payload.repository.full_name,
                payload.number,
            )
            .await?;

        tracing::info!(
            github_delivery_id = delivery,
            installation_id = payload.installation.id,
            repo_full_name = %payload.repository.full_name,
            pr_number = payload.number,
            tasks_moved = moved,
            "processed GitHub pull request webhook",
        );

        Ok(GithubWebhookOutcome::Reconciled { moved })
    }
}

pub(super) fn verify_signature(
    secret: Option<&str>,
    signature: &str,
    body: &[u8],
) -> Result<(), GithubWebhookError> {
    let secret = secret.ok_or(GithubWebhookError::NotConfigured)?;
    let encoded = signature
        .strip_prefix("sha256=")
        .ok_or(GithubWebhookError::InvalidSignature)?;
    let expected = hex::decode(encoded).map_err(|_| GithubWebhookError::InvalidSignature)?;
    let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes())
        .map_err(|_| GithubWebhookError::InvalidSignature)?;
    mac.update(body);
    mac.verify_slice(&expected)
        .map_err(|_| GithubWebhookError::InvalidSignature)
}

fn closed_pull_request(
    event: &str,
    body: &[u8],
) -> Result<Option<PullRequestEvent>, GithubWebhookError> {
    if event != "pull_request" {
        return Ok(None);
    }

    let payload = serde_json::from_slice::<PullRequestEvent>(body)?;
    match payload.action.as_str() {
        "closed" => Ok(Some(payload)),
        _ => Ok(None),
    }
}

#[cfg(test)]
pub(super) fn is_completion_event(event: &str, body: &[u8]) -> Result<bool, GithubWebhookError> {
    closed_pull_request(event, body).map(|payload| payload.is_some())
}

#[derive(Deserialize)]
struct PullRequestEvent {
    action: String,
    number: i64,
    installation: Installation,
    repository: Repository,
}

#[derive(Deserialize)]
struct Installation {
    id: i64,
}

#[derive(Deserialize)]
struct Repository {
    full_name: String,
}
