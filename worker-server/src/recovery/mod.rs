pub(crate) mod checks;
pub(crate) mod cleanup;
pub(crate) mod recover_session;

use std::sync::Arc;

use tokio::sync::RwLock;

use vulcanum_shared::client::ApiClient;
use vulcanum_shared::worker_state::WorkerState;

use crate::providers::opencode;
use crate::providers::opencode::api;
use crate::providers::opencode::spawn::read_container_port;
use crate::recovery::checks::{check_container_alive, check_host_alive};
use crate::recovery::cleanup::cleanup_stale_job;
use crate::recovery::recover_session::{mark_lost_and_submit, recover_session_task};
use crate::state::journal::Journal;

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
        let is_host = entry.harness_type == "host";
        let alive = if is_host {
            check_host_alive(entry)
        } else {
            check_container_alive(entry)
        };

        if !alive {
            cleanup_stale_job(entry);
            mark_lost_and_submit(journal, client, worker_state, entry).await;
            continue;
        }

        let port = if is_host {
            match entry.host_port {
                Some(p) => p as u16,
                None => {
                    tracing::warn!(
                        job_id = %entry.job_id,
                        "no host_port in journal, killing orphan"
                    );
                    cleanup_stale_job(entry);
                    mark_lost_and_submit(journal, client, worker_state, entry).await;
                    continue;
                }
            }
        } else {
            let Some(container_name) = entry.container_name.as_deref() else {
                mark_lost_and_submit(journal, client, worker_state, entry).await;
                continue;
            };
            match read_container_port(container_name).await {
                Ok(p) => p,
                Err(e) => {
                    tracing::warn!(
                        job_id = %entry.job_id,
                        container_name = container_name,
                        error = %e,
                        "failed to read container port"
                    );
                    crate::providers::opencode::cleanup::remove_container(Some(container_name));
                    mark_lost_and_submit(journal, client, worker_state, entry).await;
                    continue;
                }
            }
        };

        let base_url = format!("http://127.0.0.1:{port}");
        let oc_client = opencode::OpenCodeClient::new(&base_url);

        let status_map = match api::get_session_status(&oc_client).await {
            Ok(map) => map,
            Err(e) => {
                tracing::warn!(
                    job_id = %entry.job_id,
                    error = %e,
                    "failed to query session status"
                );
                cleanup_stale_job(entry);
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
                cleanup_stale_job(entry);
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
                cleanup_stale_job(entry);
                mark_lost_and_submit(journal, client, worker_state, entry).await;
                continue;
            }
        };

        match status {
            api::OpenCodeSessionStatus::Idle
            | api::OpenCodeSessionStatus::Busy
            | api::OpenCodeSessionStatus::Retry { .. } => {
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
                let cname = entry.container_name.clone();
                tokio::spawn(recover_session_task(
                    task_entry, api_client, worker, jrnl, oc_client, sid, cname,
                ));
            }
        }
    }
}
