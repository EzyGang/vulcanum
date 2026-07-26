use sqlx::PgConnection;
use uuid::Uuid;

use crate::db::work_runs::queries::implementation_followups::FinishFollowupRequestParams;
use crate::db::work_runs::queries::InsertWorkRunParams;
use crate::models::project_configs::model::ProjectConfig;
use crate::models::providers::model::IntegrationTask;
use crate::models::work_runs::errors::WorkRunsError;
use crate::models::work_runs::model::{WorkRunStatus, WorkRunType};
use crate::services::work_runs::service::request_github_implementation::GithubImplementationRequest;
use crate::services::work_runs::service::WorkRunsService;
use crate::util::github::github_pr_url;

pub(crate) struct FinalizeFollowupRequest<'a> {
    pub team_id: Uuid,
    pub project: &'a ProjectConfig,
    pub normalized_repo: &'a str,
    pub github: GithubImplementationRequest<'a>,
    pub task: IntegrationTask,
    pub task_slug: Option<String>,
    pub token: Uuid,
    pub reused_mapped_ticket: bool,
    pub existing_work_run_id: Option<Uuid>,
}

pub(crate) struct FinalizedFollowup {
    pub external_task_ref: String,
    pub work_run_id: Uuid,
    pub ticket_created: bool,
    pub task_slug: Option<String>,
}

pub(crate) struct TerminalFollowupRequest<'a> {
    pub project: &'a ProjectConfig,
    pub normalized_repo: &'a str,
    pub pr_number: i64,
    pub token: Uuid,
    pub delivery_id: &'a str,
    pub external_task_ref: Option<&'a str>,
    pub outcome: &'a str,
    pub ambiguous_task_refs: &'a [String],
}

pub(crate) struct FollowupAvailabilityRequest<'a> {
    pub project_config_id: Uuid,
    pub external_task_ref: Option<&'a str>,
    pub delivery_id: &'a str,
}

pub(crate) struct FollowupRunAvailability {
    pub existing_work_run_id: Option<Uuid>,
    pub active: bool,
}

impl WorkRunsService {
    pub(crate) async fn github_followup_run_availability(
        &self,
        db: &mut PgConnection,
        request: FollowupAvailabilityRequest<'_>,
    ) -> Result<FollowupRunAvailability, WorkRunsError> {
        let existing_work_run_id = self
            .work_runs_repo
            .find_id_by_github_delivery(&mut *db, request.delivery_id)
            .await?;
        let active = match (existing_work_run_id, request.external_task_ref) {
            (None, Some(external_task_ref)) => {
                self.work_runs_repo
                    .lock_implementation_task(
                        &mut *db,
                        request.project_config_id,
                        external_task_ref,
                    )
                    .await?;
                self.work_runs_repo
                    .has_active_implementation(
                        &mut *db,
                        request.project_config_id,
                        external_task_ref,
                    )
                    .await?
            }
            (Some(_) | None, None) | (Some(_), Some(_)) => false,
        };

        Ok(FollowupRunAvailability {
            existing_work_run_id,
            active,
        })
    }

    pub(crate) async fn finalize_github_implementation_followup(
        &self,
        db: &mut PgConnection,
        request: FinalizeFollowupRequest<'_>,
    ) -> Result<FinalizedFollowup, WorkRunsError> {
        self.work_runs_repo
            .upsert_github_followup_task_pr(
                &mut *db,
                request.project.id,
                &request.task.id,
                &github_pr_url(request.normalized_repo, request.github.pr_number),
                request.normalized_repo,
                request.github.pr_number,
            )
            .await?;
        let work_run_id = match request.existing_work_run_id {
            Some(work_run_id) => work_run_id,
            None => {
                self.work_runs_repo
                    .insert_work_run_if_not_active(
                        &mut *db,
                        InsertWorkRunParams {
                            team_id: request.team_id,
                            external_task_ref: request.task.id.clone(),
                            task_title: Some(request.task.title.clone()),
                            task_slug: request.task_slug.clone(),
                            project_config_id: request.project.id,
                            repo_full_names: vec![request.github.repo_full_name.to_owned()],
                            status: WorkRunStatus::Pending,
                            work_type: WorkRunType::Implementation,
                            parent_work_run_id: None,
                            review_target_pr_url: None,
                            review_target_repo_full_name: None,
                            github_installation_id: Some(request.github.installation_id),
                            github_delivery_id: Some(request.github.delivery_id.to_owned()),
                        },
                    )
                    .await?;
                self.work_runs_repo
                    .find_id_by_github_delivery(&mut *db, request.github.delivery_id)
                    .await?
                    .ok_or(WorkRunsError::NotFound)?
            }
        };
        let (completed, created_by_delivery_id) = self
            .work_runs_repo
            .complete_github_implementation_followup_ticket(
                &mut *db,
                request.project.id,
                request.normalized_repo,
                request.github.pr_number,
                request.token,
                &request.task.id,
            )
            .await?;
        if !completed {
            return Err(WorkRunsError::ImplementationFollowupPending);
        }
        let ticket_created = !request.reused_mapped_ticket
            && created_by_delivery_id.as_deref() == Some(request.github.delivery_id);
        self.work_runs_repo
            .finish_github_implementation_followup(
                &mut *db,
                FinishFollowupRequestParams {
                    delivery_id: request.github.delivery_id,
                    external_task_ref: Some(&request.task.id),
                    work_run_id: Some(work_run_id),
                    ticket_created,
                    outcome: "spawned",
                    ambiguous_task_refs: &[],
                },
            )
            .await?;

        Ok(FinalizedFollowup {
            external_task_ref: request.task.id,
            work_run_id,
            ticket_created,
            task_slug: request.task_slug,
        })
    }

    pub(crate) async fn finish_followup_without_run(
        &self,
        db: &mut PgConnection,
        request: TerminalFollowupRequest<'_>,
    ) -> Result<(), WorkRunsError> {
        self.work_runs_repo
            .release_github_implementation_followup_ticket(
                &mut *db,
                request.project.id,
                request.normalized_repo,
                request.pr_number,
                request.token,
            )
            .await?;
        self.work_runs_repo
            .finish_github_implementation_followup(
                db,
                FinishFollowupRequestParams {
                    delivery_id: request.delivery_id,
                    external_task_ref: request.external_task_ref,
                    work_run_id: None,
                    ticket_created: false,
                    outcome: request.outcome,
                    ambiguous_task_refs: request.ambiguous_task_refs,
                },
            )
            .await
    }
}
