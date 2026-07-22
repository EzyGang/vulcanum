use std::future::Future;
use std::time::Duration;

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
                }
            })
            .await?;
        let updated = match disposition {
            DeliveryDisposition::Complete => self.store.complete(&claim).await?,
            DeliveryDisposition::Retry(error) => self.store.retry(&claim, &error).await?,
        };
        if !updated {
            return Err(GithubAppError::DeliveryLeaseLost);
        }

        Ok(true)
    }

    async fn process_pull_request_closed(
        &self,
        delivery: &GithubWebhookDelivery,
    ) -> Result<DeliveryDisposition, GithubAppError> {
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
                tracing::info!(
                    github_delivery_id = delivery.delivery_id,
                    tasks_matched = outcome.matched,
                    tasks_moved = outcome.moved,
                    "processed GitHub pull request webhook",
                );
                Ok(DeliveryDisposition::Complete)
            }
            Ok(_) => Ok(DeliveryDisposition::Retry(
                "no linked task PR found yet".to_owned(),
            )),
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
