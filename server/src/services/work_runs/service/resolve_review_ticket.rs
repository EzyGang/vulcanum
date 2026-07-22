use uuid::Uuid;

use crate::db::work_runs::queries::review_tickets::ReviewTicketReservation;
use crate::models::project_configs::errors::ProjectConfigsError;
use crate::models::project_configs::model::ProjectConfig;
use crate::models::provider_configs::model::IntegrationProvider;
use crate::models::work_runs::errors::WorkRunsError;
use crate::services::work_runs::service::WorkRunsService;

impl WorkRunsService {
    pub(crate) async fn resolve_github_review_ticket(
        &self,
        project: &ProjectConfig,
        normalized_repo: &str,
        pr_number: i64,
        pr_title: &str,
    ) -> Result<String, WorkRunsError> {
        let reservation = self
            .work_runs_repo
            .reserve_github_review_ticket(&self.db, project.id, normalized_repo, pr_number)
            .await?;
        let (token, recovering) = match reservation {
            ReviewTicketReservation::Ready(external_task_ref) => return Ok(external_task_ref),
            ReviewTicketReservation::Pending => {
                return Err(WorkRunsError::ReviewTicketCreationPending);
            }
            ReviewTicketReservation::Acquired { token, recovering } => (token, recovering),
        };

        let external_task_ref = match self
            .create_or_recover_review_ticket(
                project,
                normalized_repo,
                pr_number,
                pr_title,
                recovering,
            )
            .await
        {
            Ok(external_task_ref) => external_task_ref,
            Err(error) => {
                self.release_review_ticket_reservation(
                    project.id,
                    normalized_repo,
                    pr_number,
                    token,
                )
                .await;
                return Err(error);
            }
        };
        let finalized = self
            .work_runs_repo
            .finalize_github_review_ticket(
                &self.db,
                project.id,
                normalized_repo,
                pr_number,
                token,
                &external_task_ref,
            )
            .await?;
        if !finalized {
            return Err(WorkRunsError::ReviewTicketCreationPending);
        }

        Ok(external_task_ref)
    }

    async fn create_or_recover_review_ticket(
        &self,
        project: &ProjectConfig,
        normalized_repo: &str,
        pr_number: i64,
        pr_title: &str,
        recovering: bool,
    ) -> Result<String, WorkRunsError> {
        let provider = self.review_ticket_provider(project).await?;
        if recovering {
            let existing = self
                .review_ticket_creator
                .find_existing(&provider, project, normalized_repo, pr_number)
                .await?;
            if let Some(external_task_ref) = existing {
                return Ok(external_task_ref);
            }
        }

        self.review_ticket_creator
            .create(&provider, project, normalized_repo, pr_number, pr_title)
            .await
    }

    async fn review_ticket_provider(
        &self,
        project: &ProjectConfig,
    ) -> Result<IntegrationProvider, WorkRunsError> {
        let provider_id = project.provider_id.ok_or(ProjectConfigsError::NoProvider)?;
        self.providers_repo
            .find_by_id(&self.db, provider_id, project.team_id)
            .await
            .map_err(WorkRunsError::from)
    }

    async fn release_review_ticket_reservation(
        &self,
        project_config_id: Uuid,
        normalized_repo: &str,
        pr_number: i64,
        token: Uuid,
    ) {
        if let Err(error) = self
            .work_runs_repo
            .release_github_review_ticket_reservation(
                &self.db,
                project_config_id,
                normalized_repo,
                pr_number,
                token,
            )
            .await
        {
            tracing::error!(
                project_config_id = %project_config_id,
                repo_full_name = normalized_repo,
                pr_number,
                error = %error,
                "failed to release review ticket reservation"
            );
        }
    }
}
