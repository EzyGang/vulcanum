use vulcanum_shared::api_types::SubmitResultRequest;

use crate::services::work_runs::model::{WorkRun, WorkRunStatus, WorkRunType};
use crate::services::work_runs::repository::queries::prs::InsertReviewResultParams;
use crate::services::work_runs::repository::queries::InsertWorkRunParams;
use crate::services::work_runs::service::review_feedback::{
    review_fix_prompt, review_requires_implementation,
};
use crate::services::work_runs::service::WorkRunsService;
use crate::util::github::github_repo_url;

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
                    review_url: params.review_url.as_deref(),
                    review_body: params.review_body.as_deref(),
                    review_already_exists: params.review_already_exists,
                },
            )
            .await
        {
            tracing::warn!(work_run_id = %run.id, error = %e, "failed to record review result");
        }

        if params.review_already_exists {
            return;
        }

        let review_body = match params.review_body.as_deref() {
            Some(body) => body,
            None => return,
        };

        if !review_requires_implementation(review_body) {
            return;
        }

        self.spawn_review_fix_run(run, pr_url, repo, review_body)
            .await;
    }

    async fn spawn_review_fix_run(
        &self,
        run: &WorkRun,
        pr_url: &str,
        repo_full_name: &str,
        review_body: &str,
    ) {
        if repo_full_name.is_empty() {
            tracing::warn!(
                work_run_id = %run.id,
                pr_url = %pr_url,
                "skipping review fix run because review target repo is missing"
            );
            return;
        }

        let params = InsertWorkRunParams {
            team_id: run.team_id,
            external_task_ref: run.external_task_ref.clone(),
            project_config_id: run.project_config_id,
            prompt_text: review_fix_prompt(run, pr_url, review_body),
            repo_url: github_repo_url(repo_full_name),
            repo_full_names: vec![repo_full_name.to_owned()],
            agents_md: run.agents_md.clone(),
            status: WorkRunStatus::Pending,
            work_type: WorkRunType::Implementation,
            parent_work_run_id: Some(run.id),
            task_body: run.task_body.clone(),
            task_title: run.task_title.clone(),
            task_slug: run.task_slug.clone(),
            review_target_pr_url: Some(pr_url.to_owned()),
            review_target_repo_full_name: Some(repo_full_name.to_owned()),
        };

        match self
            .work_runs_repo
            .insert_work_run_if_not_active(&self.db, params)
            .await
        {
            Ok(true) => (),
            Ok(false) => tracing::debug!(
                work_run_id = %run.id,
                pr_url = %pr_url,
                "skipped review fix run because an active implementation run already exists"
            ),
            Err(e) => tracing::warn!(
                work_run_id = %run.id,
                pr_url = %pr_url,
                error = %e,
                "failed to enqueue review fix run"
            ),
        }
    }
}

#[must_use]
pub(crate) fn review_comment(run: &WorkRun, params: &SubmitResultRequest) -> String {
    let pr_url = run
        .review_target_pr_url
        .as_deref()
        .unwrap_or("the pull request");
    let prefix = match params.review_already_exists {
        true => "Review already existed",
        false => "Review posted",
    };

    match params.review_url.as_deref() {
        Some(review_url) => format!("{prefix} for {pr_url}: {review_url}"),
        None => format!("{prefix} for {pr_url}"),
    }
}
