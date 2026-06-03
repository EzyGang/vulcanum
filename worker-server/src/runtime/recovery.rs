use std::sync::Arc;

use tokio::sync::RwLock;

use vulcanum_shared::api_types::SubmitResultRequest;
use vulcanum_shared::client::ApiClient;
use vulcanum_shared::runtime::agent::RunningSession;
use vulcanum_shared::worker_state::WorkerState;

use crate::daemon::job::execution::{run_turn_loop, TurnLoopCtx};
use crate::runtime::client::events;
use crate::runtime::client::session;
use crate::runtime::client::OpenCodeClient;
use crate::runtime::runner::{remove_container, OpenCodeRunningSession, SessionConfig};
use crate::runtime::serve::launch::read_container_port;
use crate::state::journal::{Journal, JournalEntry, JournalStatus};

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
        let oc_client = OpenCodeClient::new(&base_url);

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

async fn recover_session_task(
    entry: JournalEntry,
    api_client: Arc<ApiClient>,
    worker_state: Arc<RwLock<WorkerState>>,
    journal: Arc<Journal>,
    oc_client: OpenCodeClient,
    session_id: String,
    container_name: String,
) {
    let event_stream = match events::connect_events(&oc_client).await {
        Ok(s) => s,
        Err(e) => {
            tracing::error!(
                job_id = %entry.job_id,
                error = %e,
                "failed to reconnect event stream during recovery"
            );
            remove_container(Some(&container_name));
            mark_lost_and_submit(&journal, &api_client, &worker_state, &entry).await;
            return;
        }
    };

    let max_turns = entry.max_turns.unwrap_or(1).max(1);
    let current_turn = entry.turn_count.unwrap_or(0);
    let initial_turn = current_turn + 1;

    let mut running_session = OpenCodeRunningSession::new(SessionConfig {
        client: oc_client,
        session_id: session_id.clone(),
        event_stream,
        max_duration_secs: 1800,
        is_container: true,
        container_name: Some(container_name.clone()),
        server_process: None,
    });

    let access_token = worker_state.read().await.access_token.clone();
    running_session.set_event_reporter(api_client.clone(), access_token, entry.job_id);

    let workdir = std::path::Path::new(&entry.workdir);
    let artifact_path = workdir.join("home").join("finish_artifact.json");

    tracing::info!(
        job_id = %entry.job_id,
        session_id = session_id,
        initial_turn = initial_turn,
        max_turns = max_turns,
        "reconnected session, resuming turn loop"
    );

    let mut boxed: Box<dyn RunningSession> = Box::new(running_session);
    let ctx = TurnLoopCtx {
        client: api_client.clone(),
        worker_state: worker_state.clone(),
        journal: journal.clone(),
        job_id: entry.job_id,
        worker_id: uuid::Uuid::nil(),
    };
    run_turn_loop(&mut boxed, &artifact_path, max_turns, initial_turn, &ctx).await;

    remove_container(Some(&container_name));
    tracing::info!(job_id = %entry.job_id, "recovery session completed");
}

async fn mark_lost_and_submit(
    journal: &Arc<Journal>,
    client: &Arc<ApiClient>,
    worker_state: &Arc<RwLock<WorkerState>>,
    entry: &JournalEntry,
) {
    let _ = journal.update_result(entry.job_id, 1, 0, None, 0, JournalStatus::Lost);

    let result = SubmitResultRequest {
        pr_url: String::new(),
        exit_code: 1,
        tokens_used: 0,
        duration_ms: 0,
        input_tokens: 0,
        output_tokens: 0,
        cache_read_tokens: 0,
        cache_write_tokens: 0,
        model_used: None,
        finish_status: None,
        finish_summary: None,
        finish_blocked_reason: None,
        finish_next_column: None,
    };

    let access_token = worker_state.read().await.access_token.clone();
    if let Err(e) = client
        .submit_result(entry.job_id, &result, &access_token)
        .await
    {
        tracing::warn!(
            job_id = %entry.job_id,
            error = %e,
            "failed to submit lost result for stale job"
        );
    }

    let _ = journal.mark_submitted(entry.job_id);
    tracing::info!(job_id = %entry.job_id, "stale job marked as lost and submitted");
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
