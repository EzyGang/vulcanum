use std::sync::Arc;

use tokio::sync::RwLock;

use vulcanum_shared::api_types::SubmitResultRequest;
use vulcanum_shared::client::ApiClient;
use vulcanum_shared::runtime::agent::RunningSession;
use vulcanum_shared::runtime::isolation::IsolationProvider;
use vulcanum_shared::worker_state::WorkerState;

use crate::daemon::job::turn_loop::{run_turn_loop, TurnLoopCtx};
use crate::isolation::providers::host::HostIsolation;
use crate::providers::opencode::events;
use crate::providers::opencode::runner::OpenCodeRunningSession;
use crate::providers::opencode::runner::SessionConfig;
use crate::providers::opencode::OpenCodeClient;
use crate::recovery::cleanup::kill_host_process_group;
use crate::recovery::cleanup::remove_container;
use crate::state::journal::{Journal, JournalEntry, JournalStatus};

pub(crate) async fn recover_session_task(
    entry: JournalEntry,
    api_client: Arc<ApiClient>,
    worker_state: Arc<RwLock<WorkerState>>,
    journal: Arc<Journal>,
    oc_client: OpenCodeClient,
    session_id: String,
    container_name: Option<String>,
) {
    let event_stream = match events::connect_events(&oc_client).await {
        Ok(s) => s,
        Err(e) => {
            tracing::error!(
                job_id = %entry.job_id,
                error = %e,
                "failed to reconnect event stream during recovery"
            );
            cleanup_recovery(&entry);
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
        container_name,
        server_process: None,
        host_pid: entry.host_pid.map(|v| v as u32),
        host_port: entry.host_port.map(|v| v as u16),
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

    cleanup_recovery(&entry);
    tracing::info!(job_id = %entry.job_id, "recovery session completed");
}

fn cleanup_recovery(entry: &JournalEntry) {
    if entry.harness_type == "host" {
        kill_host_process_group(entry);
        let provider = HostIsolation::new();
        let env = vulcanum_shared::runtime::types::IsolatedEnvironment {
            workdir: std::path::PathBuf::from(&entry.workdir),
            container_name: entry.container_name.clone(),
            secrets: std::collections::HashMap::new(),
            env_vars: std::collections::HashMap::new(),
            runtime: None,
            image: None,
            server_host_port: None,
            limits: vulcanum_shared::runtime::types::ResourceLimits::default(),
        };
        tokio::spawn(async move {
            provider.cleanup(&env).await;
        });
    } else if let Some(ref name) = entry.container_name {
        remove_container(Some(name));
    }
}

pub(crate) async fn mark_lost_and_submit(
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
