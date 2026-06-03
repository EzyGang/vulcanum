pub(crate) mod job;
mod queue;
mod tick;

use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Context;
use tokio::signal;
use tokio::sync::{Mutex, RwLock, Semaphore};
use tokio::time::sleep;

use vulcanum_shared::client::ApiClient;
use vulcanum_shared::validate::is_environment_ready_for_backend;
use vulcanum_shared::worker_state::{load_state, WorkerState};

use crate::runtime::recovery;
use crate::state::journal::Journal;

use tick::tick;

const POLL_INTERVAL_SECS: u64 = 15;
const INITIAL_BACKOFF_MS: u64 = 1_000;
const MAX_BACKOFF_MS: u64 = 60_000;
const BACKOFF_MULTIPLIER: u64 = 2;

#[derive(Debug, PartialEq)]
enum TickOutcome {
    Success,
    Fatal(String),
    Transient(String),
}

struct DaemonState {
    client: Arc<ApiClient>,
    worker_state: Arc<RwLock<WorkerState>>,
    journal: Arc<Journal>,
    semaphore: Arc<Semaphore>,
    shutdown_rx: tokio::sync::watch::Receiver<Option<String>>,
    shutdown_tx: tokio::sync::watch::Sender<Option<String>>,
    pending_queue: Mutex<VecDeque<uuid::Uuid>>,
    harness_type: String,
}

pub async fn run() -> anyhow::Result<()> {
    let harness_type = std::env::var("VULCANUM_HARNESS").unwrap_or_else(|_| "host".to_owned());
    if !is_environment_ready_for_backend(&harness_type) {
        tracing::error!("environment validation failed — run `vulcanum worker setup` for details");
        return Err(anyhow::anyhow!(
            "worker environment is not ready — run `vulcanum worker setup` to diagnose"
        ));
    }

    let state = load_state()?.ok_or_else(|| {
        anyhow::anyhow!(
            "no worker state found — run `vulcanum worker setup --instance <instance> --code <code>` first"
        )
    })?;

    let client = ApiClient::new(state.instance_url.clone());

    let status = client
        .status()
        .await
        .context("failed to fetch server status")?;

    let refresh_buffer_secs = status.access_token_ttl_minutes * 60 / 3;

    let journal_path = journal_db_path()?;
    let journal = Arc::new(Journal::open(&journal_path).context("failed to open job journal")?);

    let semaphore = Arc::new(Semaphore::new(state.max_concurrent_jobs as usize));
    let client = Arc::new(client);
    let worker_state = Arc::new(RwLock::new(state));

    let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(None);
    let pending_queue = Mutex::new(VecDeque::new());

    let daemon_state = DaemonState {
        client: client.clone(),
        worker_state: worker_state.clone(),
        journal: journal.clone(),
        semaphore: semaphore.clone(),
        shutdown_rx,
        shutdown_tx,
        pending_queue,
        harness_type,
    };

    recovery::reconcile_running_jobs(&journal, &client, &worker_state).await;

    tracing::info!("daemon started, starting poll loop");

    let mut backoff_ms = INITIAL_BACKOFF_MS;

    loop {
        tokio::select! {
            _ = signal::ctrl_c() => {
                tracing::info!("received SIGINT, shutting down");
                return Ok(());
            }
            result = tick(&daemon_state, refresh_buffer_secs) => {
                match result {
                    TickOutcome::Success => {
                        backoff_ms = INITIAL_BACKOFF_MS;
                    }
                    TickOutcome::Fatal(msg) => {
                        tracing::error!("{msg}");
                        return Err(anyhow::anyhow!("{msg}"));
                    }
                    TickOutcome::Transient(msg) => {
                        tracing::warn!(
                            "tick failed: {msg}, retrying in {backoff_ms}ms"
                        );
                        sleep(std::time::Duration::from_millis(backoff_ms)).await;
                        backoff_ms =
                            (backoff_ms * BACKOFF_MULTIPLIER).min(MAX_BACKOFF_MS);
                    }
                }
            }
        }

        if let Some(ref msg) = *daemon_state.shutdown_rx.borrow() {
            tracing::error!("job task failed permanently: {msg}");
            return Err(anyhow::anyhow!("{msg}"));
        }
    }
}

fn journal_db_path() -> anyhow::Result<PathBuf> {
    let dir = dirs::config_dir()
        .context("failed to find config directory")?
        .join("vulcanum");
    std::fs::create_dir_all(&dir)
        .with_context(|| format!("failed to create config dir {}", dir.display()))?;
    Ok(dir.join("worker.db"))
}
