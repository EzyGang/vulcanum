use std::time::Duration;

use crate::models::github_app::errors::GithubAppError;
use crate::services::github_app::webhook_store::{duration_millis, GithubWebhookDelivery};

const KEY_PREFIX: &str = "vulcanum:github:webhook:";
const PENDING_KEY: &str = "vulcanum:github:webhooks:pending";
const DEDUPE_TTL_SECONDS: u64 = 7 * 24 * 60 * 60;

pub(super) async fn enqueue(
    client: &redis::Client,
    delivery_id: &str,
    installation_id: i64,
    repo_full_name: &str,
    pr_number: i64,
    now: u64,
) -> Result<bool, GithubAppError> {
    let mut connection = connection(client).await?;
    let inserted: i64 = redis::Script::new(
        r#"if redis.call('EXISTS', KEYS[1]) == 1 then
               return 0
           end
           redis.call('HSET', KEYS[1],
               'installation_id', ARGV[1],
               'repo_full_name', ARGV[2],
               'pr_number', ARGV[3],
               'attempts', 0,
               'completed', 0)
           redis.call('EXPIRE', KEYS[1], ARGV[5])
           redis.call('ZADD', KEYS[2], ARGV[4], ARGV[6])
           return 1"#,
    )
    .key(delivery_key(delivery_id))
    .key(PENDING_KEY)
    .arg(installation_id)
    .arg(repo_full_name)
    .arg(pr_number)
    .arg(now)
    .arg(DEDUPE_TTL_SECONDS)
    .arg(delivery_id)
    .invoke_async(&mut connection)
    .await
    .map_err(redis_error)?;

    Ok(inserted == 1)
}

pub(super) async fn claim_pending(
    client: &redis::Client,
    now: u64,
    lease: Duration,
) -> Result<Option<GithubWebhookDelivery>, GithubAppError> {
    let mut connection = connection(client).await?;
    let claimed: Option<(String, i64, String, i64, i32)> = redis::Script::new(
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
           redis.call('EXPIRE', key, ARGV[4])
           redis.call('ZADD', KEYS[1], ARGV[3], id)
           local values = redis.call('HMGET', key, 'installation_id', 'repo_full_name', 'pr_number')
           return {id, values[1], values[2], values[3], attempts}"#,
    )
    .key(PENDING_KEY)
    .arg(now)
    .arg(KEY_PREFIX)
    .arg(now.saturating_add(duration_millis(lease)))
    .arg(DEDUPE_TTL_SECONDS)
    .invoke_async(&mut connection)
    .await
    .map_err(redis_error)?;

    Ok(claimed.map(
        |(delivery_id, installation_id, repo_full_name, pr_number, attempts)| {
            GithubWebhookDelivery {
                delivery_id,
                installation_id,
                repo_full_name,
                pr_number,
                attempts,
            }
        },
    ))
}

pub(super) async fn complete(
    client: &redis::Client,
    delivery_id: &str,
) -> Result<(), GithubAppError> {
    let mut connection = connection(client).await?;
    redis::Script::new(
        r#"redis.call('HSET', KEYS[1], 'completed', 1)
           redis.call('EXPIRE', KEYS[1], ARGV[1])
           redis.call('ZREM', KEYS[2], ARGV[2])"#,
    )
    .key(delivery_key(delivery_id))
    .key(PENDING_KEY)
    .arg(DEDUPE_TTL_SECONDS)
    .arg(delivery_id)
    .invoke_async::<()>(&mut connection)
    .await
    .map_err(redis_error)
}

pub(super) async fn retry(
    client: &redis::Client,
    delivery: &GithubWebhookDelivery,
    error: &str,
    next_attempt: u64,
) -> Result<(), GithubAppError> {
    let mut connection = connection(client).await?;
    redis::Script::new(
        r#"redis.call('HSET', KEYS[1], 'last_error', ARGV[1])
           redis.call('EXPIRE', KEYS[1], ARGV[2])
           redis.call('ZADD', KEYS[2], ARGV[3], ARGV[4])"#,
    )
    .key(delivery_key(&delivery.delivery_id))
    .key(PENDING_KEY)
    .arg(error)
    .arg(DEDUPE_TTL_SECONDS)
    .arg(next_attempt)
    .arg(&delivery.delivery_id)
    .invoke_async::<()>(&mut connection)
    .await
    .map_err(redis_error)
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
