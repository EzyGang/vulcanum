use crate::models::github_app::errors::GithubAppError;
use crate::models::work_runs::errors::WorkRunsError;
use crate::services::github_app::service::webhooks::implementation_responses::{
    respond_to_implementation_outcome, respond_to_provider_failure,
};
use crate::services::github_app::service::webhooks::processing::DeliveryDisposition;
use crate::services::github_app::service::webhooks::{
    required_delivery_field, GithubWebhookService,
};
use crate::services::github_app::webhook_store::{
    GithubWebhookCommandError, GithubWebhookDelivery,
};
use crate::services::work_runs::service::request_github_implementation::{
    GithubImplementationRequest, ImplementationCommandError,
};

impl GithubWebhookService {
    pub(super) async fn process_implementation_requested(
        &self,
        delivery: &GithubWebhookDelivery,
    ) -> Result<DeliveryDisposition, GithubAppError> {
        let sender_id = required_delivery_field(&delivery.sender_id, "sender_id")?;
        let pr_title = required_delivery_field(&delivery.pr_title, "pr_title")?;
        let comment_id = delivery.comment_id.ok_or_else(|| {
            GithubAppError::Redis("implementation webhook omitted comment_id".to_owned())
        })?;
        let outcome = match self
            .work_runs
            .request_github_implementation(GithubImplementationRequest {
                delivery_id: &delivery.delivery_id,
                installation_id: delivery.installation_id,
                comment_id,
                sender_id,
                single_user_mode: self.single_user_mode,
                repo_full_name: &delivery.repo_full_name,
                pr_number: delivery.pr_number,
                pr_title,
                project_selector: delivery.project_selector.as_deref(),
                request_body: delivery.request_body.as_deref(),
                command_error: delivery.command_error.map(command_error),
            })
            .await
        {
            Ok(outcome) => outcome,
            Err(error) => {
                if provider_failure(&error) {
                    let team_id = self
                        .work_runs
                        .github
                        .repo
                        .find_team_id_by_github_installation(
                            &self.work_runs.db,
                            delivery.installation_id,
                        )
                        .await?;
                    if let Some(team_id) = team_id {
                        respond_to_provider_failure(
                            self.comment_writer.as_ref(),
                            team_id,
                            &delivery.delivery_id,
                            delivery.installation_id,
                            &delivery.repo_full_name,
                            delivery.pr_number,
                        )
                        .await?;
                    }
                }
                return Ok(DeliveryDisposition::Retry(error.to_string()));
            }
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
        match respond_to_implementation_outcome(
            self.comment_writer.as_ref(),
            app_slug,
            &delivery.delivery_id,
            delivery.installation_id,
            &delivery.repo_full_name,
            delivery.pr_number,
            delivery.request_body.as_deref(),
            &outcome,
        )
        .await
        {
            Ok(()) => Ok(DeliveryDisposition::Complete),
            Err(error) => Ok(DeliveryDisposition::Retry(error.to_string())),
        }
    }
}

fn command_error(error: GithubWebhookCommandError) -> ImplementationCommandError {
    match error {
        GithubWebhookCommandError::Malformed => ImplementationCommandError::Malformed,
        GithubWebhookCommandError::Ambiguous => ImplementationCommandError::Ambiguous,
    }
}

fn provider_failure(error: &WorkRunsError) -> bool {
    matches!(
        error,
        WorkRunsError::Provider(_)
            | WorkRunsError::ProviderConfig(_)
            | WorkRunsError::ProjectConfig(_)
    )
}
