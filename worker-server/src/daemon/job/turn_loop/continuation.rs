use super::session::remove_finish_artifact;
use super::*;

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

pub(super) async fn start_recovered_turn(
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
