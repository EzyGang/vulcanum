use std::sync::Arc;

use async_trait::async_trait;
use uuid::Uuid;

use crate::models::dispatcher::errors::DispatchError;

const CANCEL_KEY_TTL_SECS: u64 = 3_600;

#[async_trait]
pub trait CancelStore: Send + Sync {
    async fn request_cancel(&self, work_run_id: Uuid) -> Result<(), DispatchError>;
    async fn take_cancel(&self, work_run_id: Uuid) -> Result<bool, DispatchError>;
    async fn is_cancel_requested(&self, work_run_id: Uuid) -> Result<bool, DispatchError>;
}

#[derive(Clone)]
pub struct RedisCancelStore {
    client: redis::Client,
}

impl RedisCancelStore {
    pub fn new(redis_url: &str) -> Result<Self, DispatchError> {
        let client =
            redis::Client::open(redis_url).map_err(|e| DispatchError::Cancel(e.to_string()))?;
        Ok(Self { client })
    }
}

fn cancel_key(work_run_id: Uuid) -> String {
    format!("vulcanum:work_run:{work_run_id}:cancel")
}

#[async_trait]
impl CancelStore for RedisCancelStore {
    async fn request_cancel(&self, work_run_id: Uuid) -> Result<(), DispatchError> {
        let mut conn = self
            .client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| DispatchError::Cancel(e.to_string()))?;

        redis::cmd("SET")
            .arg(cancel_key(work_run_id))
            .arg("1")
            .arg("EX")
            .arg(CANCEL_KEY_TTL_SECS)
            .query_async::<()>(&mut conn)
            .await
            .map_err(|e| DispatchError::Cancel(e.to_string()))
    }

    async fn take_cancel(&self, work_run_id: Uuid) -> Result<bool, DispatchError> {
        let mut conn = self
            .client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| DispatchError::Cancel(e.to_string()))?;

        let value: Option<String> = redis::cmd("GETDEL")
            .arg(cancel_key(work_run_id))
            .query_async(&mut conn)
            .await
            .map_err(|e| DispatchError::Cancel(e.to_string()))?;

        Ok(value.is_some())
    }

    async fn is_cancel_requested(&self, work_run_id: Uuid) -> Result<bool, DispatchError> {
        let mut conn = self
            .client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| DispatchError::Cancel(e.to_string()))?;

        let exists: i64 = redis::cmd("EXISTS")
            .arg(cancel_key(work_run_id))
            .query_async(&mut conn)
            .await
            .map_err(|e| DispatchError::Cancel(e.to_string()))?;

        Ok(exists > 0)
    }
}

#[derive(Clone, Default)]
pub struct InMemoryCancelStore {
    inner: Arc<tokio::sync::RwLock<std::collections::HashMap<Uuid, ()>>>,
}

impl InMemoryCancelStore {
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
        }
    }
}

#[async_trait]
impl CancelStore for InMemoryCancelStore {
    async fn request_cancel(&self, work_run_id: Uuid) -> Result<(), DispatchError> {
        self.inner.write().await.insert(work_run_id, ());
        Ok(())
    }

    async fn take_cancel(&self, work_run_id: Uuid) -> Result<bool, DispatchError> {
        Ok(self.inner.write().await.remove(&work_run_id).is_some())
    }

    async fn is_cancel_requested(&self, work_run_id: Uuid) -> Result<bool, DispatchError> {
        Ok(self.inner.read().await.contains_key(&work_run_id))
    }
}
