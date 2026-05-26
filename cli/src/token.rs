use chrono::Utc;

use crate::client::ApiClient;
use crate::state::WorkerState;

pub async fn ensure_valid_token(
    client: &ApiClient,
    state: &mut WorkerState,
    refresh_buffer_secs: i64,
) -> anyhow::Result<()> {
    let threshold = Utc::now() + chrono::Duration::seconds(refresh_buffer_secs);

    if state.expires_at <= threshold {
        let resp = client.refresh(&state.refresh_token).await?;
        state.access_token = resp.access_token;
        state.refresh_token = resp.refresh_token;
        state.expires_at = resp.expires_at;
        crate::state::save_state(state)?;
        tracing::debug!("token refreshed, new expiry: {}", state.expires_at);
    }

    Ok(())
}
