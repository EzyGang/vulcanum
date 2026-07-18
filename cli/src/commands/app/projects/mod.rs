mod catalog;
pub(crate) mod repos;
mod runtime;
#[cfg(test)]
mod tests;

use std::io;

use uuid::Uuid;
use vulcanum_shared::api::app::projects::{AppProject, CreateProjectRequest};
use vulcanum_shared::api::app::task_trackers::TaskTracker;
use vulcanum_shared::state::app as app_state;

use crate::commands::app::projects::catalog::{
    available_projects, resolve_repos, select_candidate, validate_source_flags,
};
use crate::commands::app::projects::runtime::ProjectsRuntime;
use crate::commands::app::{
    authenticated_context, handle_authenticated_error, resolve_team, AppRuntime,
};
use crate::console::{escape_terminal, render_table};

pub(crate) struct AddOptions {
    pub(crate) provider: Option<Uuid>,
    pub(crate) workspace: Option<String>,
    pub(crate) project: Option<String>,
    pub(crate) repos: Vec<String>,
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

pub async fn add(options: AddOptions) -> anyhow::Result<()> {
    let mut stdout = io::stdout();
    let mut load_session = app_state::load_state;
    let mut save_session = app_state::save_state;
    let mut app = AppRuntime {
        stdout: &mut stdout,
        load_session: &mut load_session,
        save_session: &mut save_session,
    };
    let mut projects = ProjectsRuntime::real();
    add_with(options, &mut app, &mut projects).await
}

pub(super) async fn list_with(
    team_override: Option<Uuid>,
    runtime: &mut AppRuntime<'_>,
) -> anyhow::Result<()> {
    let context = authenticated_context(runtime).await?;
    let team = resolve_team(&context, team_override).await?;
    let projects_request = context
        .client
        .list_projects(team.id, &context.session.access_token);
    let providers_request = context
        .client
        .list_task_trackers(team.id, &context.session.access_token);
    let (projects, providers) = tokio::try_join!(projects_request, providers_request)
        .map_err(|error| handle_authenticated_error("List projects", error))?;
    let team_name = escape_terminal(&team.name);
    if projects.is_empty() {
        writeln!(
            runtime.stdout,
            "No projects configured for team {team_name} ({}).",
            team.id
        )?;
        return Ok(());
    }

    let rows = projects
        .into_iter()
        .map(|project| render_project(project, &providers))
        .collect();
    let table = render_table(
        &[
            "ID",
            "NAME",
            "TASK TRACKER",
            "EXTERNAL PROJECT",
            "AUTOMATION",
            "REPOSITORIES",
        ],
        rows,
    );
    writeln!(
        runtime.stdout,
        "Projects — {team_name} ({})\n{table}",
        team.id
    )?;
    Ok(())
}

pub(super) async fn add_with(
    options: AddOptions,
    app: &mut AppRuntime<'_>,
    projects_runtime: &mut ProjectsRuntime,
) -> anyhow::Result<()> {
    validate_source_flags(&options)?;
    let context = authenticated_context(app).await?;
    let team = resolve_team(&context, options.team).await?;
    let configured_request = context
        .client
        .list_projects(team.id, &context.session.access_token);
    let providers_request = context
        .client
        .list_task_trackers(team.id, &context.session.access_token);
    let (configured, providers) = tokio::try_join!(configured_request, providers_request)
        .map_err(|error| handle_authenticated_error("List available projects", error))?;
    let candidates = available_projects(&context, team.id, &providers, &configured).await?;
    let interactive = options.provider.is_none();
    let candidate = select_candidate(&options, &candidates, projects_runtime)?;
    let repos = resolve_repos(
        &context,
        team.id,
        &options.repos,
        &[],
        interactive,
        false,
        projects_runtime,
    )
    .await?;
    let request = CreateProjectRequest {
        external_project_id: candidate.external_project_id.clone(),
        external_workspace_id: candidate.workspace_id.clone(),
        name: candidate.name.clone(),
        provider_id: candidate.provider_id,
        enabled: false,
        repo_full_names: repos,
    };
    let created = context
        .client
        .create_project(team.id, &request, &context.session.access_token)
        .await
        .map_err(|error| handle_authenticated_error("Add project", error))?;
    writeln!(
        app.stdout,
        "Added project {} ({}) for team {} with automation disabled and {} attached repositories.",
        escape_terminal(&created.name),
        created.id,
        team.id,
        created.repo_full_names.len()
    )?;
    Ok(())
}

fn render_project(project: AppProject, providers: &[TaskTracker]) -> Vec<String> {
    let provider = project.provider_id.map_or_else(
        || "-".to_owned(),
        |id| match providers.iter().find(|provider| provider.id == id) {
            Some(provider) => format!("{} ({id})", escape_terminal(&provider.name)),
            None => id.to_string(),
        },
    );
    let repos = match project.repo_full_names.is_empty() {
        true => "-".to_owned(),
        false => project
            .repo_full_names
            .iter()
            .map(|repo| escape_terminal(repo).into_owned())
            .collect::<Vec<String>>()
            .join(", "),
    };
    vec![
        project.id.to_string(),
        escape_terminal(&project.name).into_owned(),
        provider,
        escape_terminal(&project.external_project_id).into_owned(),
        match project.enabled {
            true => "enabled".to_owned(),
            false => "disabled".to_owned(),
        },
        repos,
    ]
}
