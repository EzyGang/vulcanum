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

pub(crate) struct ResolvedFollowupTicket {
    pub task: IntegrationTask,
    pub ticket_created: bool,
}

impl WorkRunsService {
    #[allow(clippy::too_many_arguments)]
    pub(crate) async fn resolve_github_implementation_followup_ticket(
        &self,
        project: &ProjectConfig,
        normalized_repo: &str,
        pr_number: i64,
        pr_title: &str,
        delivery_id: &str,
        request_body: &str,
        token: Uuid,
        external_task_ref: Option<&str>,
    ) -> Result<ResolvedFollowupTicket, WorkRunsError> {
        let operation = self.apply_github_implementation_followup(
            project,
            normalized_repo,
            pr_number,
            pr_title,
            delivery_id,
            request_body,
            external_task_ref,
        );
        let task = match self
            .run_with_implementation_followup_heartbeat(
                project.id,
                normalized_repo,
                pr_number,
                token,
                operation,
            )
            .await
        {
            Ok(task) => task,
            Err(error) => {
                self.work_runs_repo
                    .release_github_implementation_followup_ticket(
                        &self.db,
                        project.id,
                        normalized_repo,
                        pr_number,
                        token,
                    )
                    .await?;
                return Err(error);
            }
        };
        let (completed, created_by_delivery_id) = self
            .work_runs_repo
            .complete_github_implementation_followup_ticket(
                &self.db,
                project.id,
                normalized_repo,
                pr_number,
                token,
                &task.id,
            )
            .await?;
        if !completed {
            return Err(WorkRunsError::ImplementationFollowupPending);
        }
        Ok(ResolvedFollowupTicket {
            ticket_created: created_by_delivery_id.as_deref() == Some(delivery_id),
            task,
        })
    }

    #[allow(clippy::too_many_arguments)]
    async fn apply_github_implementation_followup(
        &self,
        project: &ProjectConfig,
        normalized_repo: &str,
        pr_number: i64,
        pr_title: &str,
        delivery_id: &str,
        request_body: &str,
        external_task_ref: Option<&str>,
    ) -> Result<IntegrationTask, WorkRunsError> {
        let provider = self.implementation_followup_provider(project).await?;
        let ticket_marker = followup_ticket_marker(project.id, normalized_repo, pr_number);
        let mut task = match external_task_ref {
            Some(external_task_ref) => {
                self.implementation_followup_ticket_client
                    .fetch(&provider, external_task_ref)
                    .await?
            }
            None => match self
                .implementation_followup_ticket_client
                .find_existing(&provider, project, &ticket_marker)
                .await?
            {
                Some(task) => task,
                None => {
                    self.implementation_followup_ticket_client
                        .create(
                            &provider,
                            followup_ticket_input(
                                project,
                                normalized_repo,
                                pr_number,
                                pr_title,
                                delivery_id,
                                request_body,
                            ),
                        )
                        .await?
                }
            },
        };
        let request_marker = followup_request_marker(delivery_id);
        if !task
            .description
            .as_deref()
            .is_some_and(|description| description.contains(&request_marker))
        {
            let request_block = followup_request_block(
                delivery_id,
                &github_pr_url(normalized_repo, pr_number),
                request_body,
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
        self.work_runs_repo
            .upsert_github_followup_task_pr(
                &self.db,
                project.id,
                &task.id,
                &github_pr_url(normalized_repo, pr_number),
                normalized_repo,
                pr_number,
            )
            .await?;

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
