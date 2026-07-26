use uuid::Uuid;

use crate::models::work_runs::errors::WorkRunsError;
use crate::models::work_runs::model::{GithubImplementationFollowupRequest, TaskPrTarget};
use crate::services::work_runs::service::request_github_implementation::{
    GithubImplementationRequest, GithubImplementationRequestOutcome,
};

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) enum FollowupTicketSelection {
    Selected(Option<TaskPrTarget>),
    Ambiguous(Vec<String>),
    Invalid(Vec<String>),
}

pub(super) fn validate_persisted_request(
    persisted: &GithubImplementationFollowupRequest,
    request: &GithubImplementationRequest<'_>,
    project_config_id: Uuid,
    request_body: &str,
) -> Result<(), WorkRunsError> {
    let matches = persisted.github_installation_id == request.installation_id
        && persisted
            .repo_full_name
            .eq_ignore_ascii_case(request.repo_full_name)
        && persisted.pr_number == request.pr_number
        && persisted.comment_id == request.comment_id
        && persisted.sender_id == request.sender_id
        && persisted.project_config_id == project_config_id
        && persisted.ticket_selector.as_deref() == request.ticket_selector
        && persisted.request_body == request_body;
    match matches {
        true => Ok(()),
        false => Err(WorkRunsError::GithubDeliveryConflict),
    }
}

pub(super) fn persisted_outcome(
    team_id: Uuid,
    persisted: &GithubImplementationFollowupRequest,
) -> Option<GithubImplementationRequestOutcome> {
    match persisted.outcome.as_str() {
        "spawned" => Some(GithubImplementationRequestOutcome::Spawned {
            team_id,
            external_task_ref: persisted.external_task_ref.clone()?,
            work_run_id: persisted.work_run_id?,
            ticket_created: persisted.ticket_created,
            task_slug: None,
        }),
        "active_run" => Some(GithubImplementationRequestOutcome::AlreadyActive {
            team_id,
            external_task_ref: persisted.external_task_ref.clone()?,
            ticket_created: persisted.ticket_created,
            task_slug: None,
        }),
        "ambiguous_ticket" => Some(GithubImplementationRequestOutcome::AmbiguousTickets {
            team_id,
            project_config_id: persisted.project_config_id,
            external_task_refs: persisted.ambiguous_task_refs.clone(),
        }),
        "invalid_ticket" => Some(GithubImplementationRequestOutcome::InvalidTicketSelection {
            team_id,
            project_config_id: persisted.project_config_id,
            external_task_refs: persisted.ambiguous_task_refs.clone(),
        }),
        "pending" => None,
        _ => None,
    }
}

#[must_use]
pub(crate) fn select_followup_ticket(
    selector: Option<&str>,
    mut targets: Vec<TaskPrTarget>,
) -> FollowupTicketSelection {
    match selector {
        Some(selector) => match targets
            .iter()
            .position(|target| target.external_task_ref == selector)
        {
            Some(index) => FollowupTicketSelection::Selected(Some(targets.swap_remove(index))),
            None => FollowupTicketSelection::Invalid(task_refs(targets)),
        },
        None => match targets.len() {
            0 => FollowupTicketSelection::Selected(None),
            1 => FollowupTicketSelection::Selected(targets.pop()),
            _ => FollowupTicketSelection::Ambiguous(task_refs(targets)),
        },
    }
}

fn task_refs(targets: Vec<TaskPrTarget>) -> Vec<String> {
    targets
        .into_iter()
        .map(|target| target.external_task_ref)
        .collect()
}
