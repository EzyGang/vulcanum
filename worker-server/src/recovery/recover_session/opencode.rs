use std::sync::Arc;

use tokio::sync::RwLock;

use vulcanum_shared::api_types::WorkRunType;
use vulcanum_shared::client::ApiClient;
use vulcanum_shared::runtime::agent::RunningSession;
use vulcanum_shared::runtime::types::ResourceLimits;
use vulcanum_shared::worker_state::WorkerState;

use crate::daemon::auth::with_retry_on_401;
use crate::daemon::job::github_credentials::{
    setup_recovered_credentials, spawn_refresh_task, stop_refresh_task,
};
use crate::daemon::job::turn_loop::{run_turn_loop, TurnLoopCtx};
use crate::providers::opencode::events;
use crate::providers::opencode::runner::{OpenCodeRunningSession, SessionConfig};
use crate::providers::opencode::OpenCodeClient;
use crate::recovery::recover_session::common::{
    cleanup_recovery, mark_lost_and_submit, save_recovered_messages,
};
use crate::state::journal::{Journal, JournalEntry};

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
    let recovered_job = match with_retry_on_401(&api_client, &worker_state, |token| {
        let client = api_client.clone();
        let job_id = entry.job_id;
        async move { client.get_job(job_id, &token).await }
    })
    .await
    {
        Ok(job) => Some(job),
        Err(e) => {
            tracing::warn!(
                job_id = %entry.job_id,
                error = %e,
                "failed to load job during recovery, using implementation turn loop without github credential refresh"
            );
            None
        }
    };
    let work_type = recovered_job
        .as_ref()
        .map_or(WorkRunType::Implementation, |job| job.work_type);

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
    if let Some(job) = recovered_job.as_ref() {
        if let Err(e) =
            setup_recovered_credentials(workdir, &entry.harness_type, job.github_token.as_deref())
                .await
        {
            tracing::warn!(
                job_id = %entry.job_id,
                error = %e,
                "failed to restore github credential bridge during recovery"
            );
        }
    }

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
    reporter
        .emit(
            "session.recovered",
            serde_json::json!({"initial_turn": initial_turn}),
        )
        .await;
    let ctx = TurnLoopCtx {
        client: api_client.clone(),
        worker_state: worker_state.clone(),
        journal: journal.clone(),
        job_id: entry.job_id,
        worker_id: uuid::Uuid::nil(),
        reporter,
    };
    let github_refresh_stop = recovered_job.as_ref().and_then(|job| {
        job.github_token.as_ref().map(|_| {
            spawn_refresh_task(
                api_client.clone(),
                worker_state.clone(),
                entry.job_id,
                std::path::PathBuf::from(&entry.workdir),
                job.github_token_expires_at,
            )
        })
    });
    run_turn_loop(
        &mut boxed,
        &artifact_path,
        work_type,
        max_turns,
        initial_turn,
        &ctx,
    )
    .await;
    stop_refresh_task(github_refresh_stop);
    if let Some(session_id) = boxed.session_id().map(str::to_owned) {
        match boxed.export_messages().await {
            Ok(Some(messages)) => save_recovered_messages(entry.job_id, &session_id, &messages),
            Ok(None) => (),
            Err(e) => {
                tracing::warn!(
                    work_run_id = %entry.job_id,
                    error = %e,
                    "failed to export recovered session messages"
                );
            }
        }
    }

    cleanup_recovery(&entry);
    tracing::info!(job_id = %entry.job_id, "recovery session completed");
}
