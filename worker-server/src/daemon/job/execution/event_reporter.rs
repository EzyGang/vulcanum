use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::{mpsc, watch, Mutex, RwLock};
use tokio::task::JoinHandle;

use vulcanum_shared::api_types::WireEvent;
use vulcanum_shared::client::ApiClient;
use vulcanum_shared::worker_state::WorkerState;

use crate::daemon::auth::with_retry_on_401;

const CANCEL_POLL_INTERVAL_SECS: u64 = 10;

pub(crate) struct EventReporter {
    job_id: uuid::Uuid,
    sequence: AtomicU64,
    sender: Mutex<Option<mpsc::UnboundedSender<WireEvent>>>,
    pump: Mutex<Option<JoinHandle<()>>>,
    cancel_rx: watch::Receiver<bool>,
}

impl EventReporter {
    pub(crate) fn new(
        client: Arc<ApiClient>,
        worker_state: Arc<RwLock<WorkerState>>,
        job_id: uuid::Uuid,
    ) -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();
        let (cancel_tx, cancel_rx) = watch::channel(false);
        let pump_cancel_tx = cancel_tx.clone();
        let pump = tokio::spawn(async move {
            run_event_pump(client, worker_state, job_id, receiver, pump_cancel_tx).await;
        });

        Self {
            job_id,
            sequence: AtomicU64::new(0),
            sender: Mutex::new(Some(sender)),
            pump: Mutex::new(Some(pump)),
            cancel_rx,
        }
    }

    pub(crate) async fn emit(&self, event_type: &str, payload: serde_json::Value) {
        let seq = self.sequence.fetch_add(1, Ordering::Relaxed) + 1;
        let wire = WireEvent {
            sequence: seq,
            event_type: event_type.to_owned(),
            payload,
            occurred_at: chrono::Utc::now(),
        };
        let sender = self.sender.lock().await;
        let Some(sender) = sender.as_ref() else {
            tracing::warn!(
                work_run_id = %self.job_id,
                sequence = seq,
                "event reporter pump already shut down"
            );
            return;
        };
        if sender.send(wire).is_err() {
            tracing::warn!(
                work_run_id = %self.job_id,
                sequence = seq,
                "event reporter pump is closed"
            );
        }
    }

    pub(crate) fn cancel_receiver(&self) -> watch::Receiver<bool> {
        self.cancel_rx.clone()
    }

    pub(crate) async fn shutdown(&self) {
        self.sender.lock().await.take();

        let pump = self.pump.lock().await.take();
        if let Some(pump) = pump {
            if let Err(error) = pump.await {
                tracing::warn!(
                    work_run_id = %self.job_id,
                    error = %error,
                    "event reporter pump task failed"
                );
            }
        }
    }
}

async fn run_event_pump(
    client: Arc<ApiClient>,
    worker_state: Arc<RwLock<WorkerState>>,
    job_id: uuid::Uuid,
    mut receiver: mpsc::UnboundedReceiver<WireEvent>,
    cancel_tx: watch::Sender<bool>,
) {
    let mut next_sequence = 1;
    let mut pending = BTreeMap::new();
    let poll_interval = Duration::from_secs(CANCEL_POLL_INTERVAL_SECS);
    let mut cancel_interval =
        tokio::time::interval_at(tokio::time::Instant::now() + poll_interval, poll_interval);
    cancel_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

    loop {
        tokio::select! {
            Some(event) = receiver.recv() => {
                pending.insert(event.sequence, event);
                send_ready_events(
                    &client,
                    &worker_state,
                    job_id,
                    &cancel_tx,
                    &mut pending,
                    &mut next_sequence,
                )
                .await;
            }
            _ = cancel_interval.tick() => {
                poll_cancel_request(&client, &worker_state, job_id, &cancel_tx).await;
            }
            else => break,
        }
    }

    for (_, event) in pending {
        send_event(&client, &worker_state, job_id, &cancel_tx, event).await;
    }
}

async fn send_ready_events(
    client: &Arc<ApiClient>,
    worker_state: &Arc<RwLock<WorkerState>>,
    job_id: uuid::Uuid,
    cancel_tx: &watch::Sender<bool>,
    pending: &mut BTreeMap<u64, WireEvent>,
    next_sequence: &mut u64,
) {
    while let Some(event) = pending.remove(next_sequence) {
        send_event(client, worker_state, job_id, cancel_tx, event).await;
        *next_sequence += 1;
    }
}

pub(super) async fn poll_cancel_request(
    client: &Arc<ApiClient>,
    worker_state: &Arc<RwLock<WorkerState>>,
    job_id: uuid::Uuid,
    cancel_tx: &watch::Sender<bool>,
) {
    if *cancel_tx.borrow() {
        return;
    }

    match with_retry_on_401(client, worker_state, |token| {
        let client = Arc::clone(client);
        async move { client.append_events(job_id, &[], &token).await }
    })
    .await
    {
        Ok(resp) => {
            if resp.should_cancel {
                let _ = cancel_tx.send(true);
                tracing::warn!(
                    work_run_id = %job_id,
                    "server requested cancel via polling response"
                );
            }
        }
        Err(error) => {
            tracing::debug!(
                work_run_id = %job_id,
                error = %error,
                "failed to poll server cancellation state"
            );
        }
    }
}

async fn send_event(
    client: &Arc<ApiClient>,
    worker_state: &Arc<RwLock<WorkerState>>,
    job_id: uuid::Uuid,
    cancel_tx: &watch::Sender<bool>,
    event: WireEvent,
) {
    let sequence = event.sequence;
    let events = vec![event];
    match with_retry_on_401(client, worker_state, |token| {
        let client = Arc::clone(client);
        let events = events.clone();
        async move { client.append_events(job_id, &events, &token).await }
    })
    .await
    {
        Ok(resp) => {
            if resp.accepted == 0 {
                tracing::debug!(
                    work_run_id = %job_id,
                    sequence,
                    "server accepted no new events"
                );
            }
            if resp.should_cancel {
                let _ = cancel_tx.send(true);
                tracing::warn!(
                    work_run_id = %job_id,
                    sequence,
                    "server requested cancel via event response"
                );
            }
        }
        Err(error) => {
            tracing::warn!(
                work_run_id = %job_id,
                sequence,
                error = %error,
                "failed to send event to server"
            );
        }
    }
}
