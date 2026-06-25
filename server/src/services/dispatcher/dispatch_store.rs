use std::sync::Arc;

use async_trait::async_trait;
use uuid::Uuid;

use crate::models::dispatcher::errors::DispatchError;

const DISPATCH_KEY_TTL_SECS: u64 = 300;

#[async_trait]
pub trait DispatchStore: Send + Sync {
    async fn set_dispatched(&self, worker_id: Uuid, work_run_id: Uuid)
        -> Result<(), DispatchError>;
    async fn take_dispatched(&self, worker_id: Uuid) -> Result<Option<Uuid>, DispatchError>;
}

#[derive(Clone)]
pub struct RedisDispatchStore {
    client: redis::Client,
}

impl RedisDispatchStore {
    pub fn new(redis_url: &str) -> Result<Self, DispatchError> {
        let client = redis::Client::open(redis_url)?;
        Ok(Self { client })
    }
}

#[async_trait]
impl DispatchStore for RedisDispatchStore {
    async fn set_dispatched(
        &self,
        worker_id: Uuid,
        work_run_id: Uuid,
    ) -> Result<(), DispatchError> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;

        let key = dispatch_key(worker_id);

        redis::cmd("SET")
            .arg(&key)
            .arg(work_run_id.to_string())
            .arg("EX")
            .arg(DISPATCH_KEY_TTL_SECS)
            .query_async::<()>(&mut conn)
            .await?;

        Ok(())
    }

    async fn take_dispatched(&self, worker_id: Uuid) -> Result<Option<Uuid>, DispatchError> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;

        let key = dispatch_key(worker_id);

        let script = redis::Script::new(
            r#"local v = redis.call("GET", KEYS[1])
               if v then
                   redis.call("DEL", KEYS[1])
               end
               return v"#,
        );

        let value: Option<String> = script.key(&key).invoke_async(&mut conn).await?;

        match value {
            None => Ok(None),
            Some(s) => Uuid::parse_str(&s)
                .map(Some)
                .map_err(|e| DispatchError::Internal(e.to_string())),
        }
    }
}

fn dispatch_key(worker_id: Uuid) -> String {
    format!("vulcanum:worker:{worker_id}:dispatched")
}

#[derive(Clone, Default)]
pub struct InMemoryDispatchStore {
    inner: Arc<tokio::sync::RwLock<std::collections::HashMap<Uuid, Uuid>>>,
}

impl InMemoryDispatchStore {
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
        }
    }
}

#[async_trait]
impl DispatchStore for InMemoryDispatchStore {
    async fn set_dispatched(
        &self,
        worker_id: Uuid,
        work_run_id: Uuid,
    ) -> Result<(), DispatchError> {
        self.inner.write().await.insert(worker_id, work_run_id);
        Ok(())
    }

    async fn take_dispatched(&self, worker_id: Uuid) -> Result<Option<Uuid>, DispatchError> {
        Ok(self.inner.write().await.remove(&worker_id))
    }
}
