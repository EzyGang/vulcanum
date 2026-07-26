use async_trait::async_trait;

use crate::models::project_configs::model::ProjectConfig;
use crate::models::provider_configs::model::IntegrationProvider;
use crate::models::providers::model::{CreateIntegrationTaskInput, IntegrationTask};
use crate::models::work_runs::errors::WorkRunsError;
use crate::services::providers::client::IntegrationClient;
use crate::util::github::github_pr_url;

#[async_trait]
pub(crate) trait ImplementationFollowupTicketClient: Send + Sync {
    async fn find_existing(
        &self,
        provider: &IntegrationProvider,
        project: &ProjectConfig,
        marker: &str,
    ) -> Result<Option<IntegrationTask>, WorkRunsError>;

    async fn fetch(
        &self,
        provider: &IntegrationProvider,
        external_task_ref: &str,
    ) -> Result<IntegrationTask, WorkRunsError>;

    async fn create(
        &self,
        provider: &IntegrationProvider,
        input: CreateIntegrationTaskInput,
    ) -> Result<IntegrationTask, WorkRunsError>;

    async fn update_description(
        &self,
        provider: &IntegrationProvider,
        external_task_ref: &str,
        description: &str,
    ) -> Result<(), WorkRunsError>;

    async fn ensure_blocks(
        &self,
        provider: &IntegrationProvider,
        source_task_ref: &str,
        target_task_ref: &str,
    ) -> Result<(), WorkRunsError>;
}

pub(crate) struct IntegrationImplementationFollowupTicketClient;

#[async_trait]
impl ImplementationFollowupTicketClient for IntegrationImplementationFollowupTicketClient {
    async fn find_existing(
        &self,
        provider: &IntegrationProvider,
        project: &ProjectConfig,
        marker: &str,
    ) -> Result<Option<IntegrationTask>, WorkRunsError> {
        let board = IntegrationClient::from_provider(provider)
            .fetch_board(&project.external_project_id)
            .await?;
        Ok(board
            .columns
            .into_iter()
            .flat_map(|column| column.tasks)
            .find(|task| {
                task.description
                    .as_deref()
                    .is_some_and(|description| description.contains(marker))
            }))
    }

    async fn fetch(
        &self,
        provider: &IntegrationProvider,
        external_task_ref: &str,
    ) -> Result<IntegrationTask, WorkRunsError> {
        IntegrationClient::from_provider(provider)
            .fetch_task(external_task_ref)
            .await
            .map_err(WorkRunsError::from)
    }

    async fn create(
        &self,
        provider: &IntegrationProvider,
        input: CreateIntegrationTaskInput,
    ) -> Result<IntegrationTask, WorkRunsError> {
        IntegrationClient::from_provider(provider)
            .create_task(input)
            .await
            .map_err(WorkRunsError::from)
    }

    async fn update_description(
        &self,
        provider: &IntegrationProvider,
        external_task_ref: &str,
        description: &str,
    ) -> Result<(), WorkRunsError> {
        IntegrationClient::from_provider(provider)
            .update_task_description(external_task_ref, description)
            .await
            .map_err(WorkRunsError::from)
    }

    async fn ensure_blocks(
        &self,
        provider: &IntegrationProvider,
        source_task_ref: &str,
        target_task_ref: &str,
    ) -> Result<(), WorkRunsError> {
        IntegrationClient::from_provider(provider)
            .ensure_task_blocks(source_task_ref, target_task_ref)
            .await
            .map_err(WorkRunsError::from)
    }
}

#[must_use]
pub(crate) fn followup_ticket_input(
    project: &ProjectConfig,
    repo_full_name: &str,
    pr_number: i64,
    pr_title: &str,
    delivery_id: &str,
    request_body: &str,
) -> CreateIntegrationTaskInput {
    let pr_url = github_pr_url(repo_full_name, pr_number);
    CreateIntegrationTaskInput {
        project_id: project.external_project_id.clone(),
        title: format!("Follow up PR #{pr_number}: {pr_title}"),
        body: format!(
            "Pull request: {pr_url}\n\n{}\n\n{}",
            followup_request_block(delivery_id, &pr_url, request_body),
            followup_ticket_marker(project.id, repo_full_name, pr_number),
        ),
        status: project.progress_column.clone(),
        priority: "low".to_owned(),
    }
}

#[must_use]
pub(crate) fn followup_ticket_marker(
    project_config_id: uuid::Uuid,
    repo_full_name: &str,
    pr_number: i64,
) -> String {
    format!(
        "<!-- vulcanum:github-implementation-ticket:{project_config_id}:{repo_full_name}#{pr_number} -->"
    )
}

#[must_use]
pub(crate) fn followup_request_marker(delivery_id: &str) -> String {
    format!("<!-- vulcanum:github-implementation-followup:{delivery_id} -->")
}

#[must_use]
pub(crate) fn followup_request_block(
    delivery_id: &str,
    pr_url: &str,
    request_body: &str,
) -> String {
    format!(
        "{}\n## GitHub PR follow-up request\n\nSource: {pr_url}\n\n---\n{request_body}\n---",
        followup_request_marker(delivery_id),
    )
}
