use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::models::model_providers::errors::ModelProvidersError;
use crate::services::model_providers::auth::device_flow::{
    DeviceFlowStore, InMemoryDeviceFlowStore, PendingDeviceFlow, RedisDeviceFlowStore,
};

#[tokio::test]
async fn in_memory_store_round_trips_updates_and_consumes_attempts() {
    let store = InMemoryDeviceFlowStore::new();
    let attempt = pending_attempt(Utc::now() + chrono::Duration::minutes(5));
    let attempt_id = attempt.attempt_id;
    let next_poll_at = Utc::now() + chrono::Duration::seconds(30);

    store.insert(attempt.clone()).await.expect("insert attempt");

    let stored = store
        .get(attempt_id)
        .await
        .expect("get attempt")
        .expect("attempt exists");
    assert_eq!(stored.device_auth_id, attempt.device_auth_id);

    store
        .update_next_poll(attempt_id, next_poll_at)
        .await
        .expect("update next poll");
    let updated = store
        .get(attempt_id)
        .await
        .expect("get updated attempt")
        .expect("updated attempt exists");
    assert_eq!(updated.next_poll_at, next_poll_at);

    let consumed = store
        .consume(attempt_id)
        .await
        .expect("consume attempt")
        .expect("attempt consumed");
    assert_eq!(consumed.attempt_id, attempt_id);
    assert!(store.get(attempt_id).await.expect("get consumed").is_none());
}

#[tokio::test]
async fn in_memory_store_hides_expired_attempts() {
    let store = InMemoryDeviceFlowStore::new();
    let attempt = pending_attempt(Utc::now() - chrono::Duration::seconds(1));
    let attempt_id = attempt.attempt_id;

    store.insert(attempt).await.expect("insert attempt");

    assert!(store.get(attempt_id).await.expect("get attempt").is_none());
}

#[tokio::test]
async fn redis_store_round_trips_updates_and_consumes_attempts() {
    let Some(store) = available_redis_store().await else {
        return;
    };
    let attempt = pending_attempt(Utc::now() + chrono::Duration::minutes(5));
    let attempt_id = attempt.attempt_id;
    let next_poll_at = Utc::now() + chrono::Duration::seconds(30);

    store.insert(attempt.clone()).await.expect("insert attempt");

    let stored = store
        .get(attempt_id)
        .await
        .expect("get attempt")
        .expect("attempt exists");
    assert_eq!(stored.device_auth_id, attempt.device_auth_id);

    store
        .update_next_poll(attempt_id, next_poll_at)
        .await
        .expect("update next poll");
    let updated = store
        .get(attempt_id)
        .await
        .expect("get updated attempt")
        .expect("updated attempt exists");
    assert_eq!(updated.next_poll_at, next_poll_at);

    let consumed = store
        .consume(attempt_id)
        .await
        .expect("consume attempt")
        .expect("attempt consumed");
    assert_eq!(consumed.attempt_id, attempt_id);
    assert!(store.get(attempt_id).await.expect("get consumed").is_none());
}

#[tokio::test]
async fn redis_update_next_poll_returns_expired_for_missing_attempts() {
    let Some(store) = available_redis_store().await else {
        return;
    };

    let err = store
        .update_next_poll(Uuid::new_v4(), Utc::now())
        .await
        .expect_err("missing attempt should fail");
    assert!(matches!(err, ModelProvidersError::DeviceFlowExpired));
}

async fn available_redis_store() -> Option<RedisDeviceFlowStore> {
    let redis_url =
        std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_owned());
    let store = RedisDeviceFlowStore::new(&redis_url).ok()?;
    match store.consume(Uuid::new_v4()).await {
        Ok(_) => Some(store),
        Err(ModelProvidersError::Redis(_)) => None,
        Err(_) => Some(store),
    }
}

fn pending_attempt(expires_at: DateTime<Utc>) -> PendingDeviceFlow {
    PendingDeviceFlow {
        attempt_id: Uuid::new_v4(),
        team_id: Uuid::new_v4(),
        user_id: Some("user-1".to_owned()),
        provider_key: "openai".to_owned(),
        device_provider: "openai_chatgpt".to_owned(),
        display_name: "ChatGPT Plus".to_owned(),
        device_auth_id: "device-auth-id".to_owned(),
        user_code: "ABCD-EFGH".to_owned(),
        verification_uri: "https://example.com/device".to_owned(),
        interval_seconds: 5,
        next_poll_at: Utc::now() + chrono::Duration::seconds(5),
        expires_at,
    }
}
