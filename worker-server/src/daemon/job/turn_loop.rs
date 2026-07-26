pub(super) mod continuation;
mod session;

use std::path::Path;
use std::sync::Arc;

use tokio::sync::{watch, RwLock};
use uuid::Uuid;

use vulcanum_shared::api::wire::WorkRunType;
use vulcanum_shared::client::ApiClient;
use vulcanum_shared::runtime::agent::RunningSession;
use vulcanum_shared::runtime::types::{FinishRunArtifact, FinishStatus, SessionExport};
use vulcanum_shared::state::worker::WorkerState;

use self::continuation::{continue_session, start_recovered_turn};
use self::session::{finish_exit_code, submit_provider_failure, wait_for_session};
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
