use std::future::Future;
use std::sync::{Arc, OnceLock};

use anyhow::Context;
use chrono::Utc;
use tokio::sync::{Mutex, RwLock};

use vulcanum_shared::api::error::ApiError;
use vulcanum_shared::client::ApiClient;
use vulcanum_shared::state::worker::{save_state, WorkerState};

static REFRESH_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

pub(crate) async fn ensure_token_valid(
    client: &Arc<ApiClient>,
    worker_state: &Arc<RwLock<WorkerState>>,
    refresh_buffer_secs: i64,
) -> anyhow::Result<()> {
    let threshold = Utc::now()
        + chrono::Duration::try_seconds(refresh_buffer_secs)
            .context("invalid refresh buffer duration")?;

    if worker_state.read().await.expires_at > threshold {
        return Ok(());
    }

    let _refresh_guard = refresh_lock().lock().await;
    let refresh_token = {
        let state = worker_state.read().await;
        if state.expires_at > threshold {
            return Ok(());
        }
        state.refresh_token.clone()
    };

    let resp = client.refresh(&refresh_token).await?;
    let state = update_worker_state(worker_state, resp).await?;
    tracing::debug!(expires_at = %state.expires_at, "worker access token refreshed before expiry");

    Ok(())
}

pub(crate) async fn with_retry_on_401<T, F, Fut>(
    client: &Arc<ApiClient>,
    worker_state: &Arc<RwLock<WorkerState>>,
    mut request: F,
) -> anyhow::Result<T>
where
    F: FnMut(String) -> Fut,
    Fut: Future<Output = anyhow::Result<T>>,
{
    let access_token = worker_state.read().await.access_token.clone();
    match request(access_token.clone()).await {
        Ok(result) => Ok(result),
        Err(e) if is_unauthorized(&e) => {
            let access_token = refresh_access_token(client, worker_state, &access_token).await?;
            request(access_token).await
        }
        Err(e) => Err(e),
    }
}

async fn refresh_access_token(
    client: &Arc<ApiClient>,
    worker_state: &Arc<RwLock<WorkerState>>,
    failed_access_token: &str,
) -> anyhow::Result<String> {
    let _refresh_guard = refresh_lock().lock().await;
    let refresh_token = {
        let state = worker_state.read().await;
        if state.access_token != failed_access_token {
            return Ok(state.access_token.clone());
        }
        state.refresh_token.clone()
    };

    let resp = client.refresh(&refresh_token).await?;
    let state = update_worker_state(worker_state, resp).await?;
    tracing::debug!(expires_at = %state.expires_at, "worker access token refreshed after 401");
    Ok(state.access_token)
}

async fn update_worker_state(
    worker_state: &Arc<RwLock<WorkerState>>,
    resp: vulcanum_shared::api::wire::RefreshResponse,
) -> anyhow::Result<WorkerState> {
    let state = {
        let mut state = worker_state.write().await;
        state.access_token = resp.access_token;
        state.refresh_token = resp.refresh_token;
        state.expires_at = resp.expires_at;
        state.clone()
    };
    save_state(&state)?;
    Ok(state)
}

fn refresh_lock() -> &'static Mutex<()> {
    REFRESH_LOCK.get_or_init(|| Mutex::new(()))
}

fn is_unauthorized(error: &anyhow::Error) -> bool {
    error
        .downcast_ref::<ApiError>()
        .is_some_and(|api_error| api_error.status == 401)
}
