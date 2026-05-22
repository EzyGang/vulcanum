mod job;

use job::handle_job;

use anyhow::Context;
use std::time::Duration;

use tokio::signal;
use tokio::time::sleep;

use crate::api_error::ApiError;
use crate::client::ApiClient;
use crate::harness::validate::is_environment_ready;
use crate::state::{load_state, WorkerState};
use crate::token::ensure_valid_token;

const POLL_INTERVAL_SECS: u64 = 5;
const INITIAL_BACKOFF_MS: u64 = 1_000;
const MAX_BACKOFF_MS: u64 = 60_000;
const BACKOFF_MULTIPLIER: u64 = 2;

#[derive(Debug, PartialEq)]
enum TickOutcome {
    Success,
    Fatal(String),
    Transient,
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
        "daemon started, worker_id: {}, poll_interval: {}s, refresh_buffer: {}s",
        state.worker_id,
        POLL_INTERVAL_SECS,
        refresh_buffer_secs
    );

    let mut backoff_ms = INITIAL_BACKOFF_MS;

    loop {
        tokio::select! {
            _ = signal::ctrl_c() => {
                tracing::info!("received SIGINT, shutting down");
                return Ok(());
            }
            result = tick(&client, &mut state, refresh_buffer_secs,
            ) => {
                match result {
                    TickOutcome::Success => {
                        backoff_ms = INITIAL_BACKOFF_MS;
                    }
                    TickOutcome::Fatal(msg) => {
                        tracing::error!("{msg}");
                        return Ok(());
                    }
                    TickOutcome::Transient => {
                        tracing::error!("tick failed, retrying in {}ms", backoff_ms);
                        sleep(Duration::from_millis(backoff_ms)).await;
                        backoff_ms = (backoff_ms * BACKOFF_MULTIPLIER).min(MAX_BACKOFF_MS);
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
                "token refresh failed permanently — reconnection required: {:#}",
                e
            ));
        }
        return TickOutcome::Transient;
    }

    match client.poll(&state.access_token).await {
        Ok(Some(job_id)) => {
            return handle_job(client, state, job_id).await;
        }
        Ok(None) => {
            sleep(Duration::from_secs(POLL_INTERVAL_SECS)).await;
            TickOutcome::Success
        }
        Err(e) => {
            if is_fatal_api_error(&e) {
                return TickOutcome::Fatal(format!("poll failed with fatal error: {:#}", e));
            }
            TickOutcome::Transient
        }
    }
}

fn is_fatal_api_error(e: &anyhow::Error) -> bool {
    e.downcast_ref::<ApiError>().is_some_and(|a| a.is_fatal())
}
