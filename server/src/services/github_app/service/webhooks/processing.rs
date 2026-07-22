use std::time::Duration;

use crate::models::github_app::errors::GithubAppError;
use crate::services::github_app::service::webhooks::GithubWebhookService;
use crate::services::github_app::webhook_store::{GithubWebhookDelivery, GithubWebhookKind};

const DELIVERY_LEASE: Duration = Duration::from_secs(60);
const MAX_DELIVERIES_PER_TICK: usize = 10;

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
}
