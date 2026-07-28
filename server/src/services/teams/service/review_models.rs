use crate::models::teams::errors::TeamsError;
use crate::models::teams::model::{Team, UpdateTeamRequest};

pub(super) fn validate_review_model_pairs(
    team: &Team,
    params: &UpdateTeamRequest,
) -> Result<(), TeamsError> {
    validate_pair(
        params
            .review_primary_model_provider_key
            .as_ref()
            .map_or(team.review_primary_model_provider_key.as_deref(), |value| {
                value.as_deref()
            }),
        params
            .review_primary_model_id
            .as_ref()
            .map_or(team.review_primary_model_id.as_deref(), |value| {
                value.as_deref()
            }),
        "Review primary model",
    )?;
    validate_pair(
        params
            .review_small_model_provider_key
            .as_ref()
            .map_or(team.review_small_model_provider_key.as_deref(), |value| {
                value.as_deref()
            }),
        params
            .review_small_model_id
            .as_ref()
            .map_or(team.review_small_model_id.as_deref(), |value| {
                value.as_deref()
            }),
        "Review small model",
    )
}

fn validate_pair(
    provider_key: Option<&str>,
    model_id: Option<&str>,
    label: &str,
) -> Result<(), TeamsError> {
    if provider_key.is_some() == model_id.is_some() {
        return Ok(());
    }

    Err(TeamsError::InvalidOperation(format!(
        "{label} provider and model must be configured together"
    )))
}
