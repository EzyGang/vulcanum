mod job;

use std::time::Duration;

use anyhow::Context;
use tokio::signal;
use tokio::time::sleep;

use vulcanum_shared::api_error::ApiError;
use vulcanum_shared::client::ApiClient;
use vulcanum_shared::token::ensure_valid_token;
use vulcanum_shared::validate::is_environment_ready;
use vulcanum_shared::worker_state::{load_state, WorkerState};

use job::handle_job;

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

pub async fn run() -> anyhow::Result<()> {
    if !is_environment_ready() {
        tracing::error!("environment validation failed — run `vulcanum worker setup` for details");
        return Err(anyhow::anyhow!(
            "worker environment is not ready — run `vulcanum worker setup` to diagnose"
        ));
    }

    let mut state = load_state()?.ok_or_else(|| {
        anyhow::anyhow!(
            "no worker state found — run `vulcanum worker connect <instance> --code <code>` first"
        )
    })?;

    let client = ApiClient::new(state.instance_url.clone());

    let status = client
        .status()
        .await
        .context("failed to fetch server status")?;

    let refresh_buffer_secs = status.access_token_ttl_minutes * 60 / 3;

    tracing::info!(
        worker_id = state.worker_id.to_string().as_str(),
        poll_interval = POLL_INTERVAL_SECS,
        refresh_buffer = refresh_buffer_secs,
        "daemon started",
    );

    let mut backoff_ms = INITIAL_BACKOFF_MS;

    loop {
        tokio::select! {
            _ = signal::ctrl_c() => {
                tracing::info!("received SIGINT, shutting down");
                return Ok(());
            }
            result = tick(&client, &mut state, refresh_buffer_secs) => {
                match result {
                    TickOutcome::Success => {
                        backoff_ms = INITIAL_BACKOFF_MS;
                    }
                    TickOutcome::Fatal(msg) => {
                        tracing::error!("{msg}");
                        return Ok(());
                    }
                    TickOutcome::Transient(msg) => {
                        tracing::warn!(
                            "tick failed: {msg}, retrying in {backoff_ms}ms"
                        );
                        sleep(Duration::from_millis(backoff_ms)).await;
                        backoff_ms =
                            (backoff_ms * BACKOFF_MULTIPLIER).min(MAX_BACKOFF_MS);
                    }
                }
            }
        }
    }
}

async fn tick(
    client: &ApiClient,
    state: &mut WorkerState,
    refresh_buffer_secs: i64,
) -> TickOutcome {
    if let Err(e) = ensure_valid_token(client, state, refresh_buffer_secs).await {
        if is_fatal_api_error(&e) {
            return TickOutcome::Fatal(format!(
                "token refresh failed permanently: {e:#} — run `vulcanum worker connect <instance> --code <code>` to reconnect"
            ));
        }
        tracing::warn!("token refresh failed: {e:#} — if this persists, try `vulcanum worker connect <instance> --code <code>`");
        return TickOutcome::Transient(e.to_string());
    }

    tracing::info!("polling server for jobs");

    match client.poll(&state.access_token).await {
        Ok(Some(job_id)) => {
            return handle_job(client, state, job_id).await;
        }
        Ok(None) => {
            tracing::info!("no jobs available, sleeping {POLL_INTERVAL_SECS}s");
            sleep(Duration::from_secs(POLL_INTERVAL_SECS)).await;
            TickOutcome::Success
        }
        Err(e) => {
            if is_fatal_api_error(&e) {
                return TickOutcome::Fatal(format!(
                    "poll failed permanently: {e:#} — run `vulcanum worker connect <instance> --code <code>` to reconnect"
                ));
            }
            TickOutcome::Transient(e.to_string())
        }
    }
}

fn is_fatal_api_error(e: &anyhow::Error) -> bool {
    e.downcast_ref::<ApiError>().is_some_and(|a| a.is_fatal())
}
