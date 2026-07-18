pub(crate) mod args;
pub(crate) mod runs;
pub(crate) mod settings;
pub(crate) mod workers;

#[cfg(test)]
mod tests;

use std::io::Write;

use uuid::Uuid;
use vulcanum_shared::api::app::teams::AppTeam;
use vulcanum_shared::api::error::{is_fatal_api_error, ApiError};
use vulcanum_shared::client::ApiClient;
use vulcanum_shared::state::app::AppSession;

pub(super) struct AppRuntime<'a> {
    pub(super) stdout: &'a mut dyn Write,
    pub(super) load_session: &'a mut dyn FnMut() -> anyhow::Result<Option<AppSession>>,
    pub(super) save_session: &'a mut dyn FnMut(&AppSession) -> anyhow::Result<()>,
}

pub(super) struct AppContext {
    pub(super) client: ApiClient,
    pub(super) session: AppSession,
}

pub(super) fn load_required_session(runtime: &mut AppRuntime<'_>) -> anyhow::Result<AppSession> {
    match (runtime.load_session)()? {
        Some(session) => Ok(session),
        None => anyhow::bail!("Not logged in. Run `vulcanum login`."),
    }
}

pub(super) async fn authenticated_context(
    runtime: &mut AppRuntime<'_>,
) -> anyhow::Result<AppContext> {
    let mut session = load_required_session(runtime)?;
    let client = ApiClient::new(&session.instance_url);
    let tokens = match client.refresh_app_session(&session.refresh_token).await {
        Ok(tokens) => tokens,
        Err(error) if is_fatal_api_error(&error) => {
            anyhow::bail!("Login expired. Run `vulcanum login`.")
        }
        Err(error) => return Err(sanitize_request_error("Refresh session", error)),
    };

    session.access_token = tokens.access_token;
    session.refresh_token = tokens.refresh_token;
    session.refresh_expires_at = tokens.refresh_expires_at;
    (runtime.save_session)(&session)?;

    Ok(AppContext { client, session })
}

pub(super) async fn resolve_team(
    context: &AppContext,
    team_override: Option<Uuid>,
) -> anyhow::Result<AppTeam> {
    match team_override {
        Some(team_id) => fetch_selected_team(context, team_id, false).await,
        None => match context.session.team_id {
            Some(team_id) => fetch_selected_team(context, team_id, true).await,
            None => {
                let teams = context
                    .client
                    .list_teams(&context.session.access_token)
                    .await
                    .map_err(|error| handle_authenticated_error("List teams", error))?;
                teams
                    .into_iter()
                    .next()
                    .ok_or_else(|| anyhow::anyhow!("No teams are available for this account."))
            }
        },
    }
}

pub(super) fn sanitize_request_error(operation: &str, error: anyhow::Error) -> anyhow::Error {
    match error.downcast_ref::<ApiError>() {
        Some(api_error) if (200..300).contains(&api_error.status) => anyhow::anyhow!(
            "{operation} failed: invalid response (HTTP {})",
            api_error.status
        ),
        Some(api_error) => anyhow::anyhow!("{operation} failed: HTTP {}", api_error.status),
        None => anyhow::anyhow!("{operation} failed"),
    }
}

pub(super) fn handle_authenticated_error(operation: &str, error: anyhow::Error) -> anyhow::Error {
    match error.downcast_ref::<ApiError>() {
        Some(api_error) if api_error.status == 401 => {
            anyhow::anyhow!("Login expired. Run `vulcanum login`.")
        }
        _ => sanitize_request_error(operation, error),
    }
}

async fn fetch_selected_team(
    context: &AppContext,
    team_id: Uuid,
    pinned: bool,
) -> anyhow::Result<AppTeam> {
    match context
        .client
        .get_team(team_id, &context.session.access_token)
        .await
    {
        Ok(team) => Ok(team),
        Err(error) => match error.downcast_ref::<ApiError>() {
            Some(api_error) if api_error.status == 401 => {
                anyhow::bail!("Login expired. Run `vulcanum login`.")
            }
            Some(api_error) if pinned && matches!(api_error.status, 403 | 404) => anyhow::bail!(
                "Pinned team {team_id} is no longer accessible. Run `vulcanum settings team set <UUID>` or `vulcanum settings team clear`."
            ),
            Some(api_error) if matches!(api_error.status, 403 | 404) => Err(anyhow::anyhow!(
                "Team {team_id} is unavailable: HTTP {}",
                api_error.status
            )),
            _ => Err(sanitize_request_error("Get team", error)),
        },
    }
}
