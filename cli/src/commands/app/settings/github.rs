use std::io;

use uuid::Uuid;
use vulcanum_shared::state::app as app_state;

use crate::commands::app::settings::runtime::SettingsRuntime;
use crate::commands::app::{
    authenticated_context, handle_authenticated_error, resolve_team, AppRuntime,
};
use crate::console::escape_terminal;

pub(crate) async fn connect(no_browser: bool, team: Option<Uuid>) -> anyhow::Result<()> {
    let mut stdout = io::stdout();
    let mut load_session = app_state::load_state;
    let mut save_session = app_state::save_state;
    let mut app = AppRuntime {
        stdout: &mut stdout,
        load_session: &mut load_session,
        save_session: &mut save_session,
    };
    let mut settings = SettingsRuntime::real();
    connect_with(no_browser, team, &mut app, &mut settings).await
}

pub(crate) async fn disconnect(team: Option<Uuid>) -> anyhow::Result<()> {
    let mut stdout = io::stdout();
    let mut load_session = app_state::load_state;
    let mut save_session = app_state::save_state;
    let mut app = AppRuntime {
        stdout: &mut stdout,
        load_session: &mut load_session,
        save_session: &mut save_session,
    };
    disconnect_with(team, &mut app).await
}

pub(super) async fn connect_with(
    no_browser: bool,
    team_override: Option<Uuid>,
    app: &mut AppRuntime<'_>,
    settings: &mut SettingsRuntime,
) -> anyhow::Result<()> {
    let context = authenticated_context(app).await?;
    let team = resolve_team(&context, team_override).await?;
    let response = context
        .client
        .get_github_auth_url(team.id, &context.session.access_token)
        .await
        .map_err(|error| handle_authenticated_error("Get GitHub App authorization URL", error))?;
    writeln!(
        app.stdout,
        "Open {} to install the GitHub App for team {}.",
        escape_terminal(&response.url),
        team.id
    )?;
    if !no_browser && (settings.open_browser)(&response.url).is_err() {
        writeln!(
            settings.stderr,
            "Warning: could not open the browser; continue with the printed URL."
        )?;
    }
    writeln!(app.stdout, "GitHub App connection initiated.")?;
    Ok(())
}

pub(super) async fn disconnect_with(
    team_override: Option<Uuid>,
    app: &mut AppRuntime<'_>,
) -> anyhow::Result<()> {
    let context = authenticated_context(app).await?;
    let team = resolve_team(&context, team_override).await?;
    let installation = context
        .client
        .get_github_app_installation(team.id, &context.session.access_token)
        .await
        .map_err(|error| handle_authenticated_error("Get GitHub App installation", error))?;
    let installation = match installation {
        Some(installation) => installation,
        None => {
            writeln!(
                app.stdout,
                "GitHub App is already disconnected for team {}.",
                team.id
            )?;
            return Ok(());
        }
    };
    context
        .client
        .delete_github_app_installation(team.id, installation.id, &context.session.access_token)
        .await
        .map_err(|error| handle_authenticated_error("Disconnect GitHub App", error))?;
    writeln!(
        app.stdout,
        "Disconnected GitHub account {} from team {}.",
        escape_terminal(&installation.account_login),
        team.id
    )?;
    Ok(())
}
