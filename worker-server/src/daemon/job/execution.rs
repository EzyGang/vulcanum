use std::collections::HashMap;
use std::sync::Arc;

use chrono::Utc;
use tokio::sync::RwLock;
use uuid::Uuid;

use vulcanum_shared::api_error::{is_fatal_api_error, ApiError};
use vulcanum_shared::client::ApiClient;
use vulcanum_shared::runtime::agent::{AgentRuntime, RunningSession};
use vulcanum_shared::runtime::isolation::IsolationProvider;
use vulcanum_shared::runtime::types::ResourceLimits;
use vulcanum_shared::worker_state::WorkerState;

use super::report::{read_finish_artifact, submit_failed_result, submit_turn_result, FailedResult};
use crate::harness::dispatch::create_isolation_provider;
use crate::state::journal::Journal;

fn continuation_prompt(turn: i32, max_turns: i32) -> String {
    format!(
        "[Continuation turn {turn}/{max_turns}]\n\
         The previous turn completed. The task remains active. \
         Continue from the current workspace state. Do not restart. \
         Focus on remaining work. When done, call the finish_run tool."
    )
}

pub(crate) async fn handle_job(
    client: Arc<ApiClient>,
    worker_state: Arc<RwLock<WorkerState>>,
    journal: Arc<Journal>,
    job_id: Uuid,
    harness_type: &str,
) -> Result<(), String> {
    let worker_id = worker_state.read().await.worker_id;

    tracing::info!(
        worker_id = %worker_id,
        work_run_id = %job_id,
        "job received",
    );

    let access_token = worker_state.read().await.access_token.clone();

    let job = match client.get_job(job_id, &access_token).await {
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

    if let Err(e) = client.ack_job(job_id, &access_token).await {
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
        "kata" | "gvisor" | "docker" => Some(crate::harness::prepare::container_name(&workdir)),
        _ => None,
    };

    let started_at = Utc::now();
    if let Err(e) = journal.insert_job(
        job_id,
        &workdir_str,
        container_name.as_deref(),
        harness_type,
        started_at,
    ) {
        tracing::warn!(work_run_id = %job_id, error = %e, "journal insert failed, continuing without tracking");
    }

    let provider = create_isolation_provider(harness_type);
    let limits = ResourceLimits::default();
    let mut secrets = HashMap::new();
    secrets.insert("KANEO_INSTANCE".to_owned(), job.kaneo_instance);
    secrets.insert("KANEO_API_KEY".to_owned(), job.kaneo_api_key);
    secrets.insert("KANEO_PROJECT_ID".to_owned(), job.kaneo_project_id);
    secrets.insert("KANEO_WORKSPACE_ID".to_owned(), job.kaneo_workspace_id);
    secrets.insert("KANEO_TASK_ID".to_owned(), job.external_task_ref.clone());
    let env_vars = HashMap::new();

    let isolated_env = match provider
        .prepare(
            &workdir,
            &secrets,
            &env_vars,
            &limits,
            &job.agents_md,
            &job.opencode_config,
            &job.repo_url,
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

    let runtime = crate::runtime::serve::OpenCodeServeRuntime::new();
    let mut running_session: Box<dyn RunningSession> = match runtime
        .execute(&job.prompt_text, &isolated_env, &job.repo_url)
        .await
    {
        Ok(session) => session,
        Err(e) => {
            tracing::error!(
                worker_id = %worker_id,
                work_run_id = %job_id,
                error = %e,
                "runtime execute failed",
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

    let artifact_path = workdir.join("home").join("finish_artifact.json");
    let max_turns = job.max_turns.max(1);
    let mut turn = 1;

    loop {
        let session_export = match running_session.wait().await {
            Ok(export) => export,
            Err(e) => {
                tracing::error!(
                    worker_id = %worker_id,
                    work_run_id = %job_id,
                    turn = turn,
                    error = %e,
                    "session wait failed",
                );
                let _ = running_session.cancel().await;
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
            turn = turn,
            exit_code = session_export.exit_code,
            tokens_used = session_export.tokens_used,
            "turn completed",
        );

        let finish_artifact = read_finish_artifact(&artifact_path);

        match finish_artifact {
            Some(ref artifact) => {
                tracing::info!(
                    worker_id = %worker_id,
                    work_run_id = %job_id,
                    status = %artifact.status,
                    "agent declared finish via artifact",
                );
                submit_turn_result(
                    &client,
                    &worker_state,
                    &journal,
                    job_id,
                    &session_export,
                    Some(artifact),
                )
                .await;
                break;
            }
            None => {
                if turn >= max_turns {
                    tracing::info!(
                        worker_id = %worker_id,
                        work_run_id = %job_id,
                        turn = turn,
                        max_turns = max_turns,
                        "max turns reached, submitting result",
                    );
                    submit_turn_result(
                        &client,
                        &worker_state,
                        &journal,
                        job_id,
                        &session_export,
                        None,
                    )
                    .await;
                    break;
                }

                let prompt = continuation_prompt(turn, max_turns);
                if let Err(e) = running_session.continue_with(&prompt).await {
                    tracing::error!(
                        worker_id = %worker_id,
                        work_run_id = %job_id,
                        turn = turn,
                        error = %e,
                        "continuation prompt failed",
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

                turn += 1;
                let _ = journal.update_turn(job_id, turn);
            }
        }
    }

    provider.cleanup(&isolated_env).await;

    Ok(())
}
