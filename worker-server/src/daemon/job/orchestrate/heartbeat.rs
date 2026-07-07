use std::sync::Arc;
use std::time::Duration;

use tokio::sync::watch;

use crate::daemon::job::execution::event_reporter::EventReporter;

const HEARTBEAT_INTERVAL_SECS: u64 = 60;

pub(super) fn spawn_heartbeat(reporter: Arc<EventReporter>) -> watch::Sender<bool> {
    let (tx, mut rx) = watch::channel(false);
    tokio::spawn(async move {
        loop {
            tokio::select! {
                _ = tokio::time::sleep(Duration::from_secs(HEARTBEAT_INTERVAL_SECS)) => {
                    reporter.emit("worker.heartbeat", serde_json::json!({}));
                }
                changed = rx.changed() => {
                    if changed.is_err() || *rx.borrow() {
                        break;
                    }
                }
            }
        }
    });
    tx
}
