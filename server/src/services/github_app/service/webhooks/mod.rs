mod events;
mod responses;
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
use crate::services::github_app::service::webhooks::responses::respond_to_outcome;
use crate::services::github_app::webhook_store::{
    GithubWebhookDelivery, GithubWebhookKind, GithubWebhookStore,
};
use crate::services::work_runs::service::request_github_review::GithubReviewRequest;
use crate::services::work_runs::service::WorkRunsService;

const POLL_INTERVAL: Duration = Duration::from_secs(5);
const DELIVERY_LEASE: Duration = Duration::from_secs(60);
const MAX_DELIVERIES_PER_TICK: usize = 10;

#[derive(Clone)]
pub struct GithubWebhookService {
    secret: Option<Arc<str>>,
    app_slug: Option<Arc<str>>,
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
        store: GithubWebhookStore,
        work_runs: WorkRunsService,
        comment_writer: Arc<dyn PullRequestCommentWriter>,
    ) -> Self {
        Self {
            secret,
            app_slug,
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

    pub(crate) async fn process_batch(&self) -> usize {
        let mut processed = 0;

        for _ in 0..MAX_DELIVERIES_PER_TICK {
            match self.process_pending_once().await {
                Ok(true) => processed += 1,
                Ok(false) => break,
                Err(error) => {
                    tracing::error!(%error, "GitHub webhook delivery worker failed");
                    break;
                }
            }
        }

        processed
    }

    pub(crate) async fn process_pending_once(&self) -> Result<bool, GithubAppError> {
        let delivery = match self.store.claim_pending(DELIVERY_LEASE).await? {
            Some(delivery) => delivery,
            None => return Ok(false),
        };

        match delivery.kind {
            GithubWebhookKind::PullRequestClosed => {
                self.process_pull_request_closed(&delivery).await?
            }
            GithubWebhookKind::ReviewRequested => self.process_review_requested(&delivery).await?,
        }
        Ok(true)
    }

    async fn process_pull_request_closed(
        &self,
        delivery: &GithubWebhookDelivery,
    ) -> Result<(), GithubAppError> {
        match self
            .work_runs
            .reconcile_pull_request_completion(
                delivery.installation_id,
                &delivery.repo_full_name,
                delivery.pr_number,
            )
            .await
        {
            Ok(outcome) if outcome.matched > 0 => {
                self.store.complete(&delivery.delivery_id).await?;
                tracing::info!(
                    github_delivery_id = delivery.delivery_id,
                    tasks_matched = outcome.matched,
                    tasks_moved = outcome.moved,
                    "processed GitHub pull request webhook",
                );
            }
            Ok(_) => {
                self.store
                    .retry(delivery, "no linked task PR found yet")
                    .await?;
            }
            Err(error) => {
                tracing::warn!(
                    github_delivery_id = delivery.delivery_id,
                    %error,
                    "GitHub pull request webhook reconciliation failed; retry scheduled",
                );
                self.store.retry(delivery, &error.to_string()).await?;
            }
        }
        Ok(())
    }

    async fn process_review_requested(
        &self,
        delivery: &GithubWebhookDelivery,
    ) -> Result<(), GithubAppError> {
        let sender_id = required(&delivery.sender_id, "sender_id")?;
        let pr_title = required(&delivery.pr_title, "pr_title")?;
        let outcome = match self
            .work_runs
            .request_github_review(GithubReviewRequest {
                delivery_id: &delivery.delivery_id,
                installation_id: delivery.installation_id,
                sender_id,
                repo_full_name: &delivery.repo_full_name,
                pr_number: delivery.pr_number,
                pr_title,
                project_selector: delivery.project_selector.as_deref(),
            })
            .await
        {
            Ok(outcome) => outcome,
            Err(error) => {
                self.store.retry(delivery, &error.to_string()).await?;
                return Ok(());
            }
        };
        let app_slug = self
            .app_slug
            .as_deref()
            .ok_or(GithubAppError::NotConfigured)?;
        match respond_to_outcome(
            self.comment_writer.as_ref(),
            app_slug,
            &delivery.delivery_id,
            delivery.installation_id,
            &delivery.repo_full_name,
            delivery.pr_number,
            &outcome,
        )
        .await
        {
            Ok(()) => self.store.complete(&delivery.delivery_id).await?,
            Err(error) => self.store.retry(delivery, &error.to_string()).await?,
        }
        Ok(())
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

fn required<'a>(value: &'a Option<String>, field: &str) -> Result<&'a str, GithubAppError> {
    value
        .as_deref()
        .ok_or_else(|| GithubAppError::Redis(format!("review webhook omitted {field}")))
}
