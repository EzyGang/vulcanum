use vulcanum_shared::api_types::SubmitResultRequest;

use crate::db::work_runs::queries::prs::InsertReviewResultParams;
use crate::models::work_runs::model::WorkRun;
use crate::services::work_runs::service::WorkRunsService;

impl WorkRunsService {
    pub(crate) async fn record_review_result(&self, run: &WorkRun, params: &SubmitResultRequest) {
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
                    review_url: None,
                    review_body: params.result_summary.as_deref(),
                    review_already_exists: false,
                },
            )
            .await
        {
            tracing::warn!(work_run_id = %run.id, error = %e, "failed to record review result");
        }
    }
}

#[must_use]
pub(crate) fn review_comment(run: &WorkRun, params: &SubmitResultRequest) -> String {
    let pr_url = run
        .review_target_pr_url
        .as_deref()
        .unwrap_or("the pull request");
    match params.result_summary.as_deref() {
        Some(summary) => format!("Review completed for {pr_url}: {summary}"),
        None => format!("Review completed for {pr_url}"),
    }
}
