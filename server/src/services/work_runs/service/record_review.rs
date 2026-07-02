use crate::db::work_runs::queries::prs::InsertReviewResultParams;
use crate::models::work_runs::model::WorkRun;
use crate::services::work_runs::service::WorkRunsService;

impl WorkRunsService {
    pub(crate) async fn record_review_result(&self, run: &WorkRun) {
        let pr_url = match run.review_target_pr_url.as_deref() {
            Some(url) => url,
            None => return,
        };
        let repo = run.review_target_repo_full_name.as_deref().unwrap_or("");

        if let Err(e) = self
            .work_runs_repo
            .insert_review_result(
                &self.db,
                InsertReviewResultParams {
                    work_run_id: run.id,
                    pr_url,
                    repo_full_name: repo,
                },
            )
            .await
        {
            tracing::warn!(work_run_id = %run.id, error = %e, "failed to record review result");
        }
    }
}

#[must_use]
pub(crate) fn review_comment(run: &WorkRun) -> String {
    let pr_url = run
        .review_target_pr_url
        .as_deref()
        .unwrap_or("the pull request");
    format!("Review completed for {pr_url}")
}
