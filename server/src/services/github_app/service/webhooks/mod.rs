mod events;
mod processing;
mod responses;
mod review_requests;
#[cfg(test)]
mod tests;

use std::sync::Arc;
use std::time::Duration;

use hmac::{Hmac, Mac};
use sha2::Sha256;
use thiserror::Error;
use tokio_util::sync::CancellationToken;

use crate::models::github_app::errors::GithubAppError;
use crate::services::github_app::service::pull_requests::PullRequestCommentWriter;
use crate::services::github_app::service::webhooks::events::parse_event;
use crate::services::github_app::webhook_store::GithubWebhookStore;
use crate::services::work_runs::service::WorkRunsService;

const POLL_INTERVAL: Duration = Duration::from_secs(5);

#[derive(Clone)]
pub struct GithubWebhookService {
    secret: Option<Arc<str>>,
    app_slug: Option<Arc<str>>,
    single_user_mode: bool,
    store: GithubWebhookStore,
    work_runs: WorkRunsService,
    comment_writer: Arc<dyn PullRequestCommentWriter>,
}

#[derive(Debug, Error)]
pub enum GithubWebhookError {
    #[error("github webhook secret is not configured")]
    NotConfigured,
    #[error("github app slug is not configured")]
    MissingAppSlug,
    #[error("invalid github webhook signature")]
    InvalidSignature,
    #[error("invalid github webhook payload: {0}")]
    InvalidPayload(#[from] serde_json::Error),
    #[error("github webhook delivery persistence failed: {0}")]
    Persistence(#[from] GithubAppError),
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum GithubWebhookOutcome {
    Ignored,
    Queued { inserted: bool },
}

impl GithubWebhookService {
    #[must_use]
    pub(crate) fn new(
        secret: Option<Arc<str>>,
        app_slug: Option<Arc<str>>,
        single_user_mode: bool,
        store: GithubWebhookStore,
        work_runs: WorkRunsService,
        comment_writer: Arc<dyn PullRequestCommentWriter>,
    ) -> Self {
        Self {
            secret,
            app_slug,
            single_user_mode,
            store,
            work_runs,
            comment_writer,
        }
    }

    pub async fn handle(
        &self,
        signature: &str,
        event: &str,
        delivery: &str,
        body: &[u8],
    ) -> Result<GithubWebhookOutcome, GithubWebhookError> {
        verify_signature(self.secret.as_deref(), signature, body)?;
        let payload = match parse_event(event, delivery, self.app_slug.as_deref(), body)? {
            Some(payload) => payload,
            None => return Ok(GithubWebhookOutcome::Ignored),
        };
        let kind = payload.kind;
        let inserted = self.store.enqueue(payload).await?;

        tracing::info!(
            github_delivery_id = delivery,
            webhook_kind = kind.as_str(),
            duplicate = !inserted,
            "queued GitHub webhook",
        );

        Ok(GithubWebhookOutcome::Queued { inserted })
    }

    pub async fn run(self, cancellation: CancellationToken) {
        let mut interval = tokio::time::interval(POLL_INTERVAL);

        loop {
            tokio::select! {
                () = cancellation.cancelled() => return,
                _ = interval.tick() => {
                    tokio::select! {
                        () = cancellation.cancelled() => return,
                        _ = self.process_batch() => (),
                    }
                }
            }
        }
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
