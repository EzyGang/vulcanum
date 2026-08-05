use std::time::Duration;

use uuid::Uuid;

use crate::models::github_app::errors::GithubAppError;
use crate::services::github_app::webhook_store::{
    duration_millis, GithubWebhookClaim, GithubWebhookCommandError, GithubWebhookDelivery,
    GithubWebhookEnqueueOutcome, GithubWebhookKind,
};

const KEY_PREFIX: &str = "vulcanum:github:webhook:";
type StoredDelivery = (
    Option<String>,
    i64,
    String,
    i64,
    Option<i64>,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
);
type ClaimedDelivery = (String, StoredDelivery, i32);

const PENDING_KEY: &str = "vulcanum:github:webhooks:pending";
const DEDUPE_TTL_SECONDS: u64 = 7 * 24 * 60 * 60;

pub(super) async fn enqueue(
    client: &redis::Client,
    delivery: &GithubWebhookDelivery,
    now: u64,
) -> Result<GithubWebhookEnqueueOutcome, GithubAppError> {
    let mut connection = connection(client).await?;
    let outcome: i64 = redis::Script::new(
        r#"local exists = redis.call('EXISTS', KEYS[1])
           if exists == 1 then
               if redis.call('HGET', KEYS[1], 'terminal') == '1' then
                   return 3
               end
               if redis.call('HGET', KEYS[1], 'completed') == '1' then
                   return 2
               end
               return 0
           end
           redis.call('HSET', KEYS[1],
               'kind', ARGV[1],
               'installation_id', ARGV[2],
               'repo_full_name', ARGV[3],
               'pr_number', ARGV[4],
               'comment_id', ARGV[5],
               'sender_id', ARGV[6],
               'pr_title', ARGV[7],
               'project_selector', ARGV[8],
               'ticket_selector', ARGV[9],
               'request_body', ARGV[10],
               'command_error', ARGV[11],
               'attempts', 0,
               'completed', 0,
               'terminal', 0)
           redis.call('EXPIRE', KEYS[1], ARGV[13])
           redis.call('ZADD', KEYS[2], ARGV[12], ARGV[14])
           return 1"#,
    )
    .key(delivery_key(&delivery.delivery_id))
    .key(PENDING_KEY)
    .arg(delivery.kind.as_str())
    .arg(delivery.installation_id)
    .arg(&delivery.repo_full_name)
    .arg(delivery.pr_number)
    .arg(delivery.comment_id)
    .arg(delivery.sender_id.as_deref().unwrap_or(""))
    .arg(delivery.pr_title.as_deref().unwrap_or(""))
    .arg(delivery.project_selector.as_deref().unwrap_or(""))
    .arg(delivery.ticket_selector.as_deref().unwrap_or(""))
    .arg(delivery.request_body.as_deref().unwrap_or(""))
    .arg(delivery.command_error.map_or("", |error| error.as_str()))
    .arg(now)
    .arg(DEDUPE_TTL_SECONDS)
    .arg(&delivery.delivery_id)
    .invoke_async(&mut connection)
    .await
    .map_err(redis_error)?;

    match outcome {
        0 => Ok(GithubWebhookEnqueueOutcome::DuplicatePending),
        1 => Ok(GithubWebhookEnqueueOutcome::Inserted),
        2 => Ok(GithubWebhookEnqueueOutcome::DuplicateCompleted),
        3 => Ok(GithubWebhookEnqueueOutcome::DuplicateTerminal),
        _ => Err(GithubAppError::Redis(format!(
            "unexpected webhook enqueue outcome: {outcome}"
        ))),
    }
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
           local values = redis.call('HMGET', key, 'kind', 'installation_id', 'repo_full_name', 'pr_number', 'comment_id', 'sender_id', 'pr_title', 'project_selector', 'ticket_selector', 'request_body', 'command_error')
           return {id, values, attempts}"#,
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
            (
                kind,
                installation_id,
                repo_full_name,
                pr_number,
                comment_id,
                sender_id,
                pr_title,
                project_selector,
                ticket_selector,
                request_body,
                command_error,
            ),
            attempts,
        )) => Ok(Some(GithubWebhookClaim {
            delivery: GithubWebhookDelivery {
                delivery_id,
                kind: GithubWebhookKind::from_stored(kind.as_deref())?,
                installation_id,
                repo_full_name,
                pr_number,
                comment_id,
                sender_id: non_empty(sender_id),
                pr_title: non_empty(pr_title),
                project_selector: non_empty(project_selector),
                ticket_selector: non_empty(ticket_selector),
                request_body: non_empty(request_body),
                command_error: GithubWebhookCommandError::from_stored(command_error.as_deref())?,
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
               or redis.call('HGET', KEYS[1], 'completed') == '1'
               or redis.call('HGET', KEYS[1], 'terminal') == '1' then
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
           redis.call('HDEL', KEYS[1], 'claim_token', 'last_error')
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

pub(super) async fn terminal(
    client: &redis::Client,
    claim: &GithubWebhookClaim,
    reason: &str,
) -> Result<bool, GithubAppError> {
    let mut connection = connection(client).await?;
    let terminal: i64 = redis::Script::new(
        r#"if redis.call('HGET', KEYS[1], 'claim_token') ~= ARGV[1] then
               return 0
           end
           redis.call('HSET', KEYS[1], 'terminal', 1, 'last_error', ARGV[2])
           redis.call('HDEL', KEYS[1], 'claim_token')
           redis.call('EXPIRE', KEYS[1], ARGV[3])
           redis.call('ZREM', KEYS[2], ARGV[4])
           return 1"#,
    )
    .key(delivery_key(&claim.delivery.delivery_id))
    .key(PENDING_KEY)
    .arg(claim.token.to_string())
    .arg(reason)
    .arg(DEDUPE_TTL_SECONDS)
    .arg(&claim.delivery.delivery_id)
    .invoke_async(&mut connection)
    .await
    .map_err(redis_error)?;

    Ok(terminal == 1)
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
