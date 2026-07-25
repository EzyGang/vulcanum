pub(crate) mod auth;
#[cfg(test)]
mod auth_tests;
pub(crate) mod job;
pub(crate) mod queue;
#[cfg(test)]
mod queue_tests;
mod tick;
#[cfg(test)]
mod update_tests;

use std::collections::VecDeque;
use std::path::Path;
use std::sync::Arc;

use anyhow::Context;
use tokio::signal;
use tokio::sync::{Mutex, RwLock, Semaphore};
use tokio::time::{sleep_until, Duration, Instant};

use vulcanum_shared::client::ApiClient;
use vulcanum_shared::config::{load_config, WorkerConfig};
use vulcanum_shared::state::paths;
use vulcanum_shared::state::worker::{load_state, WorkerState};
use vulcanum_shared::validate::is_environment_ready_for_config;

use crate::daemon::queue::JobTracker;
use crate::recovery;
use crate::state::journal::Journal;
use crate::update::AutomaticUpdater;

use tick::tick;

const INITIAL_BACKOFF_MS: u64 = 1_000;
const MAX_BACKOFF_MS: u64 = 60_000;
const BACKOFF_MULTIPLIER: u64 = 2;
const MIN_UPDATE_CHECK_INTERVAL_SECS: u64 = 60;
const MAX_UPDATE_CHECK_INTERVAL_SECS: u64 = 365 * 24 * 60 * 60;
const UPDATE_IDLE_RETRY_SECS: u64 = 1;

#[derive(Debug, PartialEq)]
enum TickOutcome {
    Success,
    Idle,
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
    job_tracker: Arc<JobTracker>,
    config: WorkerConfig,
}

pub async fn run(pending_rollback: Option<&Path>) -> anyhow::Result<()> {
    let config = load_config().context("failed to load worker config")?;
    let update_interval = automatic_update_interval(&config)?;

    if !is_environment_ready_for_config(&config) {
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

    let journal_path = paths::db_path()?;
    let journal = Arc::new(Journal::open(&journal_path).context("failed to open job journal")?);

    let semaphore = Arc::new(Semaphore::new(state.max_concurrent_jobs as usize));
    let client = Arc::new(client);
    let worker_state = Arc::new(RwLock::new(state));

    let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(None);
    let pending_queue = Mutex::new(VecDeque::new());
    let job_tracker = Arc::new(JobTracker::default());

    let daemon_state = DaemonState {
        client: client.clone(),
        worker_state: worker_state.clone(),
        journal: journal.clone(),
        semaphore: semaphore.clone(),
        shutdown_rx,
        shutdown_tx,
        pending_queue,
        job_tracker: Arc::clone(&job_tracker),
        config,
    };

    recovery::reconcile_running_jobs(&journal, &client, &worker_state, &job_tracker)
        .await
        .context("failed to reconcile running jobs")?;
    if let Some(rollback_dir) = pending_rollback {
        AutomaticUpdater::confirm_current_install(rollback_dir)
            .context("failed to confirm updated worker startup")?;
        tracing::info!(
            rollback_dir = %rollback_dir.display(),
            "confirmed updated worker startup"
        );
    }
    if daemon_state.config.auto_update_enabled
        && daemon_state.job_tracker.is_idle().await
        && run_automatic_update().await?
    {
        return Ok(());
    }

    tracing::info!("daemon started, starting poll loop");

    let mut backoff_ms = INITIAL_BACKOFF_MS;
    let mut next_poll = Instant::now();
    let mut next_update_check = schedule_after(update_interval)?;

    loop {
        tokio::select! {
            _ = signal::ctrl_c() => {
                tracing::info!("received SIGINT, shutting down");
                return Ok(());
            }
            _ = sleep_until(next_update_check), if daemon_state.config.auto_update_enabled => {
                if daemon_state.job_tracker.is_idle().await
                    && daemon_state.pending_queue.lock().await.is_empty()
                {
                    next_update_check = schedule_after(update_interval)?;
                    if run_automatic_update().await? {
                        return Ok(());
                    }
                } else {
                    next_update_check =
                        schedule_after(Duration::from_secs(UPDATE_IDLE_RETRY_SECS))?;
                }
            }
            _ = sleep_until(next_poll) => {
                match tick(&daemon_state, refresh_buffer_secs).await {
                    TickOutcome::Success => {
                        backoff_ms = INITIAL_BACKOFF_MS;
                        next_poll = Instant::now();
                    }
                    TickOutcome::Idle => {
                        backoff_ms = INITIAL_BACKOFF_MS;
                        next_poll = schedule_after(Duration::from_secs(
                            daemon_state.config.poll_interval_secs
                        ))?;
                    }
                    TickOutcome::Fatal(msg) => {
                        tracing::error!("{msg}");
                        return Err(anyhow::anyhow!("{msg}"));
                    }
                    TickOutcome::Transient(msg) => {
                        tracing::warn!(
                            "tick failed: {msg}, retrying in {backoff_ms}ms"
                        );
                        next_poll =
                            schedule_after(Duration::from_millis(backoff_ms))?;
                        backoff_ms =
                            (backoff_ms * BACKOFF_MULTIPLIER).min(MAX_BACKOFF_MS);
                    }
                }

                if let Some(msg) = &*daemon_state.shutdown_rx.borrow() {
                    tracing::error!("job task failed permanently: {msg}");
                    return Err(anyhow::anyhow!("{msg}"));
                }
            }
        }
    }
}

fn automatic_update_interval(config: &WorkerConfig) -> anyhow::Result<Duration> {
    if !config.auto_update_enabled {
        return Ok(Duration::ZERO);
    }
    if !(MIN_UPDATE_CHECK_INTERVAL_SECS..=MAX_UPDATE_CHECK_INTERVAL_SECS)
        .contains(&config.update_check_interval_secs)
    {
        anyhow::bail!(
            "update_check_interval_secs must be between {MIN_UPDATE_CHECK_INTERVAL_SECS} and {MAX_UPDATE_CHECK_INTERVAL_SECS}"
        );
    }
    Ok(Duration::from_secs(config.update_check_interval_secs))
}

fn schedule_after(interval: Duration) -> anyhow::Result<Instant> {
    Instant::now()
        .checked_add(interval)
        .ok_or_else(|| anyhow::anyhow!("daemon deadline cannot be scheduled"))
}

async fn run_automatic_update() -> anyhow::Result<bool> {
    let updater = match AutomaticUpdater::for_current_install() {
        Ok(updater) => updater,
        Err(error) => {
            tracing::warn!(
                target_version = "unknown",
                error = %error,
                "automatic update failed before release discovery; continuing with the working installation"
            );
            return Ok(false);
        }
    };
    let outcome = updater.check_and_apply().await;
    outcome.log();
    if outcome.is_applied() {
        return Ok(true);
    }
    if AutomaticUpdater::current_install_has_transaction()? {
        anyhow::bail!("automatic update recovery is pending; restarting the worker daemon");
    }
    Ok(false)
}
