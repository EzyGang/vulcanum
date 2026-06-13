use std::path::Path;

use vulcanum_shared::client::ApiClient;
use vulcanum_shared::paths;
use vulcanum_shared::token::ensure_valid_token;
use vulcanum_shared::worker_state::{self, WorkerState};

use crate::commands::setup::systemd;
use crate::console;

#[cfg(test)]
#[path = "self_delete_tests.rs"]
mod self_delete_tests;

pub async fn run() -> anyhow::Result<()> {
    console::info("Preparing worker self-delete...");

    match worker_state::load_state() {
        Ok(Some(mut state)) => {
            if let Err(err) = deregister_worker(&mut state).await {
                tracing::warn!(error = %err, "worker deregistration failed");
                console::warn(&format!(
                    "Failed to deregister worker from server ({err:#})"
                ));
            }
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

async fn deregister_worker(state: &mut WorkerState) -> anyhow::Result<()> {
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

    Ok(())
}

fn cleanup_local_environment() {
    systemd::remove_worker_service_best_effort();

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
