use vulcanum_shared::api_types::WorkRunType;
use vulcanum_shared::review_feedback::review_requires_implementation;
use vulcanum_shared::runtime::types::{FinishRunArtifact, FinishStatus};

use super::prompts::{review_after_fix_prompt, review_fix_prompt};

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
    max_fix_passes: i32,
    completed_fix_passes: i32,
}

impl ReviewLoopState {
    #[must_use]
    pub(crate) fn new(work_type: WorkRunType, max_fix_passes: i32) -> Self {
        Self {
            enabled: matches!(work_type, WorkRunType::PullRequestReview),
            phase: ReviewLoopPhase::Review,
            max_fix_passes: max_fix_passes.max(0),
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
        match self.enabled {
            true => (self.max_fix_passes * 2 + 1).max(1),
            false => self.max_fix_passes.max(1),
        }
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
