use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::Digest;
use uuid::Uuid;

use crate::services::teams::errors::TeamsError;

const INVITE_KEY_PREFIX: &str = "vulcanum:team_invite";

#[async_trait]
pub trait TeamInviteStore: Send + Sync {
    async fn save(&self, token: &str, payload: &TeamInvitePayload) -> Result<(), TeamsError>;

    async fn get(&self, token: &str) -> Result<Option<TeamInvitePayload>, TeamsError>;

    async fn consume(&self, token: &str) -> Result<Option<TeamInvitePayload>, TeamsError>;
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TeamInvitePayload {
    pub team_id: Uuid,
    pub created_by_user_id: String,
    pub role: String,
    pub expires_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct RedisTeamInviteStore {
    client: redis::Client,
}

impl RedisTeamInviteStore {
    pub fn new(redis_url: &str) -> Result<Self, redis::RedisError> {
        let client = redis::Client::open(redis_url)?;
        Ok(Self { client })
    }
}

#[async_trait]
impl TeamInviteStore for RedisTeamInviteStore {
    async fn save(&self, token: &str, payload: &TeamInvitePayload) -> Result<(), TeamsError> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let key = invite_redis_key(token);
        let value = serde_json::to_string(payload)
            .map_err(|e| TeamsError::InviteStore(format!("serialize invite payload: {e}")))?;
        let ttl_secs = (payload.expires_at - Utc::now()).num_seconds().max(1) as u64;

        redis::cmd("SETEX")
            .arg(&key)
            .arg(ttl_secs)
            .arg(value)
            .query_async::<()>(&mut conn)
            .await?;

        Ok(())
    }

    async fn get(&self, token: &str) -> Result<Option<TeamInvitePayload>, TeamsError> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let key = invite_redis_key(token);
        let value: Option<String> = redis::cmd("GET").arg(&key).query_async(&mut conn).await?;

        parse_invite_payload(value)
    }

    async fn consume(&self, token: &str) -> Result<Option<TeamInvitePayload>, TeamsError> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let key = invite_redis_key(token);
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
        parse_invite_payload(value)
    }
}

#[derive(Clone, Default)]
pub struct InMemoryTeamInviteStore {
    inner: Arc<tokio::sync::RwLock<HashMap<String, TeamInvitePayload>>>,
}

impl InMemoryTeamInviteStore {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[cfg(test)]
    pub async fn contains_raw_key(&self, token: &str) -> bool {
        self.inner.read().await.contains_key(token)
    }
}

#[async_trait]
impl TeamInviteStore for InMemoryTeamInviteStore {
    async fn save(&self, token: &str, payload: &TeamInvitePayload) -> Result<(), TeamsError> {
        self.inner
            .write()
            .await
            .insert(invite_redis_key(token), payload.clone());
        Ok(())
    }

    async fn get(&self, token: &str) -> Result<Option<TeamInvitePayload>, TeamsError> {
        let payload = self
            .inner
            .read()
            .await
            .get(&invite_redis_key(token))
            .cloned();
        Ok(valid_payload(payload))
    }

    async fn consume(&self, token: &str) -> Result<Option<TeamInvitePayload>, TeamsError> {
        let payload = self.inner.write().await.remove(&invite_redis_key(token));
        Ok(valid_payload(payload))
    }
}

#[must_use]
pub(crate) fn invite_redis_key(token: &str) -> String {
    format!("{INVITE_KEY_PREFIX}:{}", hash_token(token))
}

#[must_use]
pub(crate) fn hash_token(token: &str) -> String {
    let mut hasher = sha2::Sha256::new();
    hasher.update(token.as_bytes());
    hex::encode(hasher.finalize())
}

fn parse_invite_payload(value: Option<String>) -> Result<Option<TeamInvitePayload>, TeamsError> {
    match value {
        Some(value) => serde_json::from_str::<TeamInvitePayload>(&value)
            .map(|payload| valid_payload(Some(payload)))
            .map_err(|_| TeamsError::InviteStore("invalid invite payload".to_owned())),
        None => Ok(None),
    }
}

fn valid_payload(payload: Option<TeamInvitePayload>) -> Option<TeamInvitePayload> {
    match payload {
        Some(payload) if payload.expires_at > Utc::now() => Some(payload),
        _ => None,
    }
}
