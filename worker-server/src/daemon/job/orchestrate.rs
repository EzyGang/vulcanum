mod duplicates;
mod heartbeat;
mod messages;
mod setup;

use chrono::Utc;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use vulcanum_shared::api_error::{is_fatal_api_error, ApiError};
use vulcanum_shared::client::ApiClient;
use vulcanum_shared::config::WorkerConfig;
use vulcanum_shared::runtime::agent::RunningSession;
use vulcanum_shared::runtime::isolation::IsolationProvider;
use vulcanum_shared::worker_state::WorkerState;

use super::execution::event_reporter::EventReporter;
use super::execution::submit::{submit_failed_result, FailedResult};
use super::github_credentials::stop_refresh_task;
use super::prompts::text::initial_prompt;
use super::turn_loop::{run_turn_loop, TurnLoopCtx};
use crate::daemon::auth::with_retry_on_401;
use crate::daemon::job::orchestrate::duplicates::reconcile_terminal_duplicate;
use crate::daemon::job::orchestrate::heartbeat::spawn_heartbeat;
use crate::daemon::job::orchestrate::messages::save_session_messages;
use crate::daemon::job::orchestrate::setup::{prepare_environment, PrepareEnvironmentCtx};
use crate::providers::runtime::AgentRuntimeKind;
use crate::state::journal::{Journal, JournalStatus};

