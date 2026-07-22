use std::time::Duration;

use uuid::Uuid;

use crate::models::github_app::errors::GithubAppError;
use crate::services::github_app::webhook_store::{
    duration_millis, GithubWebhookClaim, GithubWebhookDelivery, GithubWebhookKind,
};

const KEY_PREFIX: &str = "vulcanum:github:webhook:";
type ClaimedDelivery = (
    String,
    Option<String>,
    i64,
    String,
    i64,
    Option<String>,
    Option<String>,
    Option<String>,
    i32,
);

const PENDING_KEY: &str = "vulcanum:github:webhooks:pending";
const DEDUPE_TTL_SECONDS: u64 = 7 * 24 * 60 * 60;

pub(super) async fn enqueue(
    client: &redis::Client,
    delivery: &GithubWebhookDelivery,
    now: u64,
) -> Result<bool, GithubAppError> {
    let mut connection = connection(client).await?;
    let inserted: i64 = redis::Script::new(
        r#"if redis.call('EXISTS', KEYS[1]) == 1 then
               return 0
           end
           redis.call('HSET', KEYS[1],
               'kind', ARGV[1],
               'installation_id', ARGV[2],
               'repo_full_name', ARGV[3],
               'pr_number', ARGV[4],
               'sender_id', ARGV[5],
               'pr_title', ARGV[6],
               'project_selector', ARGV[7],
               'attempts', 0,
               'completed', 0)
           redis.call('EXPIRE', KEYS[1], ARGV[9])
           redis.call('ZADD', KEYS[2], ARGV[8], ARGV[10])
           return 1"#,
    )
    .key(delivery_key(&delivery.delivery_id))
    .key(PENDING_KEY)
    .arg(delivery.kind.as_str())
    .arg(delivery.installation_id)
    .arg(&delivery.repo_full_name)
    .arg(delivery.pr_number)
    .arg(delivery.sender_id.as_deref().unwrap_or(""))
    .arg(delivery.pr_title.as_deref().unwrap_or(""))
    .arg(delivery.project_selector.as_deref().unwrap_or(""))
    .arg(now)
    .arg(DEDUPE_TTL_SECONDS)
    .arg(&delivery.delivery_id)
    .invoke_async(&mut connection)
    .await
    .map_err(redis_error)?;

    Ok(inserted == 1)
}

pub(super) async fn claim_pending(
    client: &redis::Client,
    now: u64,
    lease: Duration,
    token: Uuid,
) -> Result<Option<GithubWebhookClaim>, GithubAppError> {
    let mut connection = connection(client).await?;
    let claimed: Option<ClaimedDelivery> = redis::Script::new(
        r#"local ids = redis.call('ZRANGEBYSCORE', KEYS[1], '-inf', ARGV[1], 'LIMIT', 0, 1)
           if #ids == 0 then
               return nil
           end
           local id = ids[1]
           local key = ARGV[2] .. id
           if redis.call('EXISTS', key) == 0 then
               redis.call('ZREM', KEYS[1], id)
               return nil
           end
           local attempts = redis.call('HINCRBY', key, 'attempts', 1)
           redis.call('HSET', key, 'claim_token', ARGV[5])
           redis.call('EXPIRE', key, ARGV[4])
           redis.call('ZADD', KEYS[1], ARGV[3], id)
           local values = redis.call('HMGET', key, 'kind', 'installation_id', 'repo_full_name', 'pr_number', 'sender_id', 'pr_title', 'project_selector')
           return {id, values[1], values[2], values[3], values[4], values[5], values[6], values[7], attempts}"#,
    )
    .key(PENDING_KEY)
    .arg(now)
    .arg(KEY_PREFIX)
    .arg(now.saturating_add(duration_millis(lease)))
    .arg(DEDUPE_TTL_SECONDS)
    .arg(token.to_string())
    .invoke_async(&mut connection)
    .await
    .map_err(redis_error)?;

    match claimed {
        Some((
            delivery_id,
            kind,
            installation_id,
            repo_full_name,
            pr_number,
            sender_id,
            pr_title,
            project_selector,
            attempts,
        )) => Ok(Some(GithubWebhookClaim {
            delivery: GithubWebhookDelivery {
                delivery_id,
                kind: GithubWebhookKind::from_stored(kind.as_deref())?,
                installation_id,
                repo_full_name,
                pr_number,
                sender_id: non_empty(sender_id),
                pr_title: non_empty(pr_title),
                project_selector: non_empty(project_selector),
                attempts,
            },
            token,
        })),
        None => Ok(None),
    }
}

