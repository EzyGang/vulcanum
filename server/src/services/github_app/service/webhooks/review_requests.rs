use crate::models::github_app::errors::GithubAppError;
use crate::services::github_app::service::webhooks::responses::respond_to_outcome;
use crate::services::github_app::service::webhooks::GithubWebhookService;
use crate::services::github_app::webhook_store::GithubWebhookDelivery;
use crate::services::work_runs::service::request_github_review::GithubReviewRequest;

impl GithubWebhookService {
    pub(super) async fn process_review_requested(
        &self,
        delivery: &GithubWebhookDelivery,
    ) -> Result<(), GithubAppError> {
        let sender_id = required(&delivery.sender_id, "sender_id")?;
        let pr_title = required(&delivery.pr_title, "pr_title")?;
        let outcome = match self
            .work_runs
            .request_github_review(GithubReviewRequest {
                delivery_id: &delivery.delivery_id,
                installation_id: delivery.installation_id,
                sender_id,
                repo_full_name: &delivery.repo_full_name,
                pr_number: delivery.pr_number,
                pr_title,
                project_selector: delivery.project_selector.as_deref(),
            })
            .await
        {
            Ok(outcome) => outcome,
            Err(error) => {
                self.store.retry(delivery, &error.to_string()).await?;
                return Ok(());
            }
        };
        let app_slug = self
            .app_slug
            .as_deref()
            .ok_or(GithubAppError::NotConfigured)?;
        match respond_to_outcome(
            self.comment_writer.as_ref(),
            app_slug,
            &delivery.delivery_id,
            delivery.installation_id,
            &delivery.repo_full_name,
            delivery.pr_number,
            &outcome,
        )
        .await
        {
            Ok(()) => self.store.complete(&delivery.delivery_id).await?,
            Err(error) => self.store.retry(delivery, &error.to_string()).await?,
        }
        Ok(())
    }
}

fn required<'a>(value: &'a Option<String>, field: &str) -> Result<&'a str, GithubAppError> {
    value
        .as_deref()
        .ok_or_else(|| GithubAppError::Redis(format!("review webhook omitted {field}")))
}
