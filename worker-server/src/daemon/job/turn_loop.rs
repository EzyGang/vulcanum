use std::path::Path;
use std::sync::Arc;

use tokio::sync::RwLock;
use uuid::Uuid;

use vulcanum_shared::api_types::WorkRunType;
use vulcanum_shared::client::ApiClient;
use vulcanum_shared::review_feedback::review_requires_implementation;
use vulcanum_shared::runtime::agent::RunningSession;
use vulcanum_shared::runtime::types::{FinishRunArtifact, FinishStatus};
use vulcanum_shared::worker_state::WorkerState;

use super::artifact::read_finish_artifact;
use super::event_reporter::EventReporter;
use super::prompts::{continuation_prompt, review_after_fix_prompt, review_fix_prompt};
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
    work_type: WorkRunType,
    max_turns: i32,
    initial_turn: i32,
    ctx: &TurnLoopCtx,
) -> bool {
    let mut turn = initial_turn;
    let mut review_loop = ReviewLoopState::new(work_type, max_turns);

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

        let finish_artifact = read_finish_artifact(artifact_path);

        if let Some(ref artifact) = finish_artifact {
            if let Some(prompt) = review_loop.prompt_after_artifact(artifact) {
                remove_finish_artifact(artifact_path);
                ctx.reporter.emit(
                    "review.fix.continuing",
                    serde_json::json!({
                        "turn": turn,
                        "fix_pass": review_loop.completed_fix_passes,
                        "max_fix_passes": review_loop.max_fix_passes,
                    }),
                );
                if !continue_session(running_session, &prompt, turn, ctx).await {
                    return false;
                }
                turn += 1;
                let _ = ctx.journal.update_turn(ctx.job_id, turn);
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
            ctx.reporter.emit(
                "finish.artifact.found",
                serde_json::json!({"status": artifact.status.to_string()}),
            );
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
            if session_export.exit_code != 0 {
                tracing::warn!(
                    worker_id = %ctx.worker_id,
                    work_run_id = %ctx.job_id,
                    turn = turn,
                    exit_code = session_export.exit_code,
                    provider_error = ?session_export.failure_payload,
                    "review fix turn failed, not continuing review loop",
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

            ctx.reporter.emit(
                "review.fix.completed",
                serde_json::json!({
                    "turn": turn,
                    "fix_pass": review_loop.completed_fix_passes,
                    "max_fix_passes": review_loop.max_fix_passes,
                }),
            );
            if !continue_session(running_session, &prompt, turn, ctx).await {
                return false;
            }
            turn += 1;
            let _ = ctx.journal.update_turn(ctx.job_id, turn);
            continue;
        }

        if session_export.exit_code != 0 {
            tracing::warn!(
                worker_id = %ctx.worker_id,
                work_run_id = %ctx.job_id,
                turn = turn,
                exit_code = session_export.exit_code,
                provider_error = ?session_export.failure_payload,
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
            ctx.reporter.emit(
                "turn.max_reached",
                serde_json::json!({"turn": turn, "max_turns": review_loop.effective_max_turns()}),
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

        let prompt = continuation_prompt(turn, review_loop.effective_max_turns());
        ctx.reporter.emit(
            "turn.continuing",
            serde_json::json!({"turn": turn, "next_turn": turn + 1}),
        );
        if !continue_session(running_session, &prompt, turn, ctx).await {
            return false;
        }

        turn += 1;
        let _ = ctx.journal.update_turn(ctx.job_id, turn);
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum ReviewLoopPhase {
    Review,
    Fix,
}

pub(crate) struct ReviewLoopState {
    enabled: bool,
    phase: ReviewLoopPhase,
    max_fix_passes: i32,
    completed_fix_passes: i32,
}

impl ReviewLoopState {
    pub(crate) fn new(work_type: WorkRunType, max_fix_passes: i32) -> Self {
        Self {
            enabled: matches!(work_type, WorkRunType::PullRequestReview),
            phase: ReviewLoopPhase::Review,
            max_fix_passes: max_fix_passes.max(0),
            completed_fix_passes: 0,
        }
    }

    pub(crate) fn prompt_after_artifact(&mut self, artifact: &FinishRunArtifact) -> Option<String> {
        if !self.enabled {
            return None;
        }

        match self.phase {
            ReviewLoopPhase::Review => self.prompt_after_review_artifact(artifact),
            ReviewLoopPhase::Fix => self.prompt_after_fix_artifact(artifact),
        }
    }

    fn prompt_after_review_artifact(&mut self, artifact: &FinishRunArtifact) -> Option<String> {
        if !matches!(artifact.status, FinishStatus::Completed) || artifact.review_already_exists {
            return None;
        }

        let review_body = artifact.review_body.as_deref()?;
        if !review_requires_implementation(review_body) {
            return None;
        }

        if self.completed_fix_passes >= self.max_fix_passes {
            return None;
        }

        self.completed_fix_passes += 1;
        self.phase = ReviewLoopPhase::Fix;
        Some(review_fix_prompt(review_body))
    }

    fn prompt_after_fix_artifact(&mut self, artifact: &FinishRunArtifact) -> Option<String> {
        if !matches!(artifact.status, FinishStatus::Completed) {
            return None;
        }

        self.phase = ReviewLoopPhase::Review;
        Some(review_after_fix_prompt(
            self.completed_fix_passes,
            self.max_fix_passes,
        ))
    }

    pub(crate) fn prompt_after_fix_turn(&mut self) -> Option<String> {
        if !self.enabled || !matches!(self.phase, ReviewLoopPhase::Fix) {
            return None;
        }

        self.phase = ReviewLoopPhase::Review;
        Some(review_after_fix_prompt(
            self.completed_fix_passes,
            self.max_fix_passes,
        ))
    }

    pub(crate) fn effective_max_turns(&self) -> i32 {
        match self.enabled {
            true => (self.max_fix_passes * 2 + 1).max(1),
            false => self.max_fix_passes.max(1),
        }
    }
}

async fn continue_session(
    running_session: &mut Box<dyn RunningSession>,
    prompt: &str,
    turn: i32,
    ctx: &TurnLoopCtx,
) -> bool {
    if let Err(e) = running_session.continue_with(prompt).await {
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

    true
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
