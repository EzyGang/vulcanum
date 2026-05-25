use crate::client::ApiClient;
use crate::state::{save_state, WorkerState};

pub async fn run(code: Option<String>, instance: Option<String>) -> anyhow::Result<()> {
    let instance = match instance {
        Some(url) => url,
        None => prompt_instance_url()?,
    };

    let code = match code {
        Some(c) => c,
        None => prompt_code()?,
    };

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
        expires_at: resp.expires_at,
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

    Ok(())
}

fn prompt_instance_url() -> anyhow::Result<String> {
    let url = dialoguer::Input::<String>::new()
        .with_prompt("Instance URL")
        .validate_with(|input: &String| {
            if input.trim().is_empty() {
                Err("URL is required".to_owned())
            } else {
                Ok(())
            }
        })
        .interact_text()?;
    Ok(url.trim().to_owned())
}

fn prompt_code() -> anyhow::Result<String> {
    let code = dialoguer::Input::<String>::new()
        .with_prompt("Connection code")
        .validate_with(|input: &String| {
            if input.trim().is_empty() {
                Err("Code is required".to_owned())
            } else {
                Ok(())
            }
        })
        .interact_text()?;
    Ok(code.trim().to_owned())
}
