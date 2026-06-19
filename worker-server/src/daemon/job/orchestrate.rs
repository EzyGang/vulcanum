use std::collections::HashMap;
use std::sync::Arc;

use chrono::Utc;
use tokio::sync::RwLock;
use uuid::Uuid;

use vulcanum_shared::api_error::{is_fatal_api_error, ApiError};
use vulcanum_shared::client::ApiClient;
use vulcanum_shared::config::WorkerConfig;
use vulcanum_shared::runtime::agent::{AgentRuntime, RunningSession};
use vulcanum_shared::runtime::isolation::IsolationProvider;
use vulcanum_shared::runtime::types::ResourceLimits;
use vulcanum_shared::worker_state::WorkerState;

use super::event_reporter::EventReporter;
use super::submit::{submit_failed_result, FailedResult};
use super::turn_loop::{run_turn_loop, TurnLoopCtx};
use crate::daemon::auth::with_retry_on_401;
use crate::isolation::factory::create_isolation_provider;
use crate::providers::opencode::{self, api};
use crate::state::journal::Journal;
use crate::storage::messages::MessageStore;

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
    if let Err(e) = std::fs::create_dir_all(&workdir) {
        tracing::error!(work_run_id = %job_id, error = %e, "failed to create workdir");
        return Ok(());
    }

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
        tracing::warn!(work_run_id = %job_id, error = %e, "journal insert failed, continuing without tracking");
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

    let ctx = TurnLoopCtx {
        client: client.clone(),
        worker_state: worker_state.clone(),
        journal: journal.clone(),
        job_id,
        worker_id,
        reporter,
    };

    run_turn_loop(&mut running_session, &artifact_path, max_turns, 1, &ctx).await;

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
