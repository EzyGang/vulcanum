use std::sync::Arc;

use tokio::sync::RwLock;

use vulcanum_shared::api_types::WorkRunType;
use vulcanum_shared::client::ApiClient;
use vulcanum_shared::runtime::agent::RunningSession;
use vulcanum_shared::runtime::isolation::IsolationProvider;
use vulcanum_shared::runtime::types::{IsolatedEnvironment, ResourceLimits};
use vulcanum_shared::worker_state::WorkerState;

use crate::daemon::auth::with_retry_on_401;
use crate::daemon::job::execution::submit::{submit_result_request, SubmitResultParams};
use crate::daemon::job::turn_loop::{run_turn_loop, TurnLoopCtx};
use crate::isolation::providers::host::HostIsolation;
use crate::providers::opencode::events;
use crate::providers::opencode::runner::OpenCodeRunningSession;
use crate::providers::opencode::runner::SessionConfig;
use crate::providers::opencode::OpenCodeClient;
use crate::recovery::cleanup::kill_host_process_group;
use crate::recovery::cleanup::remove_container;
use crate::state::journal::{Journal, JournalEntry, JournalResultUpdate, JournalStatus};

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
    let work_type = match with_retry_on_401(&api_client, &worker_state, |token| {
        let client = api_client.clone();
        let job_id = entry.job_id;
        async move { client.get_job(job_id, &token).await }
    })
    .await
    {
        Ok(job) => job.work_type,
        Err(e) => {
            tracing::warn!(
                job_id = %entry.job_id,
                error = %e,
                "failed to load job type during recovery, using implementation turn loop"
            );
            WorkRunType::Implementation
        }
    };

    let running_session = OpenCodeRunningSession::new(SessionConfig {
        client: oc_client,
        session_id: session_id.clone(),
        event_stream,
        max_duration_secs: ResourceLimits::default().max_duration_secs,
        container_name,
        server_process: None,
        host_pid: entry.host_pid.map(|v| v as u32),
        host_port: entry.host_port.map(|v| v as u16),
    });

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
    let reporter = Arc::new(
        crate::daemon::job::execution::event_reporter::EventReporter::new(
            api_client.clone(),
            worker_state.clone(),
            entry.job_id,
        ),
    );
    reporter.emit(
        "session.recovered",
        serde_json::json!({"initial_turn": initial_turn}),
    );
    let ctx = TurnLoopCtx {
        client: api_client.clone(),
        worker_state: worker_state.clone(),
        journal: journal.clone(),
        job_id: entry.job_id,
        worker_id: uuid::Uuid::nil(),
        reporter,
    };
    run_turn_loop(
        &mut boxed,
        &artifact_path,
        work_type,
        max_turns,
        initial_turn,
        &ctx,
    )
    .await;

    cleanup_recovery(&entry);
    tracing::info!(job_id = %entry.job_id, "recovery session completed");
}

fn cleanup_recovery(entry: &JournalEntry) {
    if entry.harness_type == "host" {
        kill_host_process_group(entry);
        let provider = HostIsolation::new();
        let env = IsolatedEnvironment {
            workdir: std::path::PathBuf::from(&entry.workdir),
            workspace_dir: std::path::PathBuf::from(&entry.workdir).join("workspace"),
            repos: Vec::new(),
            container_name: entry.container_name.clone(),
            secrets: std::collections::HashMap::new(),
            env_vars: std::collections::HashMap::new(),
            runtime: None,
            image: None,
            server_host_port: None,
            limits: ResourceLimits::default(),
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
    let _ = journal.update_result(JournalResultUpdate {
        job_id: entry.job_id,
        exit_code: 1,
        tokens_used: 0,
        input_tokens: 0,
        output_tokens: 0,
        cache_read_tokens: 0,
        cache_write_tokens: 0,
        pr_url: None,
        duration_ms: 0,
        status: JournalStatus::Lost,
    });

    let result = submit_result_request(SubmitResultParams {
        pr_urls: Vec::new(),
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
        review_url: None,
        review_body: None,
        review_already_exists: false,
    });

    if let Err(e) = with_retry_on_401(client, worker_state, |token| {
        let client = client.clone();
        let result = result.clone();
        async move { client.submit_result(entry.job_id, &result, &token).await }
    })
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
