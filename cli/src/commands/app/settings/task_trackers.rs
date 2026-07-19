use std::io;

use uuid::Uuid;
use vulcanum_shared::api::app::task_trackers::{
    CreateTaskTrackerRequest, UpdateTaskTrackerRequest,
};
use vulcanum_shared::state::app as app_state;

use crate::commands::app::settings::credentials::task_tracker_credentials;
use crate::commands::app::settings::runtime::SettingsRuntime;
use crate::commands::app::{
    authenticated_context, handle_authenticated_error, resolve_team, AppRuntime,
};
use crate::console::escape_terminal;

pub(crate) struct UpdateOptions {
    pub(crate) id: Uuid,
    pub(crate) name: Option<String>,
    pub(crate) instance_url: Option<String>,
    pub(crate) credentials_stdin: bool,
    pub(crate) prompt_credentials: bool,
    pub(crate) team: Option<Uuid>,
}

pub(crate) async fn add(
    name: String,
    instance_url: String,
    credentials_stdin: bool,
    team: Option<Uuid>,
) -> anyhow::Result<()> {
    let mut stdout = io::stdout();
    let mut load_session = app_state::load_state;
    let mut save_session = app_state::save_state;
    let mut app = AppRuntime {
        stdout: &mut stdout,
        load_session: &mut load_session,
        save_session: &mut save_session,
    };
    let mut settings = SettingsRuntime::real();
    add_with(
        &name,
        &instance_url,
        credentials_stdin,
        team,
        &mut app,
        &mut settings,
    )
    .await
}

pub(crate) async fn update(options: UpdateOptions) -> anyhow::Result<()> {
    let mut stdout = io::stdout();
    let mut load_session = app_state::load_state;
    let mut save_session = app_state::save_state;
    let mut app = AppRuntime {
        stdout: &mut stdout,
        load_session: &mut load_session,
        save_session: &mut save_session,
    };
    let mut settings = SettingsRuntime::real();
    update_with(options, &mut app, &mut settings).await
}

pub(crate) async fn remove(id: Uuid, team: Option<Uuid>) -> anyhow::Result<()> {
    let mut stdout = io::stdout();
    let mut load_session = app_state::load_state;
    let mut save_session = app_state::save_state;
    let mut runtime = AppRuntime {
        stdout: &mut stdout,
        load_session: &mut load_session,
        save_session: &mut save_session,
    };
    remove_with(id, team, &mut runtime).await
}

pub(super) async fn add_with(
    name: &str,
    instance_url: &str,
    credentials_stdin: bool,
    team_override: Option<Uuid>,
    app: &mut AppRuntime<'_>,
    settings: &mut SettingsRuntime,
) -> anyhow::Result<()> {
    let context = authenticated_context(app).await?;
    let team = resolve_team(&context, team_override).await?;
    let api_key = task_tracker_credentials(credentials_stdin, settings)?;
    let request = CreateTaskTrackerRequest {
        name: name.to_owned(),
        instance_url: instance_url.to_owned(),
        api_key,
    };
    let tracker = context
        .client
        .create_task_tracker(team.id, &request, &context.session.access_token)
        .await
        .map_err(|error| handle_authenticated_error("Add task tracker", error))?;
    writeln!(
        app.stdout,
        "Added task tracker {} ({}) for team {}.",
        escape_terminal(&tracker.name),
        tracker.id,
        team.id
    )?;
    Ok(())
}

pub(super) async fn update_with(
    options: UpdateOptions,
    app: &mut AppRuntime<'_>,
    settings: &mut SettingsRuntime,
) -> anyhow::Result<()> {
    if options.name.is_none()
        && options.instance_url.is_none()
        && !options.credentials_stdin
        && !options.prompt_credentials
    {
        anyhow::bail!("Task tracker update requires at least one change");
    }
    let context = authenticated_context(app).await?;
    let team = resolve_team(&context, options.team).await?;
    let api_key = match options.credentials_stdin || options.prompt_credentials {
        true => Some(task_tracker_credentials(
            options.credentials_stdin,
            settings,
        )?),
        false => None,
    };
    let request = UpdateTaskTrackerRequest {
        name: options.name,
        instance_url: options.instance_url,
        api_key,
    };
    let tracker = context
        .client
        .update_task_tracker(team.id, options.id, &request, &context.session.access_token)
        .await
        .map_err(|error| handle_authenticated_error("Update task tracker", error))?;
    writeln!(
        app.stdout,
        "Updated task tracker {} ({}) for team {}.",
        escape_terminal(&tracker.name),
        tracker.id,
        team.id
    )?;
    Ok(())
}

pub(super) async fn remove_with(
    id: Uuid,
    team_override: Option<Uuid>,
    app: &mut AppRuntime<'_>,
) -> anyhow::Result<()> {
    let context = authenticated_context(app).await?;
    let team = resolve_team(&context, team_override).await?;
    context
        .client
        .delete_task_tracker(team.id, id, &context.session.access_token)
        .await
        .map_err(|error| handle_authenticated_error("Remove task tracker", error))?;
    writeln!(
        app.stdout,
        "Removed task tracker {id} from team {}.",
        team.id
    )?;
    Ok(())
}
