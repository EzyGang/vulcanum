mod redis_store;

use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[cfg(test)]
use std::collections::HashMap;
#[cfg(test)]
use std::sync::Arc;

#[cfg(test)]
use tokio::sync::Mutex;

use uuid::Uuid;

use crate::models::github_app::errors::GithubAppError;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum GithubWebhookKind {
    PullRequestClosed,
    ReviewRequested,
}

impl GithubWebhookKind {
    #[must_use]
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::PullRequestClosed => "pull_request_closed",
            Self::ReviewRequested => "review_requested",
        }
    }

    fn from_stored(value: Option<&str>) -> Result<Self, GithubAppError> {
        match value {
            None | Some("") => Ok(Self::PullRequestClosed),
            Some("pull_request_closed") => Ok(Self::PullRequestClosed),
            Some("review_requested") => Ok(Self::ReviewRequested),
            Some(value) => Err(GithubAppError::Redis(format!(
                "unknown GitHub webhook delivery kind: {value}"
            ))),
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct GithubWebhookDelivery {
    pub delivery_id: String,
    pub kind: GithubWebhookKind,
    pub installation_id: i64,
    pub repo_full_name: String,
    pub pr_number: i64,
    pub sender_id: Option<String>,
    pub pr_title: Option<String>,
    pub project_selector: Option<String>,
    pub attempts: i32,
}

#[derive(Debug, Clone)]
pub(crate) struct GithubWebhookClaim {
    pub delivery: GithubWebhookDelivery,
    token: Uuid,
}

#[derive(Clone)]
pub(crate) enum GithubWebhookStore {
    Redis(redis::Client),
    #[cfg(test)]
    Memory(Arc<Mutex<HashMap<String, MemoryDelivery>>>),
}

#[cfg(test)]
#[derive(Clone)]
pub(crate) struct MemoryDelivery {
    delivery: GithubWebhookDelivery,
    next_attempt_at: u64,
    completed: bool,
    claim_token: Option<Uuid>,
}

impl GithubWebhookStore {
    pub fn redis(redis_url: &str) -> Result<Self, GithubAppError> {
        redis::Client::open(redis_url)
            .map(Self::Redis)
            .map_err(|e| GithubAppError::Redis(e.to_string()))
    }

    #[cfg(test)]
    #[must_use]
    pub fn in_memory() -> Self {
        Self::Memory(Arc::new(Mutex::new(HashMap::new())))
    }

    pub async fn enqueue(&self, delivery: GithubWebhookDelivery) -> Result<bool, GithubAppError> {
        match self {
            Self::Redis(client) => redis_store::enqueue(client, &delivery, now_millis()?).await,
            #[cfg(test)]
            Self::Memory(deliveries) => {
                let mut deliveries = deliveries.lock().await;
                if deliveries.contains_key(&delivery.delivery_id) {
                    return Ok(false);
                }
                deliveries.insert(
                    delivery.delivery_id.clone(),
                    MemoryDelivery {
                        delivery,
                        next_attempt_at: now_millis()?,
                        completed: false,
                        claim_token: None,
                    },
                );
                Ok(true)
            }
        }
    }

    pub async fn claim_pending(
        &self,
        lease: Duration,
    ) -> Result<Option<GithubWebhookClaim>, GithubAppError> {
        let now = now_millis()?;
        let token = Uuid::new_v4();
        match self {
            Self::Redis(client) => redis_store::claim_pending(client, now, lease, token).await,
            #[cfg(test)]
            Self::Memory(deliveries) => {
                let mut deliveries = deliveries.lock().await;
                let delivery = deliveries
                    .values_mut()
                    .filter(|entry| !entry.completed && entry.next_attempt_at <= now)
                    .min_by_key(|entry| entry.next_attempt_at);
                match delivery {
                    Some(entry) => {
                        entry.delivery.attempts += 1;
                        entry.next_attempt_at = now.saturating_add(duration_millis(lease));
                        entry.claim_token = Some(token);
                        Ok(Some(GithubWebhookClaim {
                            delivery: entry.delivery.clone(),
                            token,
                        }))
                    }
                    None => Ok(None),
                }
            }
        }
    }

    pub async fn renew(
        &self,
        claim: &GithubWebhookClaim,
        lease: Duration,
    ) -> Result<bool, GithubAppError> {
        let lease_expires_at = now_millis()?.saturating_add(duration_millis(lease));
        match self {
            Self::Redis(client) => redis_store::renew(client, claim, lease_expires_at).await,
            #[cfg(test)]
            Self::Memory(deliveries) => {
                let mut deliveries = deliveries.lock().await;
                let renewed = deliveries
                    .get_mut(&claim.delivery.delivery_id)
                    .filter(|entry| !entry.completed && entry.claim_token == Some(claim.token))
                    .map(|entry| entry.next_attempt_at = lease_expires_at)
                    .is_some();
                Ok(renewed)
            }
        }
    }

    pub async fn complete(&self, claim: &GithubWebhookClaim) -> Result<bool, GithubAppError> {
        match self {
            Self::Redis(client) => redis_store::complete(client, claim).await,
            #[cfg(test)]
            Self::Memory(deliveries) => {
                let mut deliveries = deliveries.lock().await;
                let completed = deliveries
                    .get_mut(&claim.delivery.delivery_id)
                    .filter(|entry| entry.claim_token == Some(claim.token))
                    .map(|entry| {
                        entry.completed = true;
                        entry.claim_token = None;
                    })
                    .is_some();
                Ok(completed)
            }
        }
    }

    pub async fn retry(
        &self,
        claim: &GithubWebhookClaim,
        error: &str,
    ) -> Result<bool, GithubAppError> {
        let attempts = claim.delivery.attempts.clamp(1, 8) as u32;
        let delay = Duration::from_secs(2_u64.pow(attempts));
        let next_attempt = now_millis()?.saturating_add(duration_millis(delay));
        match self {
            Self::Redis(client) => redis_store::retry(client, claim, error, next_attempt).await,
            #[cfg(test)]
            Self::Memory(deliveries) => {
                let mut deliveries = deliveries.lock().await;
                let retried = deliveries
                    .get_mut(&claim.delivery.delivery_id)
                    .filter(|entry| entry.claim_token == Some(claim.token))
                    .map(|entry| {
                        entry.next_attempt_at = next_attempt;
                        entry.claim_token = None;
                    })
                    .is_some();
                Ok(retried)
            }
        }
    }
}

fn now_millis() -> Result<u64, GithubAppError> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis().min(u128::from(u64::MAX)) as u64)
        .map_err(|e| GithubAppError::Redis(format!("system clock error: {e}")))
}

fn duration_millis(duration: Duration) -> u64 {
    duration.as_millis().min(u128::from(u64::MAX)) as u64
}
