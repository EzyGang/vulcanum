use std::path::Path;
use std::sync::Arc;

use tokio::sync::RwLock;
use uuid::Uuid;

use vulcanum_shared::client::ApiClient;
use vulcanum_shared::runtime::agent::RunningSession;
use vulcanum_shared::worker_state::WorkerState;

use super::artifact::read_finish_artifact;
use super::event_reporter::EventReporter;
use super::prompts::continuation_prompt;
use super::submit::{submit_failed_result, submit_turn_result, FailedResult};
use crate::state::journal::Journal;

pub(crate) struct TurnLoopCtx {
    pub client: Arc<ApiClient>,
    pub worker_state: Arc<RwLock<WorkerState>>,
    pub journal: Arc<Journal>,
    pub job_id: Uuid,
    pub worker_id: Uuid,
    pub reporter: Arc<EventReporter>,
}

pub(crate) async fn run_turn_loop(
    running_session: &mut Box<dyn RunningSession>,
    artifact_path: &Path,
    max_turns: i32,
    initial_turn: i32,
    ctx: &TurnLoopCtx,
) -> bool {
    let mut turn = initial_turn;

    loop {
        let session_export = match running_session.wait().await {
            Ok(export) => export,
            Err(e) => {
                tracing::error!(
                    worker_id = %ctx.worker_id,
                    work_run_id = %ctx.job_id,
                    turn = turn,
                    error = %e,
                    "session wait failed",
                );
                let _ = running_session.cancel().await;
                ctx.reporter.emit(
                    "session.failed",
                    serde_json::json!({"reason": "wait_error", "turn": turn}),
                );
                submit_failed_result(
                    ctx.client.clone(),
                    ctx.worker_state.clone(),
                    ctx.journal.clone(),
                    ctx.job_id,
                    &FailedResult::empty(),
                )
                .await;
                return false;
            }
        };

        tracing::info!(
            worker_id = %ctx.worker_id,
            work_run_id = %ctx.job_id,
            turn = turn,
            exit_code = session_export.exit_code,
            tokens_used = session_export.tokens_used,
            "turn completed",
        );

        ctx.reporter.emit(
            "turn.completed",
            serde_json::json!({
                "turn": turn,
                "exit_code": session_export.exit_code,
                "tokens_used": session_export.tokens_used,
            }),
        );

        if session_export.exit_code != 0 {
            tracing::warn!(
                worker_id = %ctx.worker_id,
                work_run_id = %ctx.job_id,
                turn = turn,
                exit_code = session_export.exit_code,
                "session failed, not continuing turn loop",
            );
            ctx.reporter.emit(
                "session.failed",
                serde_json::json!({
                    "reason": "nonzero_exit",
                    "turn": turn,
                    "exit_code": session_export.exit_code,
                    "tokens_used": session_export.tokens_used,
                    "model_used": session_export.model_used.clone(),
                    "provider_error": session_export.failure_payload.clone(),
                }),
            );
            submit_turn_result(
                &ctx.client,
                &ctx.worker_state,
                &ctx.journal,
                ctx.job_id,
                &session_export,
                None,
            )
            .await;
            return true;
        }

        let finish_artifact = read_finish_artifact(artifact_path);

        match finish_artifact {
            Some(ref artifact) => {
                tracing::info!(
                    worker_id = %ctx.worker_id,
                    work_run_id = %ctx.job_id,
                    status = %artifact.status,
                    "agent declared finish via artifact",
                );
                ctx.reporter.emit(
                    "finish.artifact.found",
                    serde_json::json!({"status": artifact.status.to_string()}),
                );
                submit_turn_result(
                    &ctx.client,
                    &ctx.worker_state,
                    &ctx.journal,
                    ctx.job_id,
                    &session_export,
                    Some(artifact),
                )
                .await;
                return true;
            }
            None => {
                if turn >= max_turns {
                    let mut failed_export = session_export.clone();
                    failed_export.exit_code = 1;
                    tracing::info!(
                        worker_id = %ctx.worker_id,
                        work_run_id = %ctx.job_id,
                        turn = turn,
                        max_turns = max_turns,
                        "max turns reached, submitting result",
                    );
                    ctx.reporter.emit(
                        "turn.max_reached",
                        serde_json::json!({"turn": turn, "max_turns": max_turns}),
                    );
                    submit_turn_result(
                        &ctx.client,
                        &ctx.worker_state,
                        &ctx.journal,
                        ctx.job_id,
                        &failed_export,
                        None,
                    )
                    .await;
                    return true;
                }

                let prompt = continuation_prompt(turn, max_turns);
                ctx.reporter.emit(
                    "turn.continuing",
                    serde_json::json!({"turn": turn, "next_turn": turn + 1}),
                );
                if let Err(e) = running_session.continue_with(&prompt).await {
                    tracing::error!(
                        worker_id = %ctx.worker_id,
                        work_run_id = %ctx.job_id,
                        turn = turn,
                        error = %e,
                        "continuation prompt failed",
                    );
                    ctx.reporter.emit(
                        "session.failed",
                        serde_json::json!({"reason": "continuation_failed", "turn": turn}),
                    );
                    submit_failed_result(
                        ctx.client.clone(),
                        ctx.worker_state.clone(),
                        ctx.journal.clone(),
                        ctx.job_id,
                        &FailedResult::empty(),
                    )
                    .await;
                    return false;
                }

                turn += 1;
                let _ = ctx.journal.update_turn(ctx.job_id, turn);
            }
        }
    }
}
