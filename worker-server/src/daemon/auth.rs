use std::future::Future;
use std::sync::Arc;

use tokio::sync::RwLock;

use vulcanum_shared::api_error::ApiError;
use vulcanum_shared::client::ApiClient;
use vulcanum_shared::worker_state::{save_state, WorkerState};

pub(crate) async fn with_fresh_token<T, F, Fut>(
    client: &Arc<ApiClient>,
    worker_state: &Arc<RwLock<WorkerState>>,
    mut request: F,
) -> anyhow::Result<T>
where
    F: FnMut(String) -> Fut,
    Fut: Future<Output = anyhow::Result<T>>,
{
    let access_token = worker_state.read().await.access_token.clone();
    match request(access_token).await {
        Ok(result) => Ok(result),
        Err(e) if is_unauthorized(&e) => {
            let access_token = refresh_access_token(client, worker_state).await?;
            request(access_token).await
        }
        Err(e) => Err(e),
    }
}

async fn refresh_access_token(
    client: &Arc<ApiClient>,
    worker_state: &Arc<RwLock<WorkerState>>,
) -> anyhow::Result<String> {
    let mut state = worker_state.write().await;
    let resp = client.refresh(&state.refresh_token).await?;
    state.access_token = resp.access_token;
    state.refresh_token = resp.refresh_token;
    state.expires_at = resp.expires_at;
    save_state(&state)?;
    tracing::debug!(expires_at = %state.expires_at, "worker access token refreshed after 401");
    Ok(state.access_token.clone())
}

fn is_unauthorized(error: &anyhow::Error) -> bool {
    error
        .downcast_ref::<ApiError>()
        .is_some_and(|api_error| api_error.status == 401)
}
