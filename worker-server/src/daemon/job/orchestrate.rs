use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use chrono::Utc;
use tokio::sync::{watch, RwLock};
use uuid::Uuid;

use vulcanum_shared::api_error::{is_fatal_api_error, ApiError};
use vulcanum_shared::client::ApiClient;
use vulcanum_shared::config::WorkerConfig;
use vulcanum_shared::runtime::agent::{AgentRuntime, RunningSession};
use vulcanum_shared::runtime::isolation::IsolationProvider;
use vulcanum_shared::runtime::types::ResourceLimits;
use vulcanum_shared::worker_state::WorkerState;

use super::event_reporter::EventReporter;
use super::submit::{resubmit_stored_result, submit_failed_result, FailedResult};
use super::turn_loop::{run_turn_loop, TurnLoopCtx};
use crate::daemon::auth::with_retry_on_401;
use crate::isolation::factory::create_isolation_provider;
use crate::providers::opencode::{self, api};
use crate::state::journal::{Journal, JournalEntry, JournalStatus};
use crate::storage::messages::MessageStore;

const HEARTBEAT_INTERVAL_SECS: u64 = 60;

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
    if let Err(e) = journal.insert_job(
        job_id,
        &workdir_str,
        container_name.as_deref(),
        harness_type,
        started_at,
        max_turns,
    ) {
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

    if let Err(e) = reset_fresh_workdir(&workdir).await {
        tracing::error!(work_run_id = %job_id, workdir = %workdir.display(), error = %e, "failed to reset stale workdir");
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

    if let Err(e) = tokio::fs::create_dir_all(&workdir).await {
        tracing::error!(work_run_id = %job_id, error = %e, "failed to create workdir");
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

    let provider = create_isolation_provider(config);
    let limits = ResourceLimits::default();
    let mut secrets = HashMap::new();
    secrets.insert(
        "PROVIDER_INSTANCE_URL".to_owned(),
        job.provider_instance_url,
    );
    secrets.insert("PROVIDER_API_KEY".to_owned(), job.provider_api_key);
    secrets.insert("EXTERNAL_PROJECT_ID".to_owned(), job.external_project_id);
    secrets.insert(
        "EXTERNAL_WORKSPACE_ID".to_owned(),
        job.external_workspace_id,
    );
    secrets.insert("EXTERNAL_TASK_ID".to_owned(), job.external_task_ref.clone());
    if let Some(ref token) = job.github_token {
        secrets.insert("GITHUB_TOKEN".to_owned(), token.clone());
    }
    if let Some(ref auth_content) = job.opencode_auth_content {
        secrets.insert("OPENCODE_AUTH_CONTENT".to_owned(), auth_content.clone());
    }
    for (key, value) in &job.model_provider_env {
        secrets.insert(key.clone(), value.clone());
    }
    let env_vars = HashMap::new();

    let isolated_env = match provider
        .prepare(
            &workdir,
            &secrets,
            &env_vars,
            &limits,
            job.work_type,
            &job.agents_md,
            &job.generated_opencode_config,
            &job.repos,
        )
        .await
    {
        Ok(env) => env,
        Err(e) => {
            tracing::error!(
                worker_id = %worker_id,
                work_run_id = %job_id,
                external_task_ref = %job.external_task_ref,
                error = %e,
                "isolation prepare failed",
            );
            reporter.emit(
                "session.failed",
                serde_json::json!({"reason": "isolation_prepare_failed"}),
            );
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

    if let (Some(pr_url), Some(repo_full_name)) = (
        job.review_target_pr_url.as_deref(),
        job.review_target_repo_full_name.as_deref(),
    ) {
        match crate::isolation::checkout::checkout_pull_request(
            &isolated_env.workspace_dir,
            &isolated_env.repos,
            repo_full_name,
            pr_url,
            job.github_token.as_deref(),
        )
        .await
        {
            Ok(()) => (),
            Err(e) => {
                tracing::error!(
                    worker_id = %worker_id,
                    work_run_id = %job_id,
                    repo = %repo_full_name,
                    pr_url = %pr_url,
                    error = %e,
                    "pull request checkout failed",
                );
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
        }
    }

    let prompt_text = super::prompts::initial_prompt(
        job.work_type,
        &crate::isolation::workspace::workspace_prompt_prefix(&isolated_env.repos),
        &job.prompt_text,
    );
    let runtime = crate::providers::opencode::runtime::OpenCodeServeRuntime::new();
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
                reporter.emit(
                    "session.failed",
                    serde_json::json!({"reason": "runtime_execute_failed"}),
                );
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

    let artifact_path = workdir.join("home").join("finish_artifact.json");

    reporter.emit("session.started", serde_json::json!({}));
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
    let _ = heartbeat_stop.send(true);

    if let (Some(sid), Some(base_url)) = (
        running_session.session_id(),
        running_session.agent_base_url(),
    ) {
        let oc_client = opencode::OpenCodeClient::new(base_url);
        match api::get_session_messages(&oc_client, sid, None).await {
            Ok(messages) => match MessageStore::new() {
                Ok(store) => {
                    let _ = store.save(job_id, sid, &messages);
                }
                Err(e) => {
                    tracing::warn!(work_run_id = %job_id, error = %e, "failed to create message store");
                }
            },
            Err(e) => {
                tracing::warn!(work_run_id = %job_id, error = %e, "failed to fetch session messages");
            }
        }
    }

    provider.cleanup(&isolated_env).await;

    Ok(())
}

async fn reconcile_terminal_duplicate(
    client: &Arc<ApiClient>,
    worker_state: &Arc<RwLock<WorkerState>>,
    journal: &Arc<Journal>,
    job_id: Uuid,
    entry: &JournalEntry,
) -> Result<(), String> {
    tracing::warn!(
        work_run_id = %job_id,
        local_status = ?entry.status,
        workdir = %entry.workdir,
        "duplicate dispatch matches terminal local journal state, resubmitting stored result"
    );

    match with_retry_on_401(client, worker_state, |token| {
        let client = client.clone();
        async move { client.ack_job(job_id, &token).await }
    })
    .await
    {
        Ok(()) => (),
        Err(e) => {
            if is_fatal_api_error(&e) {
                return Err(format!("ack failed permanently: {e:#} — run `vulcanum worker setup --instance <instance> --code <code>` to reconnect"));
            }
            match e.downcast_ref::<ApiError>() {
                Some(api_err) if api_err.status == 404 => {
                    tracing::info!(work_run_id = %job_id, "job was deleted or cancelled, skipping duplicate reconciliation");
                    return Ok(());
                }
                Some(api_err) if api_err.status == 409 => {
                    tracing::warn!(
                        work_run_id = %job_id,
                        error = %e,
                        "duplicate reconciliation ack was rejected, attempting result resubmit anyway"
                    );
                }
                _ => {
                    tracing::warn!(work_run_id = %job_id, error = %e, "ack failed during duplicate reconciliation");
                    return Ok(());
                }
            }
        }
    }

    resubmit_stored_result(client, worker_state, journal, entry).await;
    Ok(())
}

async fn reset_fresh_workdir(workdir: &Path) -> std::io::Result<()> {
    if !workdir.exists() {
        return Ok(());
    }
    if !is_safe_workdir(workdir) {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "refusing to delete unsafe workdir",
        ));
    }
    tokio::fs::remove_dir_all(workdir).await
}

fn is_safe_workdir(path: &Path) -> bool {
    let temp = std::env::temp_dir();
    path.starts_with(&temp)
        && path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with("vulcanum-work-"))
}

fn spawn_heartbeat(reporter: Arc<EventReporter>) -> watch::Sender<bool> {
    let (tx, mut rx) = watch::channel(false);
    tokio::spawn(async move {
        loop {
            tokio::select! {
                _ = tokio::time::sleep(Duration::from_secs(HEARTBEAT_INTERVAL_SECS)) => {
                    reporter.emit("worker.heartbeat", serde_json::json!({}));
                }
                changed = rx.changed() => {
                    if changed.is_err() || *rx.borrow() {
                        break;
                    }
                }
            }
        }
    });
    tx
}
