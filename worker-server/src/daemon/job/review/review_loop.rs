use vulcanum_shared::api::wire::WorkRunType;
use vulcanum_shared::review_feedback::review_requires_implementation;
use vulcanum_shared::runtime::types::{FinishRunArtifact, FinishStatus};

use crate::daemon::job::prompts::text::{review_after_fix_prompt, review_fix_prompt};

#[derive(Clone, Copy, Eq, PartialEq)]
enum ReviewLoopPhase {
    Review,
    Fix,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct ReviewLoopCheckpoint {
    pub fix_pass: i32,
    pub fixing: bool,
}

pub(crate) struct ReviewLoopProgress {
    pub fix_pass: i32,
    pub max_fix_passes: i32,
}

pub(crate) struct ReviewLoopState {
    enabled: bool,
    phase: ReviewLoopPhase,
    max_turns: i32,
    max_fix_passes: i32,
    completed_fix_passes: i32,
}

#[must_use]
pub(crate) fn actionable_review_body(artifact: &FinishRunArtifact) -> Option<&str> {
    if !matches!(artifact.status, FinishStatus::Completed) {
        return None;
    }

    match artifact.review_body.as_deref() {
        Some(body) if review_requires_implementation(body) => Some(body),
        Some(_) | None => None,
    }
}

impl ReviewLoopState {
    #[must_use]
    pub(crate) fn new(work_type: WorkRunType, max_turns: i32) -> Self {
        let enabled = matches!(work_type, WorkRunType::PullRequestReview);
        let configured_turns = max_turns.max(1);
        let (max_turns, max_fix_passes) = match enabled {
            true => (
                configured_turns.saturating_mul(2).saturating_add(1),
                configured_turns,
            ),
            false => (configured_turns, configured_turns),
        };
        Self {
            enabled,
            phase: ReviewLoopPhase::Review,
            max_turns,
            max_fix_passes,
            completed_fix_passes: 0,
        }
    }
    #[must_use]
    pub(crate) fn resume(
        work_type: WorkRunType,
        max_turns: i32,
        checkpoint: ReviewLoopCheckpoint,
    ) -> Self {
        let mut state = Self::new(work_type, max_turns);
        if !state.enabled {
            return state;
        }

        state.completed_fix_passes = checkpoint.fix_pass.clamp(0, state.max_fix_passes);
        state.phase = match checkpoint.fixing {
            true => ReviewLoopPhase::Fix,
            false => ReviewLoopPhase::Review,
        };
        state
    }

    #[must_use]
    pub(crate) fn prompt_after_artifact(&mut self, artifact: &FinishRunArtifact) -> Option<String> {
        if !self.enabled {
            return None;
        }

        match self.phase {
            ReviewLoopPhase::Review => self.prompt_after_review_artifact(artifact),
            ReviewLoopPhase::Fix => self.prompt_after_fix_artifact(artifact),
        }
    }

    #[must_use]
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

    #[must_use]
    pub(crate) fn prompt_after_fix_turn_within_cap(&mut self, turn: i32) -> Option<String> {
        if turn >= self.effective_max_turns() {
            return None;
        }
        self.prompt_after_fix_turn()
    }

    #[must_use]
    pub(crate) fn effective_max_turns(&self) -> i32 {
        self.max_turns
    }

    #[must_use]
    pub(crate) fn progress(&self) -> ReviewLoopProgress {
        ReviewLoopProgress {
            fix_pass: self.completed_fix_passes,
            max_fix_passes: self.max_fix_passes,
        }
    }

    #[must_use]
    pub(crate) fn checkpoint(&self) -> ReviewLoopCheckpoint {
        ReviewLoopCheckpoint {
            fix_pass: self.completed_fix_passes,
            fixing: matches!(self.phase, ReviewLoopPhase::Fix),
        }
    }

    fn prompt_after_review_artifact(&mut self, artifact: &FinishRunArtifact) -> Option<String> {
        let review_body = actionable_review_body(artifact)?;

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
}
