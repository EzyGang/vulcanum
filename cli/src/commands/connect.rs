use crate::client::ApiClient;
use crate::state::{save_state, WorkerState};

pub async fn run(code: String, instance: String) -> anyhow::Result<()> {
    let worker_name = hostname::get()
        .ok()
        .and_then(|h| h.to_str().map(|s| s.to_owned()))
        .unwrap_or_else(|| "unnamed-worker".to_owned());

    let client = ApiClient::new(instance.clone());

    let resp = client.connect(&code, &worker_name).await?;

    let state = WorkerState {
        worker_id: resp.worker_id,
        instance_url: instance,
        access_token: resp.access_token,
        refresh_token: resp.refresh_token,
        expires_at: chrono::Utc::now() + chrono::Duration::minutes(15),
    };

    save_state(&state)?;

    tracing::info!(
        "connected as worker '{}' (id: {})",
        resp.name,
        resp.worker_id
    );

    Ok(())
}
