use crate::commands::setup::host;
use crate::commands::setup::prompts::{prompt_code, prompt_instance_url};
use crate::commands::setup::service;
use crate::console;

pub async fn verify_connection() -> anyhow::Result<()> {
    use vulcanum_shared::client::ApiClient;
    use vulcanum_shared::worker_state;

    let state =
        worker_state::load_state()?.ok_or_else(|| anyhow::anyhow!("no worker state found"))?;
    let client = ApiClient::new(&state.instance_url);
    client.status().await?;
    Ok(())
}

pub async fn connect_worker(code: Option<String>, instance: Option<String>) -> anyhow::Result<()> {
    use vulcanum_shared::api_types::WorkerCapabilities;
    use vulcanum_shared::client::{probe_url_with_scheme_fallback, ApiClient};
    use vulcanum_shared::config::load_config;
    use vulcanum_shared::worker_state::{save_state, WorkerState};

    let raw_instance = match instance {
        Some(url) => url,
        None => prompt_instance_url()?,
    };

    console::info("Probing instance URL...");
    let (resolved_url, _) = probe_url_with_scheme_fallback(&raw_instance).await?;
    let trimmed_instance = raw_instance.trim().trim_end_matches('/');
    if resolved_url != trimmed_instance {
        console::info(&format!("Using {} (scheme fallback)", resolved_url));
    }

    let code = match code {
        Some(c) => c,
        None => prompt_code()?,
    };

    let worker_name = hostname::get()
        .ok()
        .and_then(|h| h.to_str().map(|s| s.to_owned()))
        .unwrap_or_else(|| "unnamed-worker".to_owned());

    let config = load_config()?;
    let isolation_backend = config.isolation_backend()?;
    let capabilities = WorkerCapabilities {
        agent_backends: vec![config.agent_backend],
        isolation_backends: vec![isolation_backend.as_str().to_owned()],
    };
    let client = ApiClient::new(&resolved_url);
    let max_concurrent_jobs = host::calculate_worker_capacity();

    let resp = client
        .connect(&code, &worker_name, Some(max_concurrent_jobs), capabilities)
        .await?;

    let state = WorkerState {
        worker_id: resp.worker_id,
        instance_url: resolved_url,
        access_token: resp.access_token,
        refresh_token: resp.refresh_token,
        expires_at: resp.expires_at,
        max_concurrent_jobs: resp.max_concurrent_jobs,
    };

    save_state(&state)?;

    tracing::info!(
        worker_id = resp.worker_id.to_string().as_str(),
        worker_name = resp.name.as_str(),
        "connected as worker '{}' (id: {}, token expires: {})",
        resp.name,
        resp.worker_id,
        resp.expires_at
    );

    if service::is_worker_service_installed() {
        tracing::info!("restarting worker service after connect");
        service::enable_and_restart_worker_service()?;
    }

    Ok(())
}
