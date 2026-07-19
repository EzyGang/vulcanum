use uuid::Uuid;
use vulcanum_shared::api::app::projects::AppProject;
use vulcanum_shared::api::app::task_trackers::TaskTracker;

use crate::commands::app::projects::runtime::ProjectsRuntime;
use crate::commands::app::projects::AddOptions;
use crate::commands::app::{handle_authenticated_error, AppContext};
use crate::console::escape_terminal;

pub(super) struct ProjectCandidate {
    pub(super) provider_id: Uuid,
    pub(super) workspace_id: String,
    pub(super) external_project_id: String,
    pub(super) name: String,
    provider_name: String,
    workspace_name: String,
}

pub(super) async fn available_projects(
    context: &AppContext,
    team_id: Uuid,
    providers: &[TaskTracker],
    configured: &[AppProject],
) -> anyhow::Result<Vec<ProjectCandidate>> {
    let mut candidates = Vec::new();
    for provider in providers {
        let workspaces = context
            .client
            .list_provider_workspaces(team_id, provider.id, &context.session.access_token)
            .await
            .map_err(|error| handle_authenticated_error("List provider workspaces", error))?;
        for workspace in workspaces {
            let projects = context
                .client
                .list_provider_projects(
                    team_id,
                    provider.id,
                    &workspace.id,
                    &context.session.access_token,
                )
                .await
                .map_err(|error| handle_authenticated_error("List provider projects", error))?;
            candidates.extend(
                projects
                    .into_iter()
                    .filter(|project| {
                        !configured.iter().any(|config| {
                            config.provider_id == Some(provider.id)
                                && config.external_project_id == project.id
                        })
                    })
                    .map(|project| ProjectCandidate {
                        provider_id: provider.id,
                        provider_name: provider.name.clone(),
                        workspace_id: workspace.id.clone(),
                        workspace_name: workspace.name.clone(),
                        external_project_id: project.id,
                        name: project.name,
                    }),
            );
        }
    }
    candidates.sort_by(|left, right| {
        left.name
            .cmp(&right.name)
            .then_with(|| left.workspace_name.cmp(&right.workspace_name))
            .then_with(|| left.provider_name.cmp(&right.provider_name))
    });
    Ok(candidates)
}

pub(super) fn select_candidate<'a>(
    options: &AddOptions,
    candidates: &'a [ProjectCandidate],
    runtime: &mut ProjectsRuntime,
) -> anyhow::Result<&'a ProjectCandidate> {
    if candidates.is_empty() {
        anyhow::bail!("No unconfigured provider projects are available");
    }
    match (&options.provider, &options.workspace, &options.project) {
        (Some(provider), Some(workspace), Some(project)) => candidates
            .iter()
            .find(|candidate| {
                candidate.provider_id == *provider
                    && candidate.workspace_id == *workspace
                    && candidate.external_project_id == *project
            })
            .ok_or_else(|| anyhow::anyhow!("Selected provider project is not available")),
        (None, None, None) => {
            if !runtime.stdin_is_terminal {
                anyhow::bail!(
                    "stdin is not a terminal; pass --provider, --workspace, and --project"
                );
            }
            let labels = candidates
                .iter()
                .map(candidate_label)
                .collect::<Vec<String>>();
            let index = (runtime.select)("Select a project", &labels)?;
            candidates
                .get(index)
                .ok_or_else(|| anyhow::anyhow!("Project selection was invalid"))
        }
        _ => anyhow::bail!("--provider, --workspace, and --project must be supplied together"),
    }
}

pub(super) async fn resolve_repos(
    context: &AppContext,
    team_id: Uuid,
    requested: &[String],
    preselected: &[String],
    interactive: bool,
    require_installation: bool,
    runtime: &mut ProjectsRuntime,
) -> anyhow::Result<Vec<String>> {
    if !requested.is_empty() {
        let available = available_repos(context, team_id).await?;
        let mut selected = Vec::new();
        for requested_name in requested {
            let repo = available
                .iter()
                .find(|repo| repo.full_name.eq_ignore_ascii_case(requested_name))
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "Repository {} is not available to the GitHub App",
                        escape_terminal(requested_name)
                    )
                })?;
            if !selected.contains(&repo.full_name) {
                selected.push(repo.full_name.clone());
            }
        }
        return Ok(selected);
    }
    if !interactive {
        return Ok(Vec::new());
    }
    let installation = context
        .client
        .get_github_app_installation(team_id, &context.session.access_token)
        .await
        .map_err(|error| handle_authenticated_error("Get GitHub App installation", error))?;
    if installation.is_none() {
        if require_installation {
            anyhow::bail!("No GitHub App installation is connected for this team");
        }
        return Ok(Vec::new());
    }
    let mut available = available_repos(context, team_id).await?;
    available.sort_by(|left, right| left.full_name.cmp(&right.full_name));
    for current in preselected {
        if !available
            .iter()
            .any(|repo| repo.full_name.eq_ignore_ascii_case(current))
        {
            anyhow::bail!(
                "Attached repository {} is no longer available; use --clear or provide available --repo values",
                escape_terminal(current)
            );
        }
    }
    if available.is_empty() {
        return Ok(Vec::new());
    }
    let labels = available
        .iter()
        .map(|repo| escape_terminal(&repo.full_name).into_owned())
        .collect::<Vec<String>>();
    let defaults = available
        .iter()
        .map(|repo| {
            preselected
                .iter()
                .any(|current| repo.full_name.eq_ignore_ascii_case(current))
        })
        .collect::<Vec<bool>>();
    let indices = (runtime.select_many)(
        "Select repositories to attach (optional)",
        &labels,
        &defaults,
    )?;
    indices
        .into_iter()
        .map(|index| {
            available
                .get(index)
                .map(|repo| repo.full_name.clone())
                .ok_or_else(|| anyhow::anyhow!("Repository selection was invalid"))
        })
        .collect()
}

pub(super) async fn available_repos(
    context: &AppContext,
    team_id: Uuid,
) -> anyhow::Result<Vec<vulcanum_shared::api::app::github::GithubRepo>> {
    context
        .client
        .list_github_repos(team_id, &context.session.access_token)
        .await
        .map_err(|error| handle_authenticated_error("List available repositories", error))
}

pub(super) fn validate_source_flags(options: &AddOptions) -> anyhow::Result<()> {
    match (&options.provider, &options.workspace, &options.project) {
        (Some(_), Some(_), Some(_)) | (None, None, None) => Ok(()),
        _ => anyhow::bail!("--provider, --workspace, and --project must be supplied together"),
    }
}

fn candidate_label(candidate: &ProjectCandidate) -> String {
    format!(
        "{} · {} · {}",
        escape_terminal(&candidate.name),
        escape_terminal(&candidate.workspace_name),
        escape_terminal(&candidate.provider_name)
    )
}
