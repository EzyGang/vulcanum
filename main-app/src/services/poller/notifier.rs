use std::collections::HashMap;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::Arc;

use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Clone)]
pub struct WorkNotifier {
    flags: Arc<RwLock<HashMap<Uuid, Arc<AtomicBool>>>>,
}

impl WorkNotifier {
    pub fn new() -> Self {
        Self {
            flags: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn notify_all(&self) {
        let flags = self.flags.read().await;
        for flag in flags.values() {
            flag.store(true, Ordering::Release);
        }
    }

    #[allow(dead_code)]
    pub async fn add_worker(&self, worker_id: Uuid) {
        self.flags
            .write()
            .await
            .entry(worker_id)
            .or_insert_with(|| Arc::new(AtomicBool::new(false)));
    }

    #[allow(dead_code)]
    pub async fn take(&self, worker_id: &Uuid) -> bool {
        match self.flags.read().await.get(worker_id) {
            Some(flag) => flag.swap(false, Ordering::AcqRel),
            None => false,
        }
    }

    #[allow(dead_code)]
    pub async fn remove_worker(&self, worker_id: &Uuid) {
        self.flags.write().await.remove(worker_id);
    }
}
