use vulcanum_shared::api::wire::WorkRunType;
use vulcanum_shared::review_feedback::review_requires_implementation;
use vulcanum_shared::runtime::types::{FinishRunArtifact, FinishStatus};

use crate::daemon::job::prompts::text::{review_after_fix_prompt, review_fix_prompt};

#[derive(Clone, Copy, Eq, PartialEq)]
enum ReviewLoopPhase {
    Review,
    Fix,
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

impl ReviewLoopState {
    #[must_use]
    pub(crate) fn new(work_type: WorkRunType, max_turns: i32) -> Self {
        let enabled = matches!(work_type, WorkRunType::PullRequestReview);
        let max_turns = max_turns.max(1);
        let max_fix_passes = match enabled {
            true => ((max_turns - 1) / 2).max(0),
            false => max_turns,
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
}
