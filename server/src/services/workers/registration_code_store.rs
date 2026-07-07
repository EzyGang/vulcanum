use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sha2::Digest;
use uuid::Uuid;

use crate::models::workers::errors::WorkersError;

/// Abstract storage for ephemeral worker registration codes.
#[async_trait]
pub trait CodeStore: Send + Sync {
    /// Save a code with its absolute expiration time.
    ///
    /// Codes are short-lived (minutes) and should auto-expire after `expires_at`.
    async fn save(
        &self,
        code: &str,
        expires_at: DateTime<Utc>,
        team_id: Uuid,
    ) -> Result<(), WorkersError>;

    /// Consume a code atomically, returning its expiration time if it existed.
    ///
    /// The operation must be atomic (get + delete in one step) so that the same
    /// code cannot be used twice.
    async fn consume(&self, code: &str) -> Result<Option<RegistrationCode>, WorkersError>;
}

#[derive(Clone, Copy)]
pub struct RegistrationCode {
    pub expires_at: DateTime<Utc>,
    pub team_id: Uuid,
}

/// Redis-backed implementation.
///
/// Keys: `vulcanum:registration_code:{sha256(code)}`
/// Values: Unix timestamp (seconds)
#[derive(Clone)]
pub struct RedisCodeStore {
    client: redis::Client,
}

impl RedisCodeStore {
    pub fn new(redis_url: &str) -> Result<Self, redis::RedisError> {
        let client = redis::Client::open(redis_url)?;
        Ok(Self { client })
    }
}

#[async_trait]
impl CodeStore for RedisCodeStore {
    async fn save(
        &self,
        code: &str,
        expires_at: DateTime<Utc>,
        team_id: Uuid,
    ) -> Result<(), WorkersError> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let key = code_redis_key(code);
        let ttl_secs = (expires_at - Utc::now()).num_seconds().max(1) as u64;

        redis::cmd("SETEX")
            .arg(&key)
            .arg(ttl_secs)
            .arg(format!("{}:{}", expires_at.timestamp(), team_id))
            .query_async::<()>(&mut conn)
            .await?;

        Ok(())
    }

    async fn consume(&self, code: &str) -> Result<Option<RegistrationCode>, WorkersError> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let key = code_redis_key(code);

        // Atomic GET + DELETE via Lua so the code can never be reused.
        let script = redis::Script::new(
            r#"
            local v = redis.call("GET", KEYS[1])
            if v then
                redis.call("DEL", KEYS[1])
            end
            return v
        "#,
        );

        let value: Option<String> = script.key(&key).invoke_async(&mut conn).await?;

        match value {
            None => Ok(None),
            Some(value) => parse_registration_code(&value),
        }
    }
}

fn code_redis_key(code: &str) -> String {
    format!("vulcanum:registration_code:{}", hash_code(code))
}

fn hash_code(code: &str) -> String {
    let mut hasher = sha2::Sha256::new();
    hasher.update(code.as_bytes());
    hex::encode(hasher.finalize())
}

/// In-memory implementation for tests.
#[allow(dead_code)]
#[derive(Clone)]
pub struct InMemoryCodeStore {
    inner: Arc<tokio::sync::RwLock<std::collections::HashMap<String, RegistrationCode>>>,
}

#[allow(dead_code)]
impl Default for InMemoryCodeStore {
    fn default() -> Self {
        Self::new()
    }
}

impl InMemoryCodeStore {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
        }
    }
}

#[async_trait]
impl CodeStore for InMemoryCodeStore {
    async fn save(
        &self,
        code: &str,
        expires_at: DateTime<Utc>,
        team_id: Uuid,
    ) -> Result<(), WorkersError> {
        let mut guard = self.inner.write().await;
        guard.insert(
            code.to_owned(),
            RegistrationCode {
                expires_at,
                team_id,
            },
        );
        Ok(())
    }

    async fn consume(&self, code: &str) -> Result<Option<RegistrationCode>, WorkersError> {
        let mut guard = self.inner.write().await;
        Ok(guard.remove(code))
    }
}

fn parse_registration_code(value: &str) -> Result<Option<RegistrationCode>, WorkersError> {
    let Some((ts, team_id)) = value.split_once(':') else {
        return Ok(None);
    };
    let ts = match ts.parse::<i64>() {
        Ok(ts) => ts,
        Err(_) => return Ok(None),
    };
    let team_id = match Uuid::parse_str(team_id) {
        Ok(team_id) => team_id,
        Err(_) => return Ok(None),
    };

    Ok(Some(RegistrationCode {
        expires_at: DateTime::from_timestamp(ts, 0).unwrap_or(DateTime::UNIX_EPOCH),
        team_id,
    }))
}