pub(super) async fn renew(
    client: &redis::Client,
    claim: &GithubWebhookClaim,
    lease_expires_at: u64,
) -> Result<bool, GithubAppError> {
    let mut connection = connection(client).await?;
    let renewed: i64 = redis::Script::new(
        r#"if redis.call('HGET', KEYS[1], 'claim_token') ~= ARGV[1]
               or redis.call('HGET', KEYS[1], 'completed') == '1' then
               return 0
           end
           redis.call('EXPIRE', KEYS[1], ARGV[2])
           redis.call('ZADD', KEYS[2], ARGV[3], ARGV[4])
           return 1"#,
    )
    .key(delivery_key(&claim.delivery.delivery_id))
    .key(PENDING_KEY)
    .arg(claim.token.to_string())
    .arg(DEDUPE_TTL_SECONDS)
    .arg(lease_expires_at)
    .arg(&claim.delivery.delivery_id)
    .invoke_async(&mut connection)
    .await
    .map_err(redis_error)?;

    Ok(renewed == 1)
}

pub(super) async fn complete(
    client: &redis::Client,
    claim: &GithubWebhookClaim,
) -> Result<bool, GithubAppError> {
    let mut connection = connection(client).await?;
    let completed: i64 = redis::Script::new(
        r#"if redis.call('HGET', KEYS[1], 'claim_token') ~= ARGV[1] then
               return 0
           end
           redis.call('HSET', KEYS[1], 'completed', 1)
           redis.call('HDEL', KEYS[1], 'claim_token')
           redis.call('EXPIRE', KEYS[1], ARGV[2])
           redis.call('ZREM', KEYS[2], ARGV[3])
           return 1"#,
    )
    .key(delivery_key(&claim.delivery.delivery_id))
    .key(PENDING_KEY)
    .arg(claim.token.to_string())
    .arg(DEDUPE_TTL_SECONDS)
    .arg(&claim.delivery.delivery_id)
    .invoke_async(&mut connection)
    .await
    .map_err(redis_error)?;

    Ok(completed == 1)
}

pub(super) async fn retry(
    client: &redis::Client,
    claim: &GithubWebhookClaim,
    error: &str,
    next_attempt: u64,
) -> Result<bool, GithubAppError> {
    let mut connection = connection(client).await?;
    let retried: i64 = redis::Script::new(
        r#"if redis.call('HGET', KEYS[1], 'claim_token') ~= ARGV[1] then
               return 0
           end
           redis.call('HSET', KEYS[1], 'last_error', ARGV[2])
           redis.call('HDEL', KEYS[1], 'claim_token')
           redis.call('EXPIRE', KEYS[1], ARGV[3])
           redis.call('ZADD', KEYS[2], ARGV[4], ARGV[5])
           return 1"#,
    )
    .key(delivery_key(&claim.delivery.delivery_id))
    .key(PENDING_KEY)
    .arg(claim.token.to_string())
    .arg(error)
    .arg(DEDUPE_TTL_SECONDS)
    .arg(next_attempt)
    .arg(&claim.delivery.delivery_id)
    .invoke_async(&mut connection)
    .await
    .map_err(redis_error)?;

    Ok(retried == 1)
}

fn non_empty(value: Option<String>) -> Option<String> {
    value.filter(|value| !value.is_empty())
}

async fn connection(
    client: &redis::Client,
) -> Result<redis::aio::MultiplexedConnection, GithubAppError> {
    client
        .get_multiplexed_async_connection()
        .await
        .map_err(redis_error)
}

fn delivery_key(delivery_id: &str) -> String {
    format!("{KEY_PREFIX}{delivery_id}")
}

fn redis_error(error: redis::RedisError) -> GithubAppError {
    GithubAppError::Redis(error.to_string())
}
