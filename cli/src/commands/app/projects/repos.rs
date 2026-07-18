use std::io;

use uuid::Uuid;
use vulcanum_shared::api::app::projects::UpdateProjectRequest;
use vulcanum_shared::state::app as app_state;

use crate::commands::app::projects::catalog::{available_repos, resolve_repos};
use crate::commands::app::projects::runtime::ProjectsRuntime;
use crate::commands::app::{
    authenticated_context, handle_authenticated_error, resolve_team, AppRuntime,
};
use crate::console::{escape_terminal, render_table};

pub(crate) struct EditOptions {
    pub(crate) project_id: Uuid,
    pub(crate) repos: Vec<String>,
    pub(crate) clear: bool,
    pub(crate) team: Option<Uuid>,
}

pub async fn list(team: Option<Uuid>) -> anyhow::Result<()> {
    let mut stdout = io::stdout();
    let mut load_session = app_state::load_state;
    let mut save_session = app_state::save_state;
    let mut runtime = AppRuntime {
        stdout: &mut stdout,
        load_session: &mut load_session,
        save_session: &mut save_session,
    };
    list_with(team, &mut runtime).await
}

pub async fn set(options: EditOptions) -> anyhow::Result<()> {
    let mut stdout = io::stdout();
    let mut load_session = app_state::load_state;
    let mut save_session = app_state::save_state;
    let mut app = AppRuntime {
        stdout: &mut stdout,
        load_session: &mut load_session,
        save_session: &mut save_session,
    };
    let mut projects = ProjectsRuntime::real();
    set_with(options, &mut app, &mut projects).await
}

pub(super) async fn list_with(
    team_override: Option<Uuid>,
    runtime: &mut AppRuntime<'_>,
) -> anyhow::Result<()> {
    let context = authenticated_context(runtime).await?;
    let team = resolve_team(&context, team_override).await?;
    let installation = context
        .client
        .get_github_app_installation(team.id, &context.session.access_token)
        .await
        .map_err(|error| handle_authenticated_error("Get GitHub App installation", error))?;
    let team_name = escape_terminal(&team.name);
    if installation.is_none() {
        writeln!(
            runtime.stdout,
            "No GitHub App installation connected for team {team_name} ({}).",
            team.id
        )?;
        return Ok(());
    }
    let mut repos = available_repos(&context, team.id).await?;
    repos.sort_by(|left, right| left.full_name.cmp(&right.full_name));
    if repos.is_empty() {
        writeln!(
            runtime.stdout,
            "No GitHub repositories available for team {team_name} ({}).",
            team.id
        )?;
        return Ok(());
    }
    let rows = repos
        .into_iter()
        .map(|repo| {
            vec![
                escape_terminal(&repo.owner).into_owned(),
                escape_terminal(&repo.name).into_owned(),
                escape_terminal(&repo.full_name).into_owned(),
            ]
        })
        .collect();
    let table = render_table(&["OWNER", "NAME", "FULL NAME"], rows);
    writeln!(
        runtime.stdout,
        "Available GitHub repositories — {team_name} ({})\n{table}",
        team.id
    )?;
    Ok(())
}

pub(super) async fn set_with(
    options: EditOptions,
    app: &mut AppRuntime<'_>,
    projects_runtime: &mut ProjectsRuntime,
) -> anyhow::Result<()> {
    let context = authenticated_context(app).await?;
    let team = resolve_team(&context, options.team).await?;
    let project = context
        .client
        .get_project(team.id, options.project_id, &context.session.access_token)
        .await
        .map_err(|error| handle_authenticated_error("Get project", error))?;
    let repos = if options.clear {
        Vec::new()
    } else {
        let interactive = options.repos.is_empty();
        if interactive && !projects_runtime.stdin_is_terminal {
            anyhow::bail!("stdin is not a terminal; pass one or more --repo values or --clear");
        }
        resolve_repos(
            &context,
            team.id,
            &options.repos,
            &project.repo_full_names,
            interactive,
            true,
            projects_runtime,
        )
        .await?
    };
    let request = UpdateProjectRequest {
        repo_full_names: repos,
    };
    let updated = context
        .client
        .update_project(
            team.id,
            options.project_id,
            &request,
            &context.session.access_token,
        )
        .await
        .map_err(|error| handle_authenticated_error("Update project repositories", error))?;
    writeln!(
        app.stdout,
        "Updated project {} ({}) with {} attached repositories.",
        escape_terminal(&updated.name),
        updated.id,
        updated.repo_full_names.len()
    )?;
    Ok(())
}
