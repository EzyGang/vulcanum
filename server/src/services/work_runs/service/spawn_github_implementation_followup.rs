use uuid::Uuid;

use crate::db::work_runs::queries::implementation_followups::FollowupTicketReservation;
use crate::db::work_runs::queries::InsertWorkRunParams;
use crate::models::project_configs::model::ProjectConfig;
use crate::models::work_runs::errors::WorkRunsError;
use crate::models::work_runs::model::{TaskPrTarget, WorkRunStatus, WorkRunType};
use crate::services::work_runs::service::request_github_implementation::{
    GithubImplementationRequest, GithubImplementationRequestOutcome,
};
use crate::services::work_runs::service::WorkRunsService;

impl WorkRunsService {
    #[allow(clippy::too_many_arguments)]
    pub(crate) async fn resolve_and_spawn_github_implementation_followup(
        &self,
        team_id: Uuid,
        selected: &ProjectConfig,
        target: Option<TaskPrTarget>,
        normalized_repo: &str,
        request: GithubImplementationRequest<'_>,
        request_body: &str,
    ) -> Result<GithubImplementationRequestOutcome, WorkRunsError> {
        let known_task_ref = target
            .as_ref()
            .map(|target| target.external_task_ref.as_str());
        let reservation = self
            .work_runs_repo
            .reserve_github_implementation_followup_ticket(
                &self.db,
                selected.id,
                normalized_repo,
                request.pr_number,
                known_task_ref,
                request.delivery_id,
            )
            .await?;
        let (token, reserved_task_ref) = match reservation {
            FollowupTicketReservation::Acquired {
                token,
                external_task_ref,
                ..
            } => (token, external_task_ref),
            FollowupTicketReservation::Pending => {
                return Err(WorkRunsError::ImplementationFollowupPending);
            }
        };
        if known_task_ref.is_some()
            && reserved_task_ref.as_deref().is_some()
            && known_task_ref != reserved_task_ref.as_deref()
        {
            self.work_runs_repo
                .release_github_implementation_followup_ticket(
                    &self.db,
                    selected.id,
                    normalized_repo,
                    request.pr_number,
                    token,
                )
                .await?;
            self.work_runs_repo
                .finish_github_implementation_followup(
                    &self.db,
                    request.delivery_id,
                    None,
                    None,
                    false,
                    "ambiguous_ticket",
                )
                .await?;
            return Ok(GithubImplementationRequestOutcome::AmbiguousTickets {
                team_id,
                external_task_refs: vec![
                    known_task_ref.unwrap_or_default().to_owned(),
                    reserved_task_ref.unwrap_or_default(),
                ],
            });
        }
        let resolved = self
            .resolve_github_implementation_followup_ticket(
                selected,
                normalized_repo,
                request.pr_number,
                request.pr_title,
                request.delivery_id,
                request_body,
                token,
                reserved_task_ref.as_deref().or(known_task_ref),
            )
            .await?;
        let inserted = self
            .work_runs_repo
            .insert_work_run_if_not_active(
                &self.db,
                InsertWorkRunParams {
                    team_id,
                    external_task_ref: resolved.task.id.clone(),
                    task_title: Some(resolved.task.title.clone()),
                    task_slug: target.and_then(|target| target.task_slug),
                    project_config_id: selected.id,
                    repo_full_names: vec![request.repo_full_name.to_owned()],
                    status: WorkRunStatus::Pending,
                    work_type: WorkRunType::Implementation,
                    parent_work_run_id: None,
                    review_target_pr_url: None,
                    review_target_repo_full_name: None,
                    github_installation_id: Some(request.installation_id),
                    github_delivery_id: Some(request.delivery_id.to_owned()),
                },
            )
            .await?;
        let work_run_id = self
            .work_runs_repo
            .find_id_by_github_delivery(&self.db, request.delivery_id)
            .await?;
        match (inserted, work_run_id) {
            (_, Some(work_run_id)) => {
                self.work_runs_repo
                    .finish_github_implementation_followup(
                        &self.db,
                        request.delivery_id,
                        Some(&resolved.task.id),
                        Some(work_run_id),
                        resolved.ticket_created,
                        "spawned",
                    )
                    .await?;
                Ok(GithubImplementationRequestOutcome::Spawned {
                    team_id,
                    external_task_ref: resolved.task.id,
                    work_run_id,
                    ticket_created: resolved.ticket_created,
                })
            }
            (false, None) => {
                self.work_runs_repo
                    .finish_github_implementation_followup(
                        &self.db,
                        request.delivery_id,
                        Some(&resolved.task.id),
                        None,
                        resolved.ticket_created,
                        "active_run",
                    )
                    .await?;
                Ok(GithubImplementationRequestOutcome::AlreadyActive {
                    team_id,
                    external_task_ref: resolved.task.id,
                    ticket_created: resolved.ticket_created,
                })
            }
            (true, None) => Err(WorkRunsError::NotFound),
        }
    }
}
