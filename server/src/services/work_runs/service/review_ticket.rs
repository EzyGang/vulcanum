use async_trait::async_trait;

use crate::models::project_configs::model::ProjectConfig;
use crate::models::provider_configs::model::IntegrationProvider;
use crate::models::providers::model::CreateIntegrationTaskInput;
use crate::models::work_runs::errors::WorkRunsError;
use crate::services::providers::client::IntegrationClient;
use crate::util::github::github_pr_url;

#[async_trait]
pub(crate) trait ReviewTicketCreator: Send + Sync {
    async fn find_existing(
        &self,
        provider: &IntegrationProvider,
        project: &ProjectConfig,
        repo_full_name: &str,
        pr_number: i64,
    ) -> Result<Option<String>, WorkRunsError>;

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
    async fn find_existing(
        &self,
        provider: &IntegrationProvider,
        project: &ProjectConfig,
        repo_full_name: &str,
        pr_number: i64,
    ) -> Result<Option<String>, WorkRunsError> {
        let marker = review_ticket_marker(project.id, repo_full_name, pr_number);
        let board = IntegrationClient::from_provider(provider)
            .fetch_board(&project.external_project_id)
            .await?;
        let task_id = board
            .columns
            .into_iter()
            .flat_map(|column| column.tasks)
            .find(|task| {
                task.description
                    .as_deref()
                    .is_some_and(|description| description.contains(&marker))
            })
            .map(|task| task.id);

        Ok(task_id)
    }

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
            "Review pull request: {}\n\n{}",
            github_pr_url(repo_full_name, pr_number),
            review_ticket_marker(project.id, repo_full_name, pr_number)
        ),
        status: project.review_column.clone(),
        priority: "low".to_owned(),
    }
}

#[must_use]
pub(crate) fn review_ticket_marker(
    project_config_id: uuid::Uuid,
    repo_full_name: &str,
    pr_number: i64,
) -> String {
    format!(
        "<!-- vulcanum:github-review-ticket:{project_config_id}:{repo_full_name}#{pr_number} -->"
    )
}

#[must_use]
pub(crate) fn review_ticket_title(pr_number: i64, pr_title: &str) -> String {
    format!("Review PR #{pr_number}: {pr_title}")
}
