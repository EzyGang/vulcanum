use uuid::Uuid;

use crate::db::work_runs::queries::implementation_followups::InsertFollowupRequestParams;
use crate::models::work_runs::errors::WorkRunsError;
use crate::models::work_runs::model::TaskPrTarget;
use crate::services::work_runs::service::github_commands::{
    response_options, select_project, GithubCommandAuthorization,
    GithubCommandAuthorizationRequest, GithubCommandResponseOptions, ProjectSelection,
};
use crate::services::work_runs::service::implementation_followup_request_state::{
    persisted_outcome, validate_persisted_request,
};
use crate::services::work_runs::service::WorkRunsService;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum ImplementationCommandError {
    Malformed,
    Ambiguous,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) struct GithubImplementationRequest<'a> {
    pub delivery_id: &'a str,
    pub installation_id: i64,
    pub comment_id: i64,
    pub sender_id: &'a str,
    pub single_user_mode: bool,
    pub repo_full_name: &'a str,
    pub pr_number: i64,
    pub pr_title: &'a str,
    pub project_selector: Option<&'a str>,
    pub request_body: Option<&'a str>,
    pub command_error: Option<ImplementationCommandError>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) enum GithubImplementationRequestOutcome {
    Spawned {
        team_id: Uuid,
        external_task_ref: String,
        work_run_id: Uuid,
        ticket_created: bool,
    },
    AlreadyActive {
        team_id: Uuid,
        external_task_ref: String,
        ticket_created: bool,
    },
    AmbiguousTickets {
        team_id: Uuid,
        external_task_refs: Vec<String>,
    },
    MalformedCommand {
        team_id: Uuid,
        error: ImplementationCommandError,
    },
    Unauthorized {
        team_id: Uuid,
    },
    UnknownInstallation,
    NoMatchingProject {
        team_id: Uuid,
    },
    ProjectSelectionRequired(GithubCommandResponseOptions),
    InvalidProjectSelection(GithubCommandResponseOptions),
}

impl WorkRunsService {
    pub(crate) async fn request_github_implementation(
        &self,
        request: GithubImplementationRequest<'_>,
    ) -> Result<GithubImplementationRequestOutcome, WorkRunsError> {
        let (team_id, projects) = match self
            .authorize_github_command(GithubCommandAuthorizationRequest {
                installation_id: request.installation_id,
                sender_id: request.sender_id,
                single_user_mode: request.single_user_mode,
                repo_full_name: request.repo_full_name,
            })
            .await?
        {
            GithubCommandAuthorization::Authorized { team_id, projects } => (team_id, projects),
            GithubCommandAuthorization::Unauthorized { team_id } => {
                return Ok(GithubImplementationRequestOutcome::Unauthorized { team_id });
            }
            GithubCommandAuthorization::UnknownInstallation => {
                return Ok(GithubImplementationRequestOutcome::UnknownInstallation);
            }
            GithubCommandAuthorization::NoMatchingProject { team_id } => {
                return Ok(GithubImplementationRequestOutcome::NoMatchingProject { team_id });
            }
        };
        if let Some(error) = request.command_error {
            return Ok(GithubImplementationRequestOutcome::MalformedCommand { team_id, error });
        }
        let request_body = match request.request_body {
            Some(body) if !body.trim().is_empty() => body,
            Some(_) | None => {
                return Ok(GithubImplementationRequestOutcome::MalformedCommand {
                    team_id,
                    error: ImplementationCommandError::Malformed,
                });
            }
        };
        let options = response_options(team_id, &projects);
        let selected = match select_project(request.project_selector, &projects, &[]) {
            ProjectSelection::Selected(project) => project,
            ProjectSelection::Required => {
                return Ok(GithubImplementationRequestOutcome::ProjectSelectionRequired(options));
            }
            ProjectSelection::Disabled | ProjectSelection::Invalid => {
                return Ok(GithubImplementationRequestOutcome::InvalidProjectSelection(
                    options,
                ));
            }
        };
        let normalized_repo = request.repo_full_name.to_ascii_lowercase();
        let persisted = self
            .work_runs_repo
            .insert_or_get_github_implementation_followup(
                &self.db,
                InsertFollowupRequestParams {
                    delivery_id: request.delivery_id,
                    github_installation_id: request.installation_id,
                    repo_full_name: &normalized_repo,
                    pr_number: request.pr_number,
                    comment_id: request.comment_id,
                    sender_id: request.sender_id,
                    project_config_id: selected.id,
                    request_body,
                },
            )
            .await?;
        validate_persisted_request(&persisted, &request, selected.id, request_body)?;
        if let Some(outcome) = persisted_outcome(team_id, &persisted) {
            return Ok(outcome);
        }

        let mut targets = self
            .work_runs_repo
            .list_task_pr_targets_for_pull_request(
                &self.db,
                request.installation_id,
                &normalized_repo,
                request.pr_number,
            )
            .await?
            .into_iter()
            .filter(|target| target.project_config_id == selected.id)
            .collect::<Vec<TaskPrTarget>>();
        if targets.len() > 1 {
            let external_task_refs = targets
                .into_iter()
                .map(|target| target.external_task_ref)
                .collect::<Vec<String>>();
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
                external_task_refs,
            });
        }
        let target = targets.pop();
        self.resolve_and_spawn_github_implementation_followup(
            team_id,
            selected,
            target,
            &normalized_repo,
            request,
            request_body,
        )
        .await
    }
}
