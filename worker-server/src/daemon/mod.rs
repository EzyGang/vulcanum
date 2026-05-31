pub(crate) mod job;
pub(crate) mod recovery;

use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Context;
use tokio::signal;
use tokio::sync::{Mutex, RwLock, Semaphore};
use tokio::time::sleep;

use vulcanum_shared::api_error::ApiError;
use vulcanum_shared::client::ApiClient;
use vulcanum_shared::token::ensure_valid_token;
use vulcanum_shared::validate::is_environment_ready_for_backend;
use vulcanum_shared::worker_state::{load_state, WorkerState};

use crate::state::journal::Journal;

use job::handle_job;
use recovery::recover_running_jobs;

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

    tracing::info!("daemon started, running recovery");

    let recovery_result =
        recover_running_jobs(journal.clone(), client.clone(), worker_state.clone()).await;

    let available_permits = (daemon_state.worker_state.read().await.max_concurrent_jobs as usize)
        .saturating_sub(recovery_result.recovered_count);
    tracing::info!(
        recovered = recovery_result.recovered_count,
        available = available_permits,
        "recovery complete, starting poll loop"
    );

    let _monitors = recovery_result.monitors;

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

async fn tick(state: &DaemonState, refresh_buffer_secs: i64) -> TickOutcome {
    {
        let mut worker_state = state.worker_state.write().await;
        if let Err(e) =
            ensure_valid_token(&state.client, &mut worker_state, refresh_buffer_secs).await
        {
            if is_fatal_api_error(&e) {
                return TickOutcome::Fatal(format!(
                    "token refresh failed permanently: {e:#} — run `vulcanum worker setup --instance <instance> --code <code>` to reconnect"
                ));
            }
            tracing::warn!("token refresh failed: {e:#} — if this persists, try `vulcanum worker setup --instance <instance> --code <code>`");
            return TickOutcome::Transient(e.to_string());
        }
    }

    try_drain_queue(state).await;

    let access_token = state.worker_state.read().await.access_token.clone();

    tracing::info!("polling server for jobs");

    match state.client.poll(&access_token).await {
        Ok(Some(job_id)) => {
            {
                let mut queue = state.pending_queue.lock().await;
                queue.push_back(job_id);
            }
            try_drain_queue(state).await;
            TickOutcome::Success
        }
        Ok(None) => {
            tracing::info!("no jobs available, sleeping {POLL_INTERVAL_SECS}s");
            sleep(std::time::Duration::from_secs(POLL_INTERVAL_SECS)).await;
            TickOutcome::Success
        }
        Err(e) => {
            if is_fatal_api_error(&e) {
                return TickOutcome::Fatal(format!(
                    "poll failed permanently: {e:#} — run `vulcanum worker setup --instance <instance> --code <code>` to reconnect"
                ));
            }
            TickOutcome::Transient(e.to_string())
        }
    }
}

async fn try_drain_queue(state: &DaemonState) {
    loop {
        let semaphore = Arc::clone(&state.semaphore);
        let Ok(permit) = semaphore.try_acquire_owned() else {
            break;
        };

        let job_id = {
            let mut queue = state.pending_queue.lock().await;
            queue.pop_front()
        };

        let Some(job_id) = job_id else {
            break;
        };

        let client = Arc::clone(&state.client);
        let worker_state = Arc::clone(&state.worker_state);
        let journal = Arc::clone(&state.journal);
        let shutdown_tx = state.shutdown_tx.clone();
        let harness_type = state.harness_type.clone();

        tokio::spawn(async move {
            let _permit = permit;
            if let Err(msg) = handle_job(client, worker_state, journal, job_id, &harness_type).await
            {
                let _ = shutdown_tx.send(Some(msg));
            }
        });
    }
}

fn is_fatal_api_error(e: &anyhow::Error) -> bool {
    e.downcast_ref::<ApiError>().is_some_and(|a| a.is_fatal())
}

fn journal_db_path() -> anyhow::Result<PathBuf> {
    let dir = dirs::config_dir()
        .context("failed to find config directory")?
        .join("vulcanum");
    std::fs::create_dir_all(&dir)
        .with_context(|| format!("failed to create config dir {}", dir.display()))?;
    Ok(dir.join("worker.db"))
}
