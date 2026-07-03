use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::RwLock;

use vulcanum_shared::api_types::{JobResponse, WorkRunType};
use vulcanum_shared::client::ApiClient;
use vulcanum_shared::constants::DEFAULT_IMAGE;
use vulcanum_shared::runtime::agent::RunningSession;
use vulcanum_shared::runtime::errors::HarnessError;
use vulcanum_shared::runtime::isolation::IsolationProvider;
use vulcanum_shared::runtime::types::{IsolatedEnvironment, ResourceLimits};
use vulcanum_shared::worker_state::WorkerState;

use crate::daemon::auth::with_retry_on_401;
use crate::daemon::job::execution::submit::{submit_result_request, SubmitResultParams};
use crate::daemon::job::github_credentials::{setup_recovered_credentials, spawn_refresh_task};
use crate::daemon::job::runtime_secrets::job_runtime_secrets;
use crate::daemon::job::turn_loop::{run_turn_loop, TurnLoopCtx};
use crate::isolation::github_credentials as isolation_github_credentials;
use crate::isolation::providers::docker::DockerIsolation;
use crate::isolation::providers::host::HostIsolation;
use crate::isolation::providers::kata::KataIsolation;
use crate::isolation::workspace;
use crate::providers::omp_rpc::runtime::OmpRpcRuntime;
use crate::providers::opencode::events;
use crate::providers::opencode::runner::OpenCodeRunningSession;
use crate::providers::opencode::runner::SessionConfig;
use crate::providers::opencode::OpenCodeClient;
use crate::recovery::cleanup::{cleanup_stale_job, kill_host_process_group};
use crate::state::journal::{Journal, JournalEntry, JournalResultUpdate, JournalStatus};

fn recovery_continuation_prompt(turn: i32, max_turns: i32) -> String {
    let next_turn = turn + 1;
    let final_turn_instruction = match next_turn >= max_turns {
        true => " This is the final allowed turn; before stopping, call the finish_run tool.",
        false => "",
    };

    format!(
        "[Continuation turn {next_turn}/{max_turns}]\n\
         The previous turn completed. The task remains active. \
         Continue from the current workspace state. Do not restart. \
         The workspace may contain multiple sibling repositories; run commands from the relevant repo directory. \
         Focus on remaining work. When done, call the finish_run tool.{final_turn_instruction}"
    )
}

pub(super) async fn recovered_omp_env(
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
    let github_refresh_stop = recovered_job.github_token.as_ref().map(|_| {
        spawn_refresh_task(
            api_client.clone(),
            worker_state.clone(),
            entry.job_id,
            workdir.clone(),
            recovered_job.github_token_expires_at,
        )
    });
    run_turn_loop(
        &mut running_session,
        &artifact_path,
        work_type,
        max_turns,
        initial_turn,
        &ctx,
    )
    .await;
    if let Some(stop) = github_refresh_stop {
        let _ = stop.send(true);
    }

    cleanup_recovery(&entry);
    tracing::info!(job_id = %entry.job_id, "OMP RPC recovery session completed");
}

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
    if let Some(stop) = github_refresh_stop {
        let _ = stop.send(true);
    }

    cleanup_recovery(&entry);
    tracing::info!(job_id = %entry.job_id, "recovery session completed");
}

fn cleanup_recovery(entry: &JournalEntry) {
    let env = IsolatedEnvironment {
        workdir: std::path::PathBuf::from(&entry.workdir),
        workspace_dir: std::path::PathBuf::from(&entry.workdir).join("workspace"),
        repos: Vec::new(),
        container_name: entry.container_name.clone(),
        secrets: HashMap::new(),
        env_vars: HashMap::new(),
        runtime: (entry.harness_type == "kata").then_some("kata-runtime"),
        image: Some(DEFAULT_IMAGE.to_owned()),
        server_host_port: None,
        limits: ResourceLimits::default(),
    };

    match entry.harness_type.as_str() {
        "host" => {
            kill_host_process_group(entry);
            tokio::spawn(async move {
                HostIsolation::new().cleanup(&env).await;
            });
        }
        "kata" => {
            tokio::spawn(async move {
                KataIsolation::new(DEFAULT_IMAGE.to_owned())
                    .cleanup(&env)
                    .await;
            });
        }
        _ => {
            tokio::spawn(async move {
                DockerIsolation::new(None, DEFAULT_IMAGE.to_owned())
                    .cleanup(&env)
                    .await;
            });
        }
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
