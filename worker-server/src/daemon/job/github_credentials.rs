use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use chrono::{DateTime, Utc};
use tokio::sync::{watch, RwLock};
use uuid::Uuid;

use vulcanum_shared::api::error::ApiError;
use vulcanum_shared::api::wire::RefreshGithubTokenResponse;
use vulcanum_shared::client::ApiClient;
use vulcanum_shared::state::worker::WorkerState;

use crate::daemon::auth::with_retry_on_401;
use crate::isolation::github_credentials as isolation_github_credentials;

const REFRESH_BEFORE_EXPIRY_SECS: i64 = 600;
const FALLBACK_REFRESH_INTERVAL_SECS: u64 = 3_000;
const RETRY_INTERVAL_SECS: u64 = 60;

pub(crate) fn spawn_refresh_task(
    client: Arc<ApiClient>,
    worker_state: Arc<RwLock<WorkerState>>,
    job_id: Uuid,
    workdir: PathBuf,
    expires_at: Option<DateTime<Utc>>,
) -> watch::Sender<bool> {
    let (stop_tx, stop_rx) = watch::channel(false);
    tokio::spawn(refresh_loop(
        client,
        worker_state,
        job_id,
        workdir,
        expires_at,
        stop_rx,
    ));
    stop_tx
}

pub(crate) fn stop_refresh_task(stop: Option<watch::Sender<bool>>) {
    if let Some(stop) = stop {
        let _ = stop.send(true);
    }
}

pub(crate) async fn setup_recovered_credentials(
    workdir: &Path,
    harness_type: &str,
    token: Option<&str>,
) -> Result<
    isolation_github_credentials::GitHubCredentialBridge,
    vulcanum_shared::runtime::errors::HarnessError,
> {
    let runtime_home = match harness_type {
        "docker" | "kata" => "/workdir/home".to_owned(),
        _ => workdir.join("home").to_string_lossy().to_string(),
    };

    isolation_github_credentials::setup(workdir, token, &runtime_home).await
}

async fn refresh_loop(
    client: Arc<ApiClient>,
    worker_state: Arc<RwLock<WorkerState>>,
    job_id: Uuid,
    workdir: PathBuf,
    mut expires_at: Option<DateTime<Utc>>,
    mut stop_rx: watch::Receiver<bool>,
) {
    loop {
        let delay = refresh_delay(expires_at);

        tokio::select! {
            _ = tokio::time::sleep(delay) => (),
            changed = stop_rx.changed() => {
                if changed.is_err() || *stop_rx.borrow() {
                    return;
                }
                continue;
            }
        }

        match refresh_once(&client, &worker_state, job_id, &workdir).await {
            Ok(next_expires_at) => {
                expires_at = next_expires_at;
            }
            Err(e) if is_retryable_refresh_error(&e) => {
                tracing::warn!(
                    work_run_id = %job_id,
                    error = %e,
                    retry_secs = RETRY_INTERVAL_SECS,
                    "failed to refresh github token"
                );
                if should_stop_after_retry(&mut stop_rx).await {
                    return;
                }
                expires_at =
                    Some(Utc::now() + chrono::Duration::seconds(RETRY_INTERVAL_SECS as i64));
            }
            Err(e) => {
                match e.downcast_ref::<ApiError>() {
                    Some(api_error) => {
                        tracing::warn!(
                            work_run_id = %job_id,
                            http_status = api_error.status,
                            response_body = %api_error.body,
                            "stopping github token refresh after non-retryable error"
                        );
                    }
                    None => {
                        tracing::warn!(
                            work_run_id = %job_id,
                            error = %e,
                            "stopping github token refresh after non-retryable error"
                        );
                    }
                }
                return;
            }
        }
    }
}

async fn refresh_once(
    client: &Arc<ApiClient>,
    worker_state: &Arc<RwLock<WorkerState>>,
    job_id: Uuid,
    workdir: &Path,
) -> anyhow::Result<Option<DateTime<Utc>>> {
    let response = with_retry_on_401(client, worker_state, |token| {
        let client = client.clone();
        async move { client.refresh_github_token(job_id, &token).await }
    })
    .await?;

    apply_refresh_response(job_id, workdir, response).await
}

async fn apply_refresh_response(
    job_id: Uuid,
    workdir: &Path,
    response: RefreshGithubTokenResponse,
) -> anyhow::Result<Option<DateTime<Utc>>> {
    match response.github_token.as_deref() {
        Some(token) => {
            isolation_github_credentials::update_token(workdir, Some(token)).await?;
            tracing::debug!(
                work_run_id = %job_id,
                expires_at = ?response.github_token_expires_at,
                "github token refreshed"
            );
        }
        None => {
            isolation_github_credentials::update_token(workdir, None).await?;
            tracing::debug!(work_run_id = %job_id, "github token refresh returned no token");
        }
    }

    Ok(response.github_token_expires_at)
}

pub(crate) fn is_retryable_refresh_error(error: &anyhow::Error) -> bool {
    match error.downcast_ref::<ApiError>() {
        Some(api_error) => {
            !matches!(api_error.status, 400..=499) || matches!(api_error.status, 408 | 429)
        }
        None => true,
    }
}

async fn should_stop_after_retry(stop_rx: &mut watch::Receiver<bool>) -> bool {
    tokio::select! {
        _ = tokio::time::sleep(Duration::from_secs(RETRY_INTERVAL_SECS)) => false,
        changed = stop_rx.changed() => changed.is_err() || *stop_rx.borrow(),
    }
}

fn refresh_delay(expires_at: Option<DateTime<Utc>>) -> Duration {
    let Some(expires_at) = expires_at else {
        return Duration::from_secs(FALLBACK_REFRESH_INTERVAL_SECS);
    };

    let refresh_at = expires_at - chrono::Duration::seconds(REFRESH_BEFORE_EXPIRY_SECS);
    let delay_secs = (refresh_at - Utc::now()).num_seconds().max(1) as u64;
    Duration::from_secs(delay_secs)
}
