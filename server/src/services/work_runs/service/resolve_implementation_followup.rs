use uuid::Uuid;

use crate::models::project_configs::errors::ProjectConfigsError;
use crate::models::project_configs::model::ProjectConfig;
use crate::models::provider_configs::model::IntegrationProvider;
use crate::models::providers::model::IntegrationTask;
use crate::models::work_runs::errors::WorkRunsError;
use crate::services::work_runs::service::implementation_followup_ticket::{
    followup_request_block, followup_request_marker, followup_ticket_input, followup_ticket_marker,
};
use crate::services::work_runs::service::WorkRunsService;
use crate::util::github::github_pr_url;

pub(crate) struct FollowupTicketRequest<'a> {
    pub project: &'a ProjectConfig,
    pub normalized_repo: &'a str,
    pub pr_number: i64,
    pub pr_title: &'a str,
    pub delivery_id: &'a str,
    pub request_body: &'a str,
    pub token: Uuid,
    pub external_task_ref: Option<&'a str>,
    pub review_task_ref: Option<&'a str>,
}

impl WorkRunsService {
    pub(crate) async fn resolve_github_implementation_followup_ticket(
        &self,
        request: FollowupTicketRequest<'_>,
    ) -> Result<IntegrationTask, WorkRunsError> {
        let operation = self.apply_github_implementation_followup(&request);
        match self
            .run_with_implementation_followup_heartbeat(
                request.project.id,
                request.normalized_repo,
                request.pr_number,
                request.token,
                operation,
            )
            .await
        {
            Ok(task) => Ok(task),
            Err(error) => {
                self.work_runs_repo
                    .release_github_implementation_followup_ticket(
                        &self.db,
                        request.project.id,
                        request.normalized_repo,
                        request.pr_number,
                        request.token,
                    )
                    .await?;
                Err(error)
            }
        }
    }

    async fn apply_github_implementation_followup(
        &self,
        request: &FollowupTicketRequest<'_>,
    ) -> Result<IntegrationTask, WorkRunsError> {
        let provider = self
            .implementation_followup_provider(request.project)
            .await?;
        let ticket_marker = followup_ticket_marker(
            request.project.id,
            request.normalized_repo,
            request.pr_number,
        );
        let mut task = match request.external_task_ref {
            Some(external_task_ref) => {
                self.implementation_followup_ticket_client
                    .fetch(&provider, external_task_ref)
                    .await?
            }
            None => match self
                .implementation_followup_ticket_client
                .find_existing(&provider, request.project, &ticket_marker)
                .await?
            {
                Some(task) => task,
                None => {
                    self.implementation_followup_ticket_client
                        .create(
                            &provider,
                            followup_ticket_input(
                                request.project,
                                request.normalized_repo,
                                request.pr_number,
                                request.pr_title,
                                request.delivery_id,
                                request.request_body,
                            ),
                        )
                        .await?
                }
            },
        };
        let request_marker = followup_request_marker(request.delivery_id);
        if !task
            .description
            .as_deref()
            .is_some_and(|description| description.contains(&request_marker))
        {
            let request_block = followup_request_block(
                request.delivery_id,
                &github_pr_url(request.normalized_repo, request.pr_number),
                request.request_body,
            );
            let description = match task.description.as_deref() {
                Some(description) if !description.is_empty() => {
                    format!("{description}\n\n{request_block}")
                }
                Some(_) | None => request_block,
            };
            self.implementation_followup_ticket_client
                .update_description(&provider, &task.id, &description)
                .await?;
            task.description = Some(description);
        }

        match request.review_task_ref {
            Some(review_task_ref) if review_task_ref != task.id.as_str() => {
                self.implementation_followup_ticket_client
                    .ensure_blocks(&provider, &task.id, review_task_ref)
                    .await?;
            }
            Some(_) | None => (),
        }

        Ok(task)
    }

    async fn implementation_followup_provider(
        &self,
        project: &ProjectConfig,
    ) -> Result<IntegrationProvider, WorkRunsError> {
        let provider_id = project.provider_id.ok_or(ProjectConfigsError::NoProvider)?;
        self.providers_repo
            .find_by_id(&self.db, provider_id, project.team_id)
            .await
            .map_err(WorkRunsError::from)
    }
}
