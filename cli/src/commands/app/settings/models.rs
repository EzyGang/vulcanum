use std::io;

use uuid::Uuid;
use vulcanum_shared::api::app::teams::UpdateTeamModelsRequest;
use vulcanum_shared::state::app as app_state;

use crate::commands::app::{
    authenticated_context, handle_authenticated_error, resolve_team, AppRuntime,
};
use crate::console::escape_terminal;

#[derive(Clone, Copy)]
pub(crate) enum ModelSlot {
    Primary,
    Small,
}

pub(crate) async fn set(
    slot: ModelSlot,
    provider_key: String,
    model_id: String,
    team: Option<Uuid>,
) -> anyhow::Result<()> {
    let mut stdout = io::stdout();
    let mut load_session = app_state::load_state;
    let mut save_session = app_state::save_state;
    let mut runtime = AppRuntime {
        stdout: &mut stdout,
        load_session: &mut load_session,
        save_session: &mut save_session,
    };
    set_with(slot, &provider_key, &model_id, team, &mut runtime).await
}

pub(crate) async fn clear(slot: ModelSlot, team: Option<Uuid>) -> anyhow::Result<()> {
    let mut stdout = io::stdout();
    let mut load_session = app_state::load_state;
    let mut save_session = app_state::save_state;
    let mut runtime = AppRuntime {
        stdout: &mut stdout,
        load_session: &mut load_session,
        save_session: &mut save_session,
    };
    clear_with(slot, team, &mut runtime).await
}

pub(super) async fn set_with(
    slot: ModelSlot,
    provider_key: &str,
    model_id: &str,
    team_override: Option<Uuid>,
    runtime: &mut AppRuntime<'_>,
) -> anyhow::Result<()> {
    let context = authenticated_context(runtime).await?;
    let team = resolve_team(&context, team_override).await?;
    let providers = context
        .client
        .list_model_providers(team.id, &context.session.access_token)
        .await
        .map_err(|error| handle_authenticated_error("List model providers", error))?;
    if !providers
        .iter()
        .any(|provider| provider.provider_key == provider_key)
    {
        anyhow::bail!(
            "Model provider {} is not connected for team {}",
            escape_terminal(provider_key),
            team.id
        );
    }
    let catalog = context
        .client
        .get_model_catalog(team.id, &context.session.access_token)
        .await
        .map_err(|error| handle_authenticated_error("Get model catalog", error))?;
    let provider = catalog
        .providers
        .iter()
        .find(|provider| provider.id == provider_key)
        .ok_or_else(|| anyhow::anyhow!("Model provider is not in the catalog"))?;
    if !provider.models.iter().any(|model| model.id == model_id) {
        anyhow::bail!(
            "Model {} is not available for provider {}",
            escape_terminal(model_id),
            escape_terminal(provider_key)
        );
    }

    let request = selection_request(slot, Some(provider_key), Some(model_id));
    context
        .client
        .update_team_models(team.id, &request, &context.session.access_token)
        .await
        .map_err(|error| handle_authenticated_error("Update team model selection", error))?;
    writeln!(
        runtime.stdout,
        "Set {} model to {}/{} for team {} ({}).",
        slot.label(),
        escape_terminal(provider_key),
        escape_terminal(model_id),
        escape_terminal(&team.name),
        team.id
    )?;
    Ok(())
}

pub(super) async fn clear_with(
    slot: ModelSlot,
    team_override: Option<Uuid>,
    runtime: &mut AppRuntime<'_>,
) -> anyhow::Result<()> {
    let context = authenticated_context(runtime).await?;
    let team = resolve_team(&context, team_override).await?;
    let request = selection_request(slot, None, None);
    context
        .client
        .update_team_models(team.id, &request, &context.session.access_token)
        .await
        .map_err(|error| handle_authenticated_error("Clear team model selection", error))?;
    writeln!(
        runtime.stdout,
        "Cleared the {} model for team {} ({}).",
        slot.label(),
        escape_terminal(&team.name),
        team.id
    )?;
    Ok(())
}

impl ModelSlot {
    fn label(self) -> &'static str {
        match self {
            Self::Primary => "primary",
            Self::Small => "small",
        }
    }
}

fn selection_request(
    slot: ModelSlot,
    provider_key: Option<&str>,
    model_id: Option<&str>,
) -> UpdateTeamModelsRequest {
    let pair = || {
        (
            Some(provider_key.map(str::to_owned)),
            Some(model_id.map(str::to_owned)),
        )
    };
    match slot {
        ModelSlot::Primary => {
            let (primary_model_provider_key, primary_model_id) = pair();
            UpdateTeamModelsRequest {
                primary_model_provider_key,
                primary_model_id,
                ..UpdateTeamModelsRequest::default()
            }
        }
        ModelSlot::Small => {
            let (small_model_provider_key, small_model_id) = pair();
            UpdateTeamModelsRequest {
                small_model_provider_key,
                small_model_id,
                ..UpdateTeamModelsRequest::default()
            }
        }
    }
}
