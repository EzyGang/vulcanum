use std::path::Path;

use vulcanum_shared::client::ApiClient;
use vulcanum_shared::state::paths;
use vulcanum_shared::state::worker::{self as worker_state, WorkerState};
use vulcanum_shared::token::ensure_valid_token;

use crate::commands::setup::service;
use crate::console;
#[cfg(test)]
mod tests;

pub async fn run() -> anyhow::Result<()> {
    console::info("Preparing worker self-delete...");

    match worker_state::load_state() {
        Ok(Some(mut state)) => {
            deregister_worker(&mut state).await;
        }
        Ok(None) => {
            console::warn("No worker state found; skipping server deregistration.");
        }
        Err(err) => {
            tracing::warn!(error = %err, "failed to load worker state");
            console::warn(&format!(
                "Failed to load worker state ({err:#}); skipping server deregistration."
            ));
        }
    }

    cleanup_local_environment();
    Ok(())
}

async fn deregister_worker(state: &mut WorkerState) {
    let client = ApiClient::new(state.instance_url.clone());

    if let Err(err) = ensure_valid_token(&client, state, 0).await {
        tracing::warn!(error = %err, "worker access token refresh failed before self-delete");
        console::warn(&format!("Token refresh failed before self-delete ({err:#}); continuing with the existing token."));
    }

    match client.delete_worker_self(&state.access_token).await {
        Ok(()) => {
            console::info("Worker deregistered from server.");
        }
        Err(err) => {
            tracing::warn!(error = %err, "worker self-delete request failed");
            console::warn(&format!(
                "Worker deregistration request failed ({err:#}); continuing with local cleanup."
            ));
        }
    }
}

fn cleanup_local_environment() {
    service::remove_worker_service_best_effort();

    match paths::vulcanum_dir() {
        Ok(dir) => remove_directory_best_effort(&dir),
        Err(err) => {
            tracing::warn!(error = %err, "failed to resolve vulcanum directory");
            console::warn(&format!(
                "Failed to resolve local worker directory ({err:#}); skipping file cleanup."
            ));
        }
    }
}

pub(crate) fn remove_directory_best_effort(path: &Path) {
    if !path.exists() {
        return;
    }

    if let Err(err) = std::fs::remove_dir_all(path) {
        tracing::warn!(error = %err, path = %path.display(), "failed to remove local worker directory");
        console::warn(&format!("Failed to remove {} ({err:#})", path.display()));
    } else {
        console::info(&format!("Removed {}.", path.display()));
    }
}
