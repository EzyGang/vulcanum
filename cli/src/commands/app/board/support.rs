use uuid::Uuid;
use vulcanum_shared::api::app::projects::AppProject;
use vulcanum_shared::api::app::task_board::{Task, TaskBoardColumn, TaskBoardResponse};

use crate::commands::app::{
    authenticated_context, handle_authenticated_error, resolve_team, AppContext, AppRuntime,
};
use crate::console::escape_terminal;

pub(in crate::commands::app) struct LoadedBoard {
    pub(in crate::commands::app) context: AppContext,
    pub(in crate::commands::app) project: AppProject,
    pub(in crate::commands::app) team_id: Uuid,
    pub(in crate::commands::app) provider_id: Uuid,
    pub(in crate::commands::app) board: TaskBoardResponse,
}

pub(in crate::commands::app) async fn load_project(
    project_id: Uuid,
    team_override: Option<Uuid>,
    runtime: &mut AppRuntime<'_>,
) -> anyhow::Result<(AppContext, AppProject, Uuid, Uuid)> {
    let context = authenticated_context(runtime).await?;
    let team = resolve_team(&context, team_override).await?;
    let project = context
        .client
        .get_project(team.id, project_id, &context.session.access_token)
        .await
        .map_err(|error| handle_authenticated_error("Get project", error))?;
    let provider_id = project.provider_id.ok_or_else(|| {
        anyhow::anyhow!(
            "Project {} has no task-tracker provider. Run `vulcanum projects list`.",
            project.id
        )
    })?;
    Ok((context, project, team.id, provider_id))
}

pub(in crate::commands::app) async fn load_board(
    project_id: Uuid,
    team_override: Option<Uuid>,
    runtime: &mut AppRuntime<'_>,
) -> anyhow::Result<LoadedBoard> {
    let (context, project, team_id, provider_id) =
        load_project(project_id, team_override, runtime).await?;
    let board = context
        .client
        .get_task_board(
            team_id,
            provider_id,
            &project.external_project_id,
            &context.session.access_token,
        )
        .await
        .map_err(|error| handle_authenticated_error("Get task board", error))?;
    Ok(LoadedBoard {
        context,
        project,
        team_id,
        provider_id,
        board,
    })
}

pub(in crate::commands::app) fn find_column<'a>(
    board: &'a TaskBoardResponse,
    selector: &str,
) -> anyhow::Result<&'a TaskBoardColumn> {
    board
        .board
        .columns
        .iter()
        .find(|column| {
            column.id.eq_ignore_ascii_case(selector)
                || column.slug.eq_ignore_ascii_case(selector)
                || column.name.eq_ignore_ascii_case(selector)
        })
        .ok_or_else(|| anyhow::anyhow!("Column `{selector}` was not found."))
}

pub(super) fn find_task<'a>(
    board: &'a TaskBoardResponse,
    selector: &str,
) -> anyhow::Result<&'a Task> {
    board
        .board
        .columns
        .iter()
        .flat_map(|column| column.tasks.iter())
        .find(|task| task.id == selector || task_slug(task).eq_ignore_ascii_case(selector))
        .ok_or_else(|| anyhow::anyhow!("Task `{selector}` was not found by provider ID or slug."))
}

pub(super) fn task_slug(task: &Task) -> String {
    match (task.project_slug.as_deref(), task.number) {
        (Some(project_slug), Some(number)) if !project_slug.is_empty() => {
            format!("{}-{number}", project_slug.to_uppercase())
        }
        _ => task.id.clone(),
    }
}

pub(super) fn project_mark(board: &TaskBoardResponse, value: &str) -> String {
    if value.is_empty() {
        return "— (unset)".to_owned();
    }
    match find_column(board, value) {
        Ok(column) => format!(
            "{} ({})",
            escape_terminal(&column.name),
            escape_terminal(&column.slug)
        ),
        Err(_) => format!("{} (missing)", escape_terminal(value)),
    }
}

pub(super) fn task_labels(task: &Task) -> String {
    if task.labels.is_empty() {
        return "—".to_owned();
    }
    task.labels
        .iter()
        .map(|label| escape_terminal(&label.name).into_owned())
        .collect::<Vec<String>>()
        .join(", ")
}

pub(super) fn task_summary_row(task: &Task, column: Option<&str>) -> Vec<String> {
    vec![
        escape_terminal(&task_slug(task)).into_owned(),
        escape_terminal(&task.id).into_owned(),
        escape_terminal(&task.title).into_owned(),
        column.map_or_else(|| "—".to_owned(), |name| escape_terminal(name).into_owned()),
        task_labels(task),
        escape_terminal(&task.priority).into_owned(),
    ]
}

pub(super) fn page_bounds(
    total: usize,
    page: usize,
    page_size: usize,
) -> anyhow::Result<(usize, usize, usize)> {
    if page == 0 || page_size == 0 {
        anyhow::bail!("--page and --page-size must be greater than zero.");
    }
    let total_pages = total.div_ceil(page_size).max(1);
    if page > total_pages {
        anyhow::bail!("Page {page} exceeds the last page ({total_pages}).");
    }
    let start = (page - 1)
        .checked_mul(page_size)
        .ok_or_else(|| anyhow::anyhow!("Requested page is too large."))?;
    Ok((start, (start + page_size).min(total), total_pages))
}
