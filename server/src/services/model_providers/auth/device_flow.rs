use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::services::model_providers::auth::credentials::OAuthCredential;
use crate::services::model_providers::errors::ModelProvidersError;

#[derive(Clone, Debug)]
pub struct DeviceStart {
    pub device_auth_id: String,
    pub user_code: String,
    pub verification_uri: String,
    pub interval_seconds: i64,
}

#[derive(Clone, Debug)]
pub enum DevicePoll {
    Pending,
    Complete(OAuthCredential),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PendingDeviceFlow {
    pub attempt_id: Uuid,
    pub team_id: Uuid,
    pub user_id: Option<String>,
    pub provider_key: String,
    pub device_provider: String,
    pub display_name: String,
    pub device_auth_id: String,
    pub user_code: String,
    pub verification_uri: String,
    pub interval_seconds: i64,
    pub next_poll_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct RedisDeviceFlowStore {
    client: redis::Client,
}

#[async_trait]
pub trait DeviceFlowStore: Send + Sync {
    async fn insert(&self, attempt: PendingDeviceFlow) -> Result<(), ModelProvidersError>;

    async fn get(&self, attempt_id: Uuid)
        -> Result<Option<PendingDeviceFlow>, ModelProvidersError>;

    async fn update_next_poll(
        &self,
        attempt_id: Uuid,
        next_poll_at: DateTime<Utc>,
    ) -> Result<(), ModelProvidersError>;

    async fn consume(
        &self,
        attempt_id: Uuid,
    ) -> Result<Option<PendingDeviceFlow>, ModelProvidersError>;
}

#[async_trait]
pub trait DeviceAuthProvider: Send + Sync {
    fn provider_id(&self) -> &'static str;

    fn model_provider_key(&self) -> &'static str;

    async fn start(&self) -> Result<DeviceStart, ModelProvidersError>;

    async fn poll(&self, pending: &PendingDeviceFlow) -> Result<DevicePoll, ModelProvidersError>;

    async fn refresh(
        &self,
        credential: &OAuthCredential,
    ) -> Result<OAuthCredential, ModelProvidersError>;
}

#[derive(Clone, Default)]
pub struct InMemoryDeviceFlowStore {
    attempts: Arc<Mutex<HashMap<Uuid, PendingDeviceFlow>>>,
}

impl InMemoryDeviceFlowStore {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl RedisDeviceFlowStore {
    pub fn new(redis_url: &str) -> Result<Self, redis::RedisError> {
        Ok(Self {
            client: redis::Client::open(redis_url)?,
        })
    }
}

#[async_trait]
impl DeviceFlowStore for RedisDeviceFlowStore {
    async fn insert(&self, attempt: PendingDeviceFlow) -> Result<(), ModelProvidersError> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let key = device_flow_key(attempt.attempt_id);
        let ttl_secs = (attempt.expires_at - Utc::now()).num_seconds().max(1) as u64;
        let value = serde_json::to_string(&attempt)
            .map_err(|e| ModelProvidersError::InvalidAuthConfig(e.to_string()))?;

        redis::cmd("SETEX")
            .arg(key)
            .arg(ttl_secs)
            .arg(value)
            .query_async::<()>(&mut conn)
            .await?;
        Ok(())
    }

    async fn get(
        &self,
        attempt_id: Uuid,
    ) -> Result<Option<PendingDeviceFlow>, ModelProvidersError> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let value: Option<String> = redis::cmd("GET")
            .arg(device_flow_key(attempt_id))
            .query_async(&mut conn)
            .await?;
        value
            .map(|value| {
                serde_json::from_str::<PendingDeviceFlow>(&value)
                    .map_err(|e| ModelProvidersError::InvalidAuthConfig(e.to_string()))
            })
            .transpose()
    }

    async fn update_next_poll(
        &self,
        attempt_id: Uuid,
        next_poll_at: DateTime<Utc>,
    ) -> Result<(), ModelProvidersError> {
        let Some(mut attempt) = self.get(attempt_id).await? else {
            return Err(ModelProvidersError::DeviceFlowExpired);
        };
        attempt.next_poll_at = next_poll_at;
        self.insert(attempt).await
    }

    async fn consume(
        &self,
        attempt_id: Uuid,
    ) -> Result<Option<PendingDeviceFlow>, ModelProvidersError> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let script = redis::Script::new(
            r#"
            local v = redis.call("GET", KEYS[1])
            if v then
                redis.call("DEL", KEYS[1])
            end
            return v
        "#,
        );
        let value: Option<String> = script
            .key(device_flow_key(attempt_id))
            .invoke_async(&mut conn)
            .await?;

        value
            .map(|value| {
                serde_json::from_str::<PendingDeviceFlow>(&value)
                    .map_err(|e| ModelProvidersError::InvalidAuthConfig(e.to_string()))
            })
            .transpose()
    }
}

#[async_trait]
impl DeviceFlowStore for InMemoryDeviceFlowStore {
    async fn insert(&self, attempt: PendingDeviceFlow) -> Result<(), ModelProvidersError> {
        self.attempts
            .lock()
            .await
            .insert(attempt.attempt_id, attempt);
        Ok(())
    }

    async fn get(
        &self,
        attempt_id: Uuid,
    ) -> Result<Option<PendingDeviceFlow>, ModelProvidersError> {
        let attempt = self.attempts.lock().await.get(&attempt_id).cloned();
        match attempt.filter(|attempt| attempt.expires_at > Utc::now()) {
            Some(attempt) => Ok(Some(attempt)),
            None => Ok(None),
        }
    }

    async fn update_next_poll(
        &self,
        attempt_id: Uuid,
        next_poll_at: DateTime<Utc>,
    ) -> Result<(), ModelProvidersError> {
        match self.attempts.lock().await.get_mut(&attempt_id) {
            Some(attempt) => {
                attempt.next_poll_at = next_poll_at;
                Ok(())
            }
            None => Err(ModelProvidersError::DeviceFlowExpired),
        }
    }

    async fn consume(
        &self,
        attempt_id: Uuid,
    ) -> Result<Option<PendingDeviceFlow>, ModelProvidersError> {
        Ok(self.attempts.lock().await.remove(&attempt_id))
    }
}

fn device_flow_key(attempt_id: Uuid) -> String {
    format!("vulcanum:model_provider_device_flow:{attempt_id}")
}
