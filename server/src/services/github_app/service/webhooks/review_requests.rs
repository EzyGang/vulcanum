use crate::models::github_app::errors::GithubAppError;
use crate::services::github_app::service::webhooks::processing::DeliveryDisposition;
use crate::services::github_app::service::webhooks::responses::respond_to_outcome;
use crate::services::github_app::service::webhooks::{
    required_delivery_field, GithubWebhookService,
};
use crate::services::github_app::webhook_store::GithubWebhookDelivery;
use crate::services::work_runs::service::request_github_review::GithubReviewRequest;

impl GithubWebhookService {
    pub(super) async fn process_review_requested(
        &self,
        delivery: &GithubWebhookDelivery,
    ) -> Result<DeliveryDisposition, GithubAppError> {
        let sender_id = required_delivery_field(&delivery.sender_id, "sender_id")?;
        let pr_title = required_delivery_field(&delivery.pr_title, "pr_title")?;
        let comment_id = delivery
            .comment_id
            .ok_or_else(|| GithubAppError::Redis("review webhook omitted comment_id".to_owned()))?;
        let outcome = match self
            .work_runs
            .request_github_review(GithubReviewRequest {
                delivery_id: &delivery.delivery_id,
                installation_id: delivery.installation_id,
                sender_id,
                single_user_mode: self.single_user_mode,
                repo_full_name: &delivery.repo_full_name,
                pr_number: delivery.pr_number,
                pr_title,
                project_selector: delivery.project_selector.as_deref(),
            })
            .await
        {
            Ok(outcome) => outcome,
            Err(error) => return Ok(DeliveryDisposition::Retry(error.to_string())),
        };
        let app_slug = self
            .app_slug
            .as_deref()
            .ok_or(GithubAppError::NotConfigured)?;
        if let Err(error) = self
            .comment_writer
            .react_to_comment(
                delivery.installation_id,
                &delivery.repo_full_name,
                comment_id,
            )
            .await
        {
            return Ok(DeliveryDisposition::Retry(error.to_string()));
        }
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
            Ok(()) => Ok(DeliveryDisposition::Complete),
            Err(error) => Ok(DeliveryDisposition::Retry(error.to_string())),
        }
    }
}
