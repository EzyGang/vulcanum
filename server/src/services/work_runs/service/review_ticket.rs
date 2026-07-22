use async_trait::async_trait;

use crate::models::project_configs::model::ProjectConfig;
use crate::models::provider_configs::model::IntegrationProvider;
use crate::models::providers::model::CreateIntegrationTaskInput;
use crate::models::work_runs::errors::WorkRunsError;
use crate::services::providers::client::IntegrationClient;
use crate::util::github::github_pr_url;

#[async_trait]
pub(crate) trait ReviewTicketCreator: Send + Sync {
    async fn create(
        &self,
        provider: &IntegrationProvider,
        project: &ProjectConfig,
        repo_full_name: &str,
        pr_number: i64,
        pr_title: &str,
    ) -> Result<String, WorkRunsError>;
}

pub(crate) struct IntegrationReviewTicketCreator;

#[async_trait]
impl ReviewTicketCreator for IntegrationReviewTicketCreator {
    async fn create(
        &self,
        provider: &IntegrationProvider,
        project: &ProjectConfig,
        repo_full_name: &str,
        pr_number: i64,
        pr_title: &str,
    ) -> Result<String, WorkRunsError> {
        let task = IntegrationClient::from_provider(provider)
            .create_task(review_ticket_input(
                project,
                repo_full_name,
                pr_number,
                pr_title,
            ))
            .await?;

        Ok(task.id)
    }
}

#[must_use]
pub(crate) fn review_ticket_input(
    project: &ProjectConfig,
    repo_full_name: &str,
    pr_number: i64,
    pr_title: &str,
) -> CreateIntegrationTaskInput {
    CreateIntegrationTaskInput {
        project_id: project.external_project_id.clone(),
        title: review_ticket_title(pr_number, pr_title),
        body: format!(
            "Review pull request: {}",
            github_pr_url(repo_full_name, pr_number)
        ),
        status: project.review_column.clone(),
        priority: "low".to_owned(),
    }
}

#[must_use]
pub(crate) fn review_ticket_title(pr_number: i64, pr_title: &str) -> String {
    format!("Review PR #{pr_number}: {pr_title}")
}
