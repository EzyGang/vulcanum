use std::sync::Arc;
use std::time::Duration;

use hmac::{Hmac, Mac};
use serde::Deserialize;
use sha2::Sha256;
use thiserror::Error;
use tokio_util::sync::CancellationToken;

use crate::models::github_app::errors::GithubAppError;
use crate::services::github_app::webhook_store::GithubWebhookStore;
use crate::services::work_runs::service::WorkRunsService;

const POLL_INTERVAL: Duration = Duration::from_secs(5);
const DELIVERY_LEASE: Duration = Duration::from_secs(60);
const MAX_DELIVERIES_PER_TICK: usize = 10;

#[derive(Clone)]
pub struct GithubWebhookService {
    secret: Option<Arc<str>>,
    store: GithubWebhookStore,
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
        store: GithubWebhookStore,
        work_runs: WorkRunsService,
    ) -> Self {
        Self {
            secret,
            store,
            work_runs,
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
        let payload = match closed_pull_request(event, body)? {
            Some(payload) => payload,
            None => return Ok(GithubWebhookOutcome::Ignored),
        };
        let inserted = self
            .store
            .enqueue(
                delivery,
                payload.installation.id,
                &payload.repository.full_name,
                payload.number,
            )
            .await?;

        tracing::info!(
            github_delivery_id = delivery,
            duplicate = !inserted,
            "queued GitHub pull request webhook",
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
                Err(e) => {
                    tracing::error!(error = %e, "GitHub webhook delivery worker failed");
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
                    .retry(&delivery, "no linked task PR found yet")
                    .await?;
            }
            Err(e) => {
                tracing::warn!(
                    github_delivery_id = delivery.delivery_id,
                    error = %e,
                    "GitHub pull request webhook reconciliation failed; retry scheduled",
                );
                self.store.retry(&delivery, &e.to_string()).await?;
            }
        }

        Ok(true)
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
