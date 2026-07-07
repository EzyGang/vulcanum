use std::sync::Arc;

use tokio::sync::RwLock;

use vulcanum_shared::api_types::JobResponse;
use vulcanum_shared::client::ApiClient;
use vulcanum_shared::constants::DEFAULT_IMAGE;
use vulcanum_shared::runtime::errors::HarnessError;
use vulcanum_shared::runtime::types::{IsolatedEnvironment, ResourceLimits};
use vulcanum_shared::worker_state::WorkerState;

use crate::daemon::auth::with_retry_on_401;
use crate::daemon::job::github_credentials::{
    setup_recovered_credentials, spawn_refresh_task, stop_refresh_task,
};
use crate::daemon::job::runtime_secrets::job_runtime_secrets;
use crate::daemon::job::turn_loop::{run_turn_loop, TurnLoopCtx};
use crate::isolation::github_credentials as isolation_github_credentials;
use crate::isolation::workspace;
use crate::providers::omp_rpc::runtime::OmpRpcRuntime;
use crate::recovery::cleanup::cleanup_stale_job;
use crate::recovery::recover_session::common::{
    cleanup_recovery, mark_lost_and_submit, recovery_continuation_prompt, save_recovered_messages,
};
use crate::state::journal::{Journal, JournalEntry};

pub(crate) async fn recovered_omp_env(
    entry: &JournalEntry,
    job: &JobResponse,
) -> Result<IsolatedEnvironment, HarnessError> {
    let workdir = std::path::PathBuf::from(&entry.workdir);
    let secrets = job_runtime_secrets(job);
    let sanitized_secrets = isolation_github_credentials::without_direct_token_env(&secrets);
    let github_credentials =
        setup_recovered_credentials(&workdir, &entry.harness_type, job.github_token.as_deref())
            .await?;
    let runtime_home = match entry.harness_type.as_str() {
        "docker" | "kata" => "/workdir/home".to_owned(),
        _ => workdir.join("home").to_string_lossy().to_string(),
    };
    let runtime_tmpdir = match entry.harness_type.as_str() {
        "docker" | "kata" => "/workdir/tmp".to_owned(),
        _ => workdir.join("tmp").to_string_lossy().to_string(),
    };
    let finish_artifact = match entry.harness_type.as_str() {
        "docker" | "kata" => "/workdir/home/finish_artifact.json".to_owned(),
        _ => workdir
            .join("home")
            .join("finish_artifact.json")
            .to_string_lossy()
            .to_string(),
    };
    let mut env_vars = sanitized_secrets.clone();
    env_vars.extend(workspace::omp_environment_vars(
        &runtime_home,
        &runtime_tmpdir,
    ));
    env_vars.insert("FINISH_ARTIFACT_PATH".to_owned(), finish_artifact);
    env_vars.extend(match entry.harness_type.as_str() {
        "docker" | "kata" => github_credentials.runtime_env,
        _ => github_credentials.host_env,
    });

    Ok(IsolatedEnvironment {
        workdir: workdir.clone(),
        workspace_dir: workdir.join("workspace"),
        repos: Vec::new(),
        container_name: entry.container_name.clone(),
        secrets: sanitized_secrets,
        env_vars,
        runtime: (entry.harness_type == "kata").then_some("kata-runtime"),
        image: Some(DEFAULT_IMAGE.to_owned()),
        server_host_port: None,
        limits: ResourceLimits::default(),
    })
}

pub(crate) async fn recover_omp_rpc_session_task(
    entry: JournalEntry,
    api_client: Arc<ApiClient>,
    worker_state: Arc<RwLock<WorkerState>>,
    journal: Arc<Journal>,
) {
    let Some(session_path) = entry.agent_session_path.as_deref() else {
        cleanup_recovery(&entry);
        mark_lost_and_submit(&journal, &api_client, &worker_state, &entry).await;
        return;
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
        Ok(job) => job,
        Err(e) => {
            tracing::warn!(
                job_id = %entry.job_id,
                error = %e,
                "failed to load OMP job during recovery"
            );
            cleanup_recovery(&entry);
            mark_lost_and_submit(&journal, &api_client, &worker_state, &entry).await;
            return;
        }
    };
    let work_type = recovered_job.work_type;
    let workdir = std::path::PathBuf::from(&entry.workdir);

    cleanup_stale_job(&entry);
    let env = match recovered_omp_env(&entry, &recovered_job).await {
        Ok(env) => env,
        Err(e) => {
            tracing::error!(
                job_id = %entry.job_id,
                error = %e,
                "failed to restore OMP recovery environment"
            );
            mark_lost_and_submit(&journal, &api_client, &worker_state, &entry).await;
            return;
        }
    };
    let github_refresh_stop = recovered_job.github_token.as_ref().map(|_| {
        spawn_refresh_task(
            api_client.clone(),
            worker_state.clone(),
            entry.job_id,
            workdir.clone(),
            recovered_job.github_token_expires_at,
        )
    });

    let prompt = recovery_continuation_prompt(current_turn, max_turns);
    let runtime = OmpRpcRuntime::new();
    let mut running_session = match runtime
        .resume(&prompt, &env, std::path::Path::new(session_path))
        .await
    {
        Ok(session) => session,
        Err(e) => {
            tracing::error!(
                job_id = %entry.job_id,
                error = %e,
                "failed to resume OMP RPC session"
            );
            stop_refresh_task(github_refresh_stop);
            mark_lost_and_submit(&journal, &api_client, &worker_state, &entry).await;
            return;
        }
    };
    let artifact_path = workdir.join("home").join("finish_artifact.json");
    let reporter = Arc::new(
        crate::daemon::job::execution::event_reporter::EventReporter::new(
            api_client.clone(),
            worker_state.clone(),
            entry.job_id,
        ),
    );
    reporter.emit(
        "session.recovered",
        serde_json::json!({"initial_turn": initial_turn, "backend": "omp_rpc"}),
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
        &mut running_session,
        &artifact_path,
        work_type,
        max_turns,
        initial_turn,
        &ctx,
    )
    .await;
    stop_refresh_task(github_refresh_stop);
    if let Some(session_id) = running_session.session_id().map(str::to_owned) {
        match running_session.export_messages().await {
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
    tracing::info!(job_id = %entry.job_id, "OMP RPC recovery session completed");
}
