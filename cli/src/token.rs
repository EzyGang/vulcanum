use chrono::Utc;

use crate::client::ApiClient;
use crate::state::WorkerState;

const REFRESH_BUFFER_SECS: i64 = 60;

/// Refreshes the access token if it expires within the buffer window.
/// Returns the updated state (saved to disk) or an error.
pub async fn ensure_valid_token(client: &ApiClient, state: &mut WorkerState) -> anyhow::Result<()> {
    let now = Utc::now();
    let threshold = now + chrono::Duration::seconds(REFRESH_BUFFER_SECS);

    if state.expires_at <= threshold {
        let resp = client.refresh(&state.refresh_token).await?;
        state.access_token = resp.access_token;
        state.refresh_token = resp.refresh_token;
        state.expires_at = resp.expires_at;
        crate::state::save_state(state)?;
        tracing::info!("token refreshed, new expiry: {}", state.expires_at);
    }

    Ok(())
}
