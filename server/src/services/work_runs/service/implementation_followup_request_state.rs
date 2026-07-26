use uuid::Uuid;

use crate::models::work_runs::errors::WorkRunsError;
use crate::models::work_runs::model::GithubImplementationFollowupRequest;
use crate::services::work_runs::service::request_github_implementation::{
    GithubImplementationRequest, GithubImplementationRequestOutcome,
};

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
        }),
        "active_run" => Some(GithubImplementationRequestOutcome::AlreadyActive {
            team_id,
            external_task_ref: persisted.external_task_ref.clone()?,
            ticket_created: persisted.ticket_created,
        }),
        "ambiguous_ticket" => Some(GithubImplementationRequestOutcome::AmbiguousTickets {
            team_id,
            external_task_refs: Vec::new(),
        }),
        "pending" => None,
        _ => None,
    }
}
