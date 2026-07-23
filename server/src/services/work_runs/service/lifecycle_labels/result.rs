use crate::models::work_runs::model::{WorkRun, WorkRunStatus, WorkRunType};
use crate::services::work_runs::service::lifecycle_labels::LifecycleLabelState;
use crate::services::work_runs::service::spawn_review::ReviewSpawnOutcome;
use crate::services::work_runs::service::WorkRunsService;

impl WorkRunsService {
    pub(crate) async fn set_lifecycle_label_after_result(
        &self,
        run: &WorkRun,
        status: WorkRunStatus,
        review_outcome: Option<ReviewSpawnOutcome>,
    ) {
        if run.is_standalone_review() {
            return;
        }

        let state = match (run.work_type, status) {
            (WorkRunType::Implementation, WorkRunStatus::Completed) => {
                match review_outcome.unwrap_or(ReviewSpawnOutcome::NoPullRequests) {
                    ReviewSpawnOutcome::NoPullRequests => Some(LifecycleLabelState::ReadyForHuman),
                    ReviewSpawnOutcome::ReviewNeeded => Some(LifecycleLabelState::ReviewNeeded),
                    ReviewSpawnOutcome::ReviewRunning => Some(LifecycleLabelState::ReviewRunning),
                }
            }
            (WorkRunType::Implementation, WorkRunStatus::Failed) => {
                Some(LifecycleLabelState::NeedsAttention)
            }
            (WorkRunType::PullRequestReview, WorkRunStatus::Completed) => {
                self.review_completion_lifecycle_label(run).await
            }
            (WorkRunType::PullRequestReview, WorkRunStatus::Failed) => {
                Some(LifecycleLabelState::NeedsAttention)
            }
            _ => None,
        };

        if let Some(state) = state {
            self.set_lifecycle_label_for_run(run, state).await;
        }
    }

    async fn review_completion_lifecycle_label(
        &self,
        run: &WorkRun,
    ) -> Option<LifecycleLabelState> {
        let parent_id = match run.parent_work_run_id {
            Some(parent_id) => parent_id,
            None => return Some(LifecycleLabelState::ReadyForHuman),
        };

        let summary = match self
            .work_runs_repo
            .review_sibling_summary(&self.db, parent_id, run.id)
            .await
        {
            Ok(summary) => summary,
            Err(e) => {
                tracing::warn!(
                    work_run_id = %run.id,
                    parent_work_run_id = %parent_id,
                    error = %e,
                    "failed to load review sibling summary for lifecycle labels",
                );
                return Some(LifecycleLabelState::ReadyForHuman);
            }
        };

        if summary.failed_count > 0 {
            return Some(LifecycleLabelState::NeedsAttention);
        }

        match summary.active_count {
            0 => Some(LifecycleLabelState::ReadyForHuman),
            _ => None,
        }
    }
}