pub(crate) async fn handle_job(
    client: Arc<ApiClient>,
    worker_state: Arc<RwLock<WorkerState>>,
    journal: Arc<Journal>,
    job_id: Uuid,
    config: &WorkerConfig,
) -> Result<(), String> {
    let harness_type = config.harness.as_str();
    let worker_id = worker_state.read().await.worker_id;

    tracing::info!(
        worker_id = %worker_id,
        work_run_id = %job_id,
        "job received",
    );

    let reporter = Arc::new(EventReporter::new(
        client.clone(),
        worker_state.clone(),
        job_id,
    ));

    let job = match with_retry_on_401(&client, &worker_state, |token| {
        let client = client.clone();
        async move { client.get_job(job_id, &token).await }
    })
    .await
    {
        Ok(j) => j,
        Err(e) => {
            if is_fatal_api_error(&e) {
                return Err(format!("get_job failed permanently: {e:#} — run `vulcanum worker setup --instance <instance> --code <code>` to reconnect"));
            }
            if let Some(api_err) = e.downcast_ref::<ApiError>() {
                if api_err.status == 404 {
                    tracing::info!(work_run_id = %job_id, "job was deleted or cancelled, skipping");
                    return Ok(());
                }
            }
            tracing::warn!(work_run_id = %job_id, error = %e, "get_job failed transiently, skipping");
            return Ok(());
        }
    };

    match journal.find_by_id(job_id) {
        Ok(Some(entry)) => match entry.status {
            JournalStatus::Running => {
                tracing::warn!(
                    work_run_id = %job_id,
                    local_status = ?entry.status,
                    workdir = %entry.workdir,
                    "duplicate dispatch ignored because job is still running locally"
                );
                return Ok(());
            }
            JournalStatus::Completed
            | JournalStatus::Failed
            | JournalStatus::Lost
            | JournalStatus::Submitted => {
                reconcile_terminal_duplicate(&client, &worker_state, &journal, job_id, &entry)
                    .await?;
                return Ok(());
            }
        },
        Ok(None) => (),
        Err(e) => {
            tracing::error!(work_run_id = %job_id, error = %e, "failed to check local journal before ack");
            return Ok(());
        }
    }

    if let Err(e) = with_retry_on_401(&client, &worker_state, |token| {
        let client = client.clone();
        async move { client.ack_job(job_id, &token).await }
    })
    .await
    {
        if is_fatal_api_error(&e) {
            return Err(format!("ack failed permanently: {e:#} — run `vulcanum worker setup --instance <instance> --code <code>` to reconnect"));
        }
        if let Some(api_err) = e.downcast_ref::<ApiError>() {
            if api_err.status == 404 {
                tracing::info!(work_run_id = %job_id, "job was deleted or cancelled, skipping ack");
                return Ok(());
            }
        }
        tracing::warn!(work_run_id = %job_id, error = %e, "ack failed transiently, skipping");
        return Ok(());
    }

    tracing::info!(
        worker_id = %worker_id,
        work_run_id = %job_id,
        external_task_ref = %job.external_task_ref,
        prompt_len = job.prompt_text.len(),
        "executing job",
    );

    let workdir = std::env::temp_dir().join(format!("vulcanum-work-{}", job_id));
    let workdir_str = workdir.to_string_lossy().to_string();

    let container_name = match harness_type {
        "kata" | "docker" => Some(crate::isolation::workspace::container_name(&workdir)),
        _ => None,
    };

    let max_turns = job.max_turns.max(1);
    let started_at = Utc::now();
    if let Err(e) = journal.insert_job(crate::state::journal::JournalInsert {
        job_id,
        workdir: &workdir_str,
        container_name: container_name.as_deref(),
        harness_type,
        started_at,
        max_turns,
        agent_backend: job.agent_backend.as_str(),
    }) {
        match journal.find_by_id(job_id) {
            Ok(Some(entry)) => {
                tracing::warn!(
                    work_run_id = %job_id,
                    local_status = ?entry.status,
                    workdir = %entry.workdir,
                    error = %e,
                    "journal insert found duplicate after ack, skipping duplicate job"
                );
                return Ok(());
            }
            Ok(None) | Err(_) => {
                tracing::error!(work_run_id = %job_id, error = %e, "journal insert failed, submitting failed result");
                submit_failed_result(
                    client,
                    worker_state,
                    journal,
                    job_id,
                    &FailedResult::empty(),
                )
                .await;
                return Ok(());
            }
        }
    }

    let prepared = match prepare_environment(PrepareEnvironmentCtx {
        client: client.clone(),
        worker_state: worker_state.clone(),
        journal: journal.clone(),
        reporter: reporter.clone(),
        worker_id,
        job_id,
        job: &job,
        config,
        workdir: &workdir,
    })
    .await
    {
        Ok(prepared) => prepared,
        Err(()) => return Ok(()),
    };
    let provider = prepared.provider;
    let isolated_env = prepared.isolated_env;
    let github_refresh_stop = prepared.github_refresh_stop;

    let prompt_text = initial_prompt(
        job.work_type,
        &crate::isolation::workspace::workspace_prompt_prefix(&isolated_env.repos),
        &job.prompt_text,
    );
    let runtime = AgentRuntimeKind::new(job.agent_backend);
    let mut running_session: Box<dyn RunningSession> =
        match runtime.execute(&prompt_text, &isolated_env).await {
            Ok(session) => session,
            Err(e) => {
                tracing::error!(
                    worker_id = %worker_id,
                    work_run_id = %job_id,
                    error = %e,
                    "runtime execute failed",
                );
                reporter
                    .emit(
                        "session.failed",
                        serde_json::json!({"reason": "runtime_execute_failed"}),
                    )
                    .await;
                reporter.shutdown().await;
                stop_refresh_task(github_refresh_stop);
                provider.cleanup(&isolated_env).await;
                submit_failed_result(
                    client,
                    worker_state,
                    journal,
                    job_id,
                    &FailedResult::empty(),
                )
                .await;
                return Ok(());
            }
        };

    tracing::info!(
        worker_id = %worker_id,
        work_run_id = %job_id,
        "session started, entering turn loop",
    );

    if let Some(sid) = running_session.session_id() {
        let _ = journal.set_session_id(job_id, sid);
    }
    if let Some((pid, port)) = running_session.host_server_info() {
        let _ = journal.set_host_info(job_id, pid.into(), port.into());
    }
    let _ = journal.set_agent_metadata(
        job_id,
        running_session.agent_session_path(),
        isolated_env
            .env_vars
            .get("PI_CONFIG_HOME")
            .map(String::as_str),
        isolated_env
            .env_vars
            .get("PI_STATE_HOME")
            .map(String::as_str),
        Some(job.agent_backend.as_str()),
        running_session.agent_pid().map(i64::from),
    );

    let artifact_path = workdir.join("home").join("finish_artifact.json");

    reporter
        .emit("session.started", serde_json::json!({}))
        .await;
    let heartbeat_stop = spawn_heartbeat(reporter.clone());
    let ctx = TurnLoopCtx {
        client: client.clone(),
        worker_state: worker_state.clone(),
        journal: journal.clone(),
        job_id,
        worker_id,
        reporter,
    };

    run_turn_loop(
        &mut running_session,
        &artifact_path,
        job.work_type,
        max_turns,
        1,
        &ctx,
    )
    .await;
    stop_refresh_task(github_refresh_stop);
    let _ = heartbeat_stop.send(true);
    ctx.reporter.shutdown().await;

    save_session_messages(job_id, &mut running_session).await;

    if let Err(error) = running_session.cleanup().await {
        tracing::warn!(
            work_run_id = %job_id,
            error = %error,
            "provider runtime cleanup returned an error",
        );
    }

    provider.cleanup(&isolated_env).await;

    Ok(())
}
