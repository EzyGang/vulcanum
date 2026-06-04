pub(crate) mod task;

use std::sync::Arc;

use tokio::sync::RwLock;

use vulcanum_shared::client::ApiClient;
use vulcanum_shared::worker_state::WorkerState;

use crate::opencode;
use crate::opencode::session;
use crate::recovery::task::{mark_lost_and_submit, recover_session_task};
use crate::runtime::launch::read_container_port;
use crate::session::remove_container;
use crate::state::journal::{Journal, JournalEntry};

pub async fn reconcile_running_jobs(
    journal: &Arc<Journal>,
    client: &Arc<ApiClient>,
    worker_state: &Arc<RwLock<WorkerState>>,
) {
    let running = match journal.list_running() {
        Ok(entries) => entries,
        Err(e) => {
            tracing::error!(error = %e, "failed to list running jobs for recovery");
            return;
        }
    };

    if running.is_empty() {
        return;
    }

    tracing::info!(count = running.len(), "reconciling stale running jobs");

    for entry in &running {
        let alive = check_container_alive(entry);

        if !alive {
            mark_lost_and_submit(journal, client, worker_state, entry).await;
            continue;
        }

        let Some(container_name) = entry.container_name.as_deref() else {
            mark_lost_and_submit(journal, client, worker_state, entry).await;
            continue;
        };

        let port = match read_container_port(container_name).await {
            Ok(p) => p,
            Err(e) => {
                tracing::warn!(
                    job_id = %entry.job_id,
                    container_name = container_name,
                    error = %e,
                    "failed to read container port"
                );
                remove_container(Some(container_name));
                mark_lost_and_submit(journal, client, worker_state, entry).await;
                continue;
            }
        };

        let base_url = format!("http://127.0.0.1:{port}");
        let oc_client = opencode::OpenCodeClient::new(&base_url);

        let status_map = match session::get_session_status(&oc_client).await {
            Ok(map) => map,
            Err(e) => {
                tracing::warn!(
                    job_id = %entry.job_id,
                    error = %e,
                    "failed to query session status"
                );
                remove_container(Some(container_name));
                mark_lost_and_submit(journal, client, worker_state, entry).await;
                continue;
            }
        };

        let session_id = match entry.session_id.as_deref() {
            Some(sid) => sid,
            None => {
                tracing::warn!(
                    job_id = %entry.job_id,
                    "no session_id in journal"
                );
                remove_container(Some(container_name));
                mark_lost_and_submit(journal, client, worker_state, entry).await;
                continue;
            }
        };

        let status = match status_map.get(session_id) {
            Some(s) => s,
            None => {
                tracing::warn!(
                    job_id = %entry.job_id,
                    session_id = session_id,
                    "session not found in status map"
                );
                remove_container(Some(container_name));
                mark_lost_and_submit(journal, client, worker_state, entry).await;
                continue;
            }
        };

        match status {
            session::OpenCodeSessionStatus::Idle
            | session::OpenCodeSessionStatus::Busy
            | session::OpenCodeSessionStatus::Retry { .. } => {
                tracing::info!(
                    job_id = %entry.job_id,
                    session_id = session_id,
                    "reconnecting to live session"
                );
                let task_entry = entry.clone();
                let api_client = Arc::clone(client);
                let worker = Arc::clone(worker_state);
                let jrnl = Arc::clone(journal);
                let sid = session_id.to_owned();
                let cname = container_name.to_owned();
                tokio::spawn(recover_session_task(
                    task_entry, api_client, worker, jrnl, oc_client, sid, cname,
                ));
            }
        }
    }
}

fn check_container_alive(entry: &JournalEntry) -> bool {
    let Some(name) = &entry.container_name else {
        return false;
    };

    let output = std::process::Command::new("docker")
        .args(["inspect", "--format", "{{.State.Running}}", name])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .output();

    match output {
        Ok(out) => String::from_utf8_lossy(&out.stdout).trim() == "true",
        Err(_) => false,
    }
}
