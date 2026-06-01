use std::sync::Arc;

use crate::services::dispatcher::cancel_store::{CancelStore, InMemoryCancelStore};

#[tokio::test]
async fn in_memory_cancel_lifecycle() {
    let store: Arc<dyn CancelStore> = Arc::new(InMemoryCancelStore::new());
    let id = uuid::Uuid::new_v4();

    assert!(!store.is_cancel_requested(id).await.expect("is"));
    store.request_cancel(id).await.expect("request");
    assert!(store.is_cancel_requested(id).await.expect("is 2"));

    let taken = store.take_cancel(id).await.expect("take");
    assert!(taken, "first take must consume the flag");
    assert!(!store.is_cancel_requested(id).await.expect("is 3"));

    let second = store.take_cancel(id).await.expect("take 2");
    assert!(!second, "second take is a no-op");
}
