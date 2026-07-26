use std::path::Path;

use vulcanum_shared::runtime::agent::RunningSession;
use vulcanum_shared::runtime::types::FinishRunArtifact;

use crate::daemon::job::execution::artifact::read_finish_artifact;
use crate::daemon::job::execution::submit::{
    submit_failed_result, submit_turn_result, FailedResult,
};
use crate::daemon::job::review::review_loop::{actionable_review_body, ReviewLoopState};

use super::session::{finish_exit_code, remove_finish_artifact};
use super::{PendingTurn, RecoveryTurn, StartupAction, StartupTurn, TurnLoopCtx};

#[must_use]
pub(crate) fn select_startup_action(
    artifact: Option<&FinishRunArtifact>,
    review_loop: &mut ReviewLoopState,
    turn: i32,
    pending_turn: Option<PendingTurn>,
    recovery_turn: Option<RecoveryTurn>,
) -> Option<StartupAction> {
    if let Some(pending) = pending_turn {
        let replays_artifact_transition = artifact.is_some_and(|artifact| {
            pending.cleanup_finish_artifact && actionable_review_body(artifact).is_some()
        });
        if artifact.is_none() || replays_artifact_transition {
            return Some(StartupAction::Continue(StartupTurn {
                prompt: pending.prompt,
                turn,
                cleanup_finish_artifact: pending.cleanup_finish_artifact,
                staged: true,
            }));
        }
    }

    if let Some(artifact) = artifact {
        let prompt = review_loop.prompt_after_artifact(artifact);
        return match prompt {
            Some(prompt) if turn < review_loop.effective_max_turns() => {
                Some(StartupAction::Continue(StartupTurn {
                    prompt,
                    turn: turn + 1,
                    cleanup_finish_artifact: true,
                    staged: false,
                }))
            }
            Some(_) | None => Some(StartupAction::SubmitArtifact),
        };
    }

    recovery_turn.map(|recovery| {
        StartupAction::Continue(StartupTurn {
            prompt: recovery.prompt,
            turn: recovery.turn,
            cleanup_finish_artifact: false,
            staged: false,
        })
    })
}

pub(super) enum RecoveredTurnStart {
    Continue,
    Submitted,
    Failed,
}

pub(super) async fn start_recovered_turn(
    running_session: &mut Box<dyn RunningSession>,
    artifact_path: &Path,
    review_loop: &mut ReviewLoopState,
    turn: &mut i32,
    pending_turn: Option<PendingTurn>,
    recovery_turn: Option<RecoveryTurn>,
    ctx: &TurnLoopCtx,
) -> RecoveredTurnStart {
    let artifact = read_finish_artifact(artifact_path);
    let action = select_startup_action(
        artifact.as_ref(),
        review_loop,
        *turn,
        pending_turn,
        recovery_turn,
    );
    let Some(action) = action else {
        return RecoveredTurnStart::Continue;
    };

    let startup = match action {
        StartupAction::Continue(startup) => startup,
        StartupAction::SubmitArtifact => {
            let Some(artifact) = artifact.as_ref() else {
                return RecoveredTurnStart::Failed;
            };
            return submit_recovered_artifact(running_session, artifact, ctx).await;
        }
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
    if !continued {
        return RecoveredTurnStart::Failed;
    }

    *turn = startup.turn;
    RecoveredTurnStart::Continue
}

async fn submit_recovered_artifact(
    running_session: &mut Box<dyn RunningSession>,
    artifact: &FinishRunArtifact,
    ctx: &TurnLoopCtx,
) -> RecoveredTurnStart {
    let mut session_export = match running_session.export().await {
        Ok(export) => export,
        Err(error) => {
            fail_continuation(
                ctx,
                0,
                &format!("failed to export recovered terminal artifact: {error}"),
            )
            .await;
            return RecoveredTurnStart::Failed;
        }
    };
    session_export.exit_code = finish_exit_code(artifact);
    submit_turn_result(
        &ctx.client,
        &ctx.worker_state,
        &ctx.journal,
        ctx.job_id,
        &session_export,
        Some(artifact),
    )
    .await;
    RecoveredTurnStart::Submitted
}

#[must_use]
pub(crate) fn continuation_prompt_for_dispatch(
    prompt: &str,
    job_id: uuid::Uuid,
    turn: i32,
) -> String {
    format!("{prompt}\n\n[Vulcanum continuation id: {job_id}:{turn}]")
}

pub(super) async fn continue_session(
    running_session: &mut Box<dyn RunningSession>,
    prompt: &str,
    next_turn: i32,
    review_loop: &ReviewLoopState,
    cleanup_finish_artifact: bool,
    artifact_path: &Path,
    ctx: &TurnLoopCtx,
) -> bool {
    let checkpoint = review_loop.checkpoint();
    let dispatch_prompt = continuation_prompt_for_dispatch(prompt, ctx.job_id, next_turn);
    if let Err(error) = ctx.journal.stage_turn(
        ctx.job_id,
        next_turn,
        checkpoint.fix_pass,
        checkpoint.fixing,
        &dispatch_prompt,
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
        &dispatch_prompt,
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
    match running_session.prompt_was_dispatched(prompt).await {
        Ok(true) => {
            if let Err(error) = ctx.journal.clear_pending_turn(ctx.job_id) {
                fail_continuation(
                    ctx,
                    turn,
                    &format!("failed to reconcile continuation: {error}"),
                )
                .await;
                return false;
            }
            return true;
        }
        Ok(false) => (),
        Err(error) => {
            fail_continuation(
                ctx,
                turn,
                &format!("failed to reconcile continuation dispatch: {error}"),
            )
            .await;
            return false;
        }
    }

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
