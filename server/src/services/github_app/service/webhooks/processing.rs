use std::future::Future;
use std::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};

use tokio::time::MissedTickBehavior;

use crate::models::github_app::errors::GithubAppError;
use crate::services::github_app::service::webhooks::GithubWebhookService;
use crate::services::github_app::webhook_store::{
    GithubWebhookClaim, GithubWebhookDelivery, GithubWebhookKind,
};

#[cfg(not(test))]
const DELIVERY_LEASE: Duration = Duration::from_secs(60);
#[cfg(test)]
const DELIVERY_LEASE: Duration = Duration::from_millis(300);
#[cfg(not(test))]
const DELIVERY_HEARTBEAT_INTERVAL: Duration = Duration::from_secs(10);
#[cfg(test)]
const DELIVERY_HEARTBEAT_INTERVAL: Duration = Duration::from_millis(50);
const MAX_DELIVERIES_PER_TICK: usize = 10;

pub(super) enum DeliveryDisposition {
    Complete,
    Retry(String),
    Terminal(String),
}

impl GithubWebhookService {
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
        let claim = match self.store.claim_pending(DELIVERY_LEASE).await? {
            Some(claim) => claim,
            None => return Ok(false),
        };
        let delivery = &claim.delivery;
        let disposition = self
            .with_claim_heartbeat(&claim, async {
                match delivery.kind {
                    GithubWebhookKind::PullRequestClosed => {
                        self.process_pull_request_closed(delivery).await
                    }
                    GithubWebhookKind::ReviewRequested => {
                        self.process_review_requested(delivery).await
                    }
                    GithubWebhookKind::ImplementationFollowupRequested => {
                        self.process_implementation_requested(delivery).await
                    }
                }
            })
            .await?;
        let disposition_name = match &disposition {
            DeliveryDisposition::Complete => "completed",
            DeliveryDisposition::Retry(_) => "retry_scheduled",
            DeliveryDisposition::Terminal(_) => "terminal_policy_skip",
        };
        let updated = match disposition {
            DeliveryDisposition::Complete => self.store.complete(&claim).await?,
            DeliveryDisposition::Retry(error) => {
                let attempts = delivery.attempts.clamp(1, 8) as u32;
                let delay = Duration::from_secs(2_u64.pow(attempts));
                let next_retry_at_unix_ms = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map(|now| now.saturating_add(delay).as_millis())
                    .unwrap_or_default();
                tracing::warn!(
                    github_delivery_id = delivery.delivery_id,
                    attempts = delivery.attempts,
                    last_error = %error,
                    next_retry_at_unix_ms,
                    "scheduling GitHub webhook delivery retry",
                );
                self.store.retry(&claim, &error).await?
            }
            DeliveryDisposition::Terminal(reason) => self.store.terminal(&claim, &reason).await?,
        };
        if !updated {
            return Err(GithubAppError::DeliveryLeaseLost);
        }
        tracing::info!(
            github_delivery_id = delivery.delivery_id,
            disposition = disposition_name,
            attempts = delivery.attempts,
            "updated GitHub webhook delivery state",
        );

        Ok(true)
    }

    async fn process_pull_request_closed(
        &self,
        delivery: &GithubWebhookDelivery,
    ) -> Result<DeliveryDisposition, GithubAppError> {
        match self
            .work_runs
            .reconcile_pull_request_completion(&delivery.repo_full_name, delivery.pr_number)
            .await
        {
            Ok(outcome) if outcome.matched == 0 => Ok(DeliveryDisposition::Retry(
                "no linked task PR found yet".to_owned(),
            )),
            Ok(outcome) if !outcome.retryable.is_empty() => {
                let error = outcome.retryable.join("; ");
                tracing::warn!(
                    github_delivery_id = delivery.delivery_id,
                    tasks_matched = outcome.matched,
                    tasks_moved = outcome.moved,
                    tasks_already_done = outcome.already_done,
                    retryable_target_count = outcome.retryable.len(),
                    terminal_target_count = outcome.terminal.len(),
                    last_error = %error,
                    "GitHub pull request webhook reconciliation deferred",
                );
                Ok(DeliveryDisposition::Retry(error))
            }
            Ok(outcome) if !outcome.terminal.is_empty() => {
                let reason = outcome.terminal.join("; ");
                tracing::warn!(
                    github_delivery_id = delivery.delivery_id,
                    tasks_matched = outcome.matched,
                    tasks_moved = outcome.moved,
                    tasks_already_done = outcome.already_done,
                    terminal_target_count = outcome.terminal.len(),
                    last_error = %reason,
                    "GitHub pull request webhook reached terminal policy state",
                );
                Ok(DeliveryDisposition::Terminal(reason))
            }
            Ok(outcome) if outcome.moved + outcome.already_done == outcome.matched => {
                tracing::info!(
                    github_delivery_id = delivery.delivery_id,
                    tasks_matched = outcome.matched,
                    tasks_moved = outcome.moved,
                    tasks_already_done = outcome.already_done,
                    "processed GitHub pull request webhook",
                );
                Ok(DeliveryDisposition::Complete)
            }
            Ok(outcome) => Ok(DeliveryDisposition::Retry(format!(
                "{} matched targets were not reconciled",
                outcome.matched
            ))),
            Err(error) => {
                tracing::warn!(
                    github_delivery_id = delivery.delivery_id,
                    %error,
                    "GitHub pull request webhook reconciliation failed; retry scheduled",
                );
                Ok(DeliveryDisposition::Retry(error.to_string()))
            }
        }
    }

    async fn with_claim_heartbeat<F, T>(
        &self,
        claim: &GithubWebhookClaim,
        operation: F,
    ) -> Result<T, GithubAppError>
    where
        F: Future<Output = Result<T, GithubAppError>>,
    {
        let mut heartbeat = tokio::time::interval(DELIVERY_HEARTBEAT_INTERVAL);
        heartbeat.set_missed_tick_behavior(MissedTickBehavior::Delay);
        heartbeat.tick().await;
        tokio::pin!(operation);

        loop {
            tokio::select! {
                biased;
                result = &mut operation => return result,
                _ = heartbeat.tick() => {
                    if !self.store.renew(claim, DELIVERY_LEASE).await? {
                        return Err(GithubAppError::DeliveryLeaseLost);
                    }
                }
            }
        }
    }
}
