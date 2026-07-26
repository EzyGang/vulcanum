use std::path::Path;
use std::sync::Arc;

use tokio::sync::{watch, RwLock};
use uuid::Uuid;

use vulcanum_shared::api::wire::WorkRunType;
use vulcanum_shared::client::ApiClient;
use vulcanum_shared::runtime::agent::RunningSession;
use vulcanum_shared::runtime::types::{FinishRunArtifact, FinishStatus, SessionExport};
use vulcanum_shared::state::worker::WorkerState;

use super::execution::artifact::read_finish_artifact;
use super::execution::event_reporter::EventReporter;
use super::execution::submit::{submit_failed_result, submit_turn_result, FailedResult};
use super::prompts::text::continuation_prompt;
use super::review::review_loop::{ReviewLoopCheckpoint, ReviewLoopState};
use crate::state::journal::Journal;

pub(crate) struct TurnLoopCtx {
    pub client: Arc<ApiClient>,
    pub worker_state: Arc<RwLock<WorkerState>>,
    pub journal: Arc<Journal>,
    pub job_id: Uuid,
    pub worker_id: Uuid,
    pub reporter: Arc<EventReporter>,
}
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct PendingTurn {
    pub prompt: String,
    pub cleanup_finish_artifact: bool,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct RecoveryTurn {
    pub prompt: String,
    pub turn: i32,
}
pub(crate) struct TurnLoopStart {
    pub work_type: WorkRunType,
    pub max_turns: i32,
    pub turn: i32,
    pub review_checkpoint: ReviewLoopCheckpoint,
    pub pending_turn: Option<PendingTurn>,
    pub recovery_turn: Option<RecoveryTurn>,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct StartupTurn {
    pub prompt: String,
    pub turn: i32,
    pub cleanup_finish_artifact: bool,
    pub staged: bool,
}

pub(crate) async fn run_turn_loop(
    running_session: &mut Box<dyn RunningSession>,
    artifact_path: &Path,
    start: TurnLoopStart,
    ctx: &TurnLoopCtx,
) -> bool {
    let mut turn = start.turn;
    let max_turns = start.max_turns;
    let mut review_loop =
        ReviewLoopState::resume(start.work_type, max_turns, start.review_checkpoint);
    if !start_recovered_turn(
        running_session,
        artifact_path,
        &mut review_loop,
        &mut turn,
        start.pending_turn,
        start.recovery_turn,
        ctx,
    )
    .await
    {
        return false;
    }
    let mut cancel_rx = ctx.reporter.cancel_receiver();

    loop {
        let session_export =
            match wait_for_session(running_session, &mut cancel_rx, turn, ctx).await {
                Some(export) => export,
                None => return false,
            };

        tracing::info!(
            worker_id = %ctx.worker_id,
            work_run_id = %ctx.job_id,
            turn = turn,
            exit_code = session_export.exit_code,
            tokens_used = session_export.tokens_used,
            "turn completed",
        );

        ctx.reporter
            .emit(
                "turn.completed",
                serde_json::json!({
                    "turn": turn,
                    "exit_code": session_export.exit_code,
                    "tokens_used": session_export.tokens_used,
                }),
            )
            .await;

        if session_export.exit_code != 0 {
            submit_provider_failure(ctx, turn, &session_export).await;
            return true;
        }

        let finish_artifact = read_finish_artifact(artifact_path);

        if let Some(artifact) = &finish_artifact {
            if let Some(prompt) = review_loop.prompt_after_artifact(artifact) {
                let progress = review_loop.progress();
                ctx.reporter
                    .emit(
                        "review.fix.continuing",
                        serde_json::json!({
                            "turn": turn,
                            "fix_pass": progress.fix_pass,
                            "max_fix_passes": progress.max_fix_passes,
                        }),
                    )
                    .await;
                let next_turn = turn + 1;
                if !continue_session(
                    running_session,
                    &prompt,
                    next_turn,
                    &review_loop,
                    true,
                    artifact_path,
                    ctx,
                )
                .await
                {
                    return false;
                }
                turn = next_turn;
                continue;
            }

            let mut artifact_export = session_export.clone();
            artifact_export.exit_code = finish_exit_code(artifact);
            tracing::info!(
                worker_id = %ctx.worker_id,
                work_run_id = %ctx.job_id,
                status = %artifact.status,
                "agent declared finish via artifact",
            );
            ctx.reporter
                .emit(
                    "finish.artifact.found",
                    serde_json::json!({"status": artifact.status.to_string()}),
                )
                .await;
            submit_turn_result(
                &ctx.client,
                &ctx.worker_state,
                &ctx.journal,
                ctx.job_id,
                &artifact_export,
                Some(artifact),
            )
            .await;
            return true;
        }

        if let Some(prompt) = review_loop.prompt_after_fix_turn() {
            let progress = review_loop.progress();
            ctx.reporter
                .emit(
                    "review.fix.completed",
                    serde_json::json!({
                        "turn": turn,
                        "fix_pass": progress.fix_pass,
                        "max_fix_passes": progress.max_fix_passes,
                    }),
                )
                .await;
            let next_turn = turn + 1;
            if !continue_session(
                running_session,
                &prompt,
                next_turn,
                &review_loop,
                false,
                artifact_path,
                ctx,
            )
            .await
            {
                return false;
            }
            turn = next_turn;
            continue;
        }

        if turn >= review_loop.effective_max_turns() {
            let mut failed_export = session_export.clone();
            failed_export.exit_code = 1;
            tracing::info!(
                worker_id = %ctx.worker_id,
                work_run_id = %ctx.job_id,
                turn = turn,
                max_turns = max_turns,
                "max turns reached, submitting result",
            );
            ctx.reporter
                .emit(
                    "turn.max_reached",
                    serde_json::json!({
                        "turn": turn,
                        "max_turns": review_loop.effective_max_turns(),
                    }),
                )
                .await;
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

        let prompt = continuation_prompt(turn, review_loop.effective_max_turns());
        ctx.reporter
            .emit(
                "turn.continuing",
                serde_json::json!({"turn": turn, "next_turn": turn + 1}),
            )
            .await;
        let next_turn = turn + 1;
        if !continue_session(
            running_session,
            &prompt,
            next_turn,
            &review_loop,
            false,
            artifact_path,
            ctx,
        )
        .await
        {
            return false;
        }
        turn = next_turn;
    }
}

async fn wait_for_session(
    running_session: &mut Box<dyn RunningSession>,
    cancel_rx: &mut watch::Receiver<bool>,
    turn: i32,
    ctx: &TurnLoopCtx,
) -> Option<SessionExport> {
    if *cancel_rx.borrow() {
        return cancel_running_session(running_session, turn, ctx).await;
    }

    loop {
        tokio::select! {
            result = running_session.wait() => {
                return match result {
                    Ok(export) => Some(export),
                    Err(error) => {
                        tracing::error!(
                            worker_id = %ctx.worker_id,
                            work_run_id = %ctx.job_id,
                            turn = turn,
                            error = %error,
                            "session wait failed",
                        );
                        let _ = running_session.cancel().await;
                        ctx.reporter
                            .emit(
                                "session.failed",
                                serde_json::json!({"reason": "wait_error", "turn": turn}),
                            )
                            .await;
                        submit_failed_result(
                            ctx.client.clone(),
                            ctx.worker_state.clone(),
                            ctx.journal.clone(),
                            ctx.job_id,
                            &FailedResult::empty(),
                        )
                        .await;
                        None
                    }
                };
            }
            changed = cancel_rx.changed() => match changed {
                Ok(()) if *cancel_rx.borrow() => {
                    return cancel_running_session(running_session, turn, ctx).await;
                }
                Ok(()) => (),
                Err(_) => (),
            },
        }
    }
}

async fn cancel_running_session(
    running_session: &mut Box<dyn RunningSession>,
    turn: i32,
    ctx: &TurnLoopCtx,
) -> Option<SessionExport> {
    tracing::warn!(
        worker_id = %ctx.worker_id,
        work_run_id = %ctx.job_id,
        turn = turn,
        "server-requested cancellation received, cancelling running session",
    );
    if let Err(error) = running_session.cancel().await {
        tracing::warn!(
            worker_id = %ctx.worker_id,
            work_run_id = %ctx.job_id,
            turn = turn,
            error = %error,
            "provider cancellation returned an error after cleanup",
        );
    }
    ctx.reporter
        .emit(
            "session.cancelled",
            serde_json::json!({"reason": "server_requested", "turn": turn}),
        )
        .await;
    match running_session.export().await {
        Ok(export) => {
            submit_turn_result(
                &ctx.client,
                &ctx.worker_state,
                &ctx.journal,
                ctx.job_id,
                &export,
                None,
            )
            .await;
        }
        Err(error) => {
            tracing::warn!(
                worker_id = %ctx.worker_id,
                work_run_id = %ctx.job_id,
                turn = turn,
                error = %error,
                "failed to export cancelled session",
            );
            submit_failed_result(
                ctx.client.clone(),
                ctx.worker_state.clone(),
                ctx.journal.clone(),
                ctx.job_id,
                &FailedResult::empty(),
            )
            .await;
        }
    }
    None
}

async fn submit_provider_failure(ctx: &TurnLoopCtx, turn: i32, session_export: &SessionExport) {
    tracing::warn!(
        worker_id = %ctx.worker_id,
        work_run_id = %ctx.job_id,
        turn = turn,
        exit_code = session_export.exit_code,
        provider_error = ?session_export.failure_payload,
        "session failed, not continuing turn loop",
    );
    ctx.reporter
        .emit(
            "session.failed",
            serde_json::json!({
                "reason": "nonzero_exit",
                "turn": turn,
                "exit_code": session_export.exit_code,
                "tokens_used": session_export.tokens_used,
                "model_used": session_export.model_used.clone(),
                "provider_error": session_export.failure_payload.clone(),
            }),
        )
        .await;
    submit_turn_result(
        &ctx.client,
        &ctx.worker_state,
        &ctx.journal,
        ctx.job_id,
        session_export,
        None,
    )
    .await;
}

#[must_use]
pub(crate) fn select_startup_turn(
    artifact: Option<&FinishRunArtifact>,
    review_loop: &mut ReviewLoopState,
    turn: i32,
    pending_turn: Option<PendingTurn>,
    recovery_turn: Option<RecoveryTurn>,
) -> Option<StartupTurn> {
    match pending_turn {
        Some(pending) => Some(StartupTurn {
            prompt: pending.prompt,
            turn,
            cleanup_finish_artifact: pending.cleanup_finish_artifact,
            staged: true,
        }),
        None => {
            let review_prompt =
                artifact.and_then(|artifact| review_loop.prompt_after_artifact(artifact));
            match review_prompt {
                Some(prompt) => Some(StartupTurn {
                    prompt,
                    turn: turn + 1,
                    cleanup_finish_artifact: true,
                    staged: false,
                }),
                None => recovery_turn.map(|recovery| StartupTurn {
                    prompt: recovery.prompt,
                    turn: recovery.turn,
                    cleanup_finish_artifact: false,
                    staged: false,
                }),
            }
        }
    }
}

async fn start_recovered_turn(
    running_session: &mut Box<dyn RunningSession>,
    artifact_path: &Path,
    review_loop: &mut ReviewLoopState,
    turn: &mut i32,
    pending_turn: Option<PendingTurn>,
    recovery_turn: Option<RecoveryTurn>,
    ctx: &TurnLoopCtx,
) -> bool {
    let artifact = read_finish_artifact(artifact_path);
    let Some(startup) = select_startup_turn(
        artifact.as_ref(),
        review_loop,
        *turn,
        pending_turn,
        recovery_turn,
    ) else {
        return true;
    };

    let continued = match startup.staged {
        true => {
            dispatch_staged_turn(
                running_session,
                &startup.prompt,
                startup.turn,
                startup.cleanup_finish_artifact,
                artifact_path,
                ctx,
            )
            .await
        }
        false => {
            continue_session(
                running_session,
                &startup.prompt,
                startup.turn,
                review_loop,
                startup.cleanup_finish_artifact,
                artifact_path,
                ctx,
            )
            .await
        }
    };
    if continued {
        *turn = startup.turn;
    }
    continued
}

async fn continue_session(
    running_session: &mut Box<dyn RunningSession>,
    prompt: &str,
    next_turn: i32,
    review_loop: &ReviewLoopState,
    cleanup_finish_artifact: bool,
    artifact_path: &Path,
    ctx: &TurnLoopCtx,
) -> bool {
    let checkpoint = review_loop.checkpoint();
    if let Err(error) = ctx.journal.stage_turn(
        ctx.job_id,
        next_turn,
        checkpoint.fix_pass,
        checkpoint.fixing,
        prompt,
        cleanup_finish_artifact,
    ) {
        fail_continuation(
            ctx,
            next_turn,
            &format!("failed to stage continuation: {error}"),
        )
        .await;
        return false;
    }

    dispatch_staged_turn(
        running_session,
        prompt,
        next_turn,
        cleanup_finish_artifact,
        artifact_path,
        ctx,
    )
    .await
}

async fn dispatch_staged_turn(
    running_session: &mut Box<dyn RunningSession>,
    prompt: &str,
    turn: i32,
    cleanup_finish_artifact: bool,
    artifact_path: &Path,
    ctx: &TurnLoopCtx,
) -> bool {
    if cleanup_finish_artifact {
        remove_finish_artifact(artifact_path);
    }
    if let Err(error) = running_session.continue_with(prompt).await {
        fail_continuation(ctx, turn, &format!("continuation prompt failed: {error}")).await;
        return false;
    }
    if let Err(error) = ctx.journal.clear_pending_turn(ctx.job_id) {
        fail_continuation(
            ctx,
            turn,
            &format!("failed to finalize continuation: {error}"),
        )
        .await;
        return false;
    }
    true
}

async fn fail_continuation(ctx: &TurnLoopCtx, turn: i32, error: &str) {
    tracing::error!(
        worker_id = %ctx.worker_id,
        work_run_id = %ctx.job_id,
        turn,
        error,
        "continuation failed",
    );
    ctx.reporter
        .emit(
            "session.failed",
            serde_json::json!({"reason": "continuation_failed", "turn": turn}),
        )
        .await;
    submit_failed_result(
        ctx.client.clone(),
        ctx.worker_state.clone(),
        ctx.journal.clone(),
        ctx.job_id,
        &FailedResult::empty(),
    )
    .await;
}

fn remove_finish_artifact(path: &Path) {
    match std::fs::remove_file(path) {
        Ok(()) => (),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => (),
        Err(e) => {
            tracing::warn!(path = %path.display(), error = %e, "failed to remove finish artifact")
        }
    }
}

#[must_use]
fn finish_exit_code(artifact: &FinishRunArtifact) -> i32 {
    match artifact.status {
        FinishStatus::Completed => 0,
        FinishStatus::Failed | FinishStatus::Blocked => 1,
    }
}
