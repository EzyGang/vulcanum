use uuid::Uuid;

use crate::db::work_runs::queries::implementation_followups::FollowupTicketReservation;
use crate::models::project_configs::model::ProjectConfig;
use crate::models::work_runs::errors::WorkRunsError;
use crate::services::work_runs::service::finalize_implementation_followup::{
    FinalizeFollowupRequest, FollowupAvailabilityRequest, TerminalFollowupRequest,
};
use crate::services::work_runs::service::implementation_followup_request_state::{
    select_followup_ticket, FollowupTicketSelection,
};
use crate::services::work_runs::service::request_github_implementation::{
    GithubImplementationRequest, GithubImplementationRequestOutcome,
};
use crate::services::work_runs::service::resolve_implementation_followup::FollowupTicketRequest;
use crate::services::work_runs::service::WorkRunsService;

impl WorkRunsService {
    pub(crate) async fn resolve_and_spawn_github_implementation_followup(
        &self,
        team_id: Uuid,
        selected: &ProjectConfig,
        normalized_repo: &str,
        request: GithubImplementationRequest<'_>,
        request_body: &str,
    ) -> Result<GithubImplementationRequestOutcome, WorkRunsError> {
        let reservation = self
            .work_runs_repo
            .reserve_github_implementation_followup_ticket(
                &self.db,
                selected.id,
                normalized_repo,
                request.pr_number,
                None,
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

        let review_task_ref = self
            .work_runs_repo
            .find_github_review_ticket(&self.db, selected.id, normalized_repo, request.pr_number)
            .await?;

        let mut transaction = self.db.begin().await?;
        self.work_runs_repo
            .lock_task_pr_target(
                &mut transaction,
                selected.id,
                normalized_repo,
                request.pr_number,
            )
            .await?;
        let targets = self
            .work_runs_repo
            .list_task_pr_targets_for_pull_request(
                &mut *transaction,
                request.installation_id,
                normalized_repo,
                request.pr_number,
            )
            .await?
            .into_iter()
            .filter(|target| target.project_config_id == selected.id)
            .collect();
        let target = match select_followup_ticket(request.ticket_selector, targets) {
            FollowupTicketSelection::Selected(target) => target,
            FollowupTicketSelection::Ambiguous(external_task_refs) => {
                self.finish_followup_without_run(
                    &mut transaction,
                    TerminalFollowupRequest {
                        project: selected,
                        normalized_repo,
                        pr_number: request.pr_number,
                        token,
                        delivery_id: request.delivery_id,
                        external_task_ref: None,
                        outcome: "ambiguous_ticket",
                        ambiguous_task_refs: &external_task_refs,
                    },
                )
                .await?;
                transaction.commit().await?;
                return Ok(GithubImplementationRequestOutcome::AmbiguousTickets {
                    team_id,
                    project_config_id: selected.id,
                    external_task_refs,
                });
            }
            FollowupTicketSelection::Invalid(external_task_refs) => {
                self.finish_followup_without_run(
                    &mut transaction,
                    TerminalFollowupRequest {
                        project: selected,
                        normalized_repo,
                        pr_number: request.pr_number,
                        token,
                        delivery_id: request.delivery_id,
                        external_task_ref: None,
                        outcome: "invalid_ticket",
                        ambiguous_task_refs: &external_task_refs,
                    },
                )
                .await?;
                transaction.commit().await?;
                return Ok(GithubImplementationRequestOutcome::InvalidTicketSelection {
                    team_id,
                    project_config_id: selected.id,
                    external_task_refs,
                });
            }
        };
        let reused_mapped_ticket = target.is_some();
        let mapped_task_slug = target.as_ref().and_then(|target| target.task_slug.clone());
        let external_task_ref = target
            .as_ref()
            .map(|target| target.external_task_ref.as_str())
            .or(reserved_task_ref.as_deref());

        let availability = self
            .github_followup_run_availability(
                &mut transaction,
                FollowupAvailabilityRequest {
                    project_config_id: selected.id,
                    external_task_ref,
                    delivery_id: request.delivery_id,
                },
            )
            .await?;
        if availability.active {
            let active_task_ref = external_task_ref.ok_or(WorkRunsError::NotFound)?;
            self.finish_followup_without_run(
                &mut transaction,
                TerminalFollowupRequest {
                    project: selected,
                    normalized_repo,
                    pr_number: request.pr_number,
                    token,
                    delivery_id: request.delivery_id,
                    outcome: "active_run",
                    external_task_ref,
                    ambiguous_task_refs: &[],
                },
            )
            .await?;
            transaction.commit().await?;
            return Ok(GithubImplementationRequestOutcome::AlreadyActive {
                team_id,
                external_task_ref: active_task_ref.to_owned(),
                ticket_created: false,
                task_slug: mapped_task_slug,
            });
        }
        let existing_work_run_id = availability.existing_work_run_id;

        let task = self
            .resolve_github_implementation_followup_ticket(FollowupTicketRequest {
                project: selected,
                normalized_repo,
                pr_number: request.pr_number,
                pr_title: request.pr_title,
                delivery_id: request.delivery_id,
                request_body,
                token,
                external_task_ref,
                review_task_ref: review_task_ref.as_deref(),
            })
            .await?;
        let task_slug = mapped_task_slug.or_else(|| task.slug());
        self.work_runs_repo
            .lock_implementation_task(&mut transaction, selected.id, &task.id)
            .await?;
        let finalized = self
            .finalize_github_implementation_followup(
                &mut transaction,
                FinalizeFollowupRequest {
                    team_id,
                    project: selected,
                    normalized_repo,
                    github: request,
                    task,
                    task_slug,
                    token,
                    reused_mapped_ticket,
                    existing_work_run_id,
                },
            )
            .await?;
        transaction.commit().await?;

        Ok(GithubImplementationRequestOutcome::Spawned {
            team_id,
            external_task_ref: finalized.external_task_ref,
            work_run_id: finalized.work_run_id,
            ticket_created: finalized.ticket_created,
            task_slug: finalized.task_slug,
        })
    }
}
