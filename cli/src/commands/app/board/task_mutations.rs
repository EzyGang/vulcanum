use std::io::Read;

use uuid::Uuid;
use vulcanum_shared::api::app::task_board::{
    CreateTaskRequest, MoveTaskRequest, UpdateTaskRequest,
};

use super::support::{find_column, find_task, load_board, load_project, task_slug};
use crate::commands::app::{handle_authenticated_error, AppRuntime};
use crate::console::escape_terminal;

pub(super) struct CreateOptions {
    pub(super) project_id: Uuid,
    pub(super) title: String,
    pub(super) body: Option<String>,
    pub(super) body_stdin: bool,
    pub(super) status: Option<String>,
    pub(super) priority: Option<String>,
    pub(super) team: Option<Uuid>,
}

pub(super) struct EditOptions {
    pub(super) project_id: Uuid,
    pub(super) task: String,
    pub(super) title: Option<String>,
    pub(super) body: Option<String>,
    pub(super) body_stdin: bool,
    pub(super) team: Option<Uuid>,
}

pub(super) async fn create(
    options: CreateOptions,
    runtime: &mut AppRuntime<'_>,
    stdin: &mut dyn Read,
) -> anyhow::Result<()> {
    if options.title.trim().is_empty() {
        anyhow::bail!("Task title must not be empty.");
    }
    let body = read_body(options.body, options.body_stdin, stdin)?.unwrap_or_default();
    let (context, project, team_id, provider_id) =
        load_project(options.project_id, options.team, runtime).await?;
    let request = CreateTaskRequest {
        title: options.title,
        body,
        status: options.status,
        priority: options.priority,
    };
    let response = context
        .client
        .create_board_task(
            team_id,
            provider_id,
            &project.external_project_id,
            &request,
            &context.session.access_token,
        )
        .await
        .map_err(|error| handle_authenticated_error("Create task", error))?;
    writeln!(
        runtime.stdout,
        "Created task {} ({}) in project {} ({}).",
        escape_terminal(&task_slug(&response.task)),
        escape_terminal(&response.task.id),
        escape_terminal(&project.name),
        project.id
    )?;
    Ok(())
}

pub(super) async fn edit(
    options: EditOptions,
    runtime: &mut AppRuntime<'_>,
    stdin: &mut dyn Read,
) -> anyhow::Result<()> {
    let replacement_body = read_body(options.body, options.body_stdin, stdin)?;
    if options.title.is_none() && replacement_body.is_none() {
        anyhow::bail!("Specify --title, --body, or --body-stdin.");
    }
    if options
        .title
        .as_deref()
        .is_some_and(|title| title.trim().is_empty())
    {
        anyhow::bail!("Task title must not be empty.");
    }
    let loaded = load_board(options.project_id, options.team, runtime).await?;
    let task = find_task(&loaded.board, &options.task)?;
    let task_id = task.id.clone();
    let request = UpdateTaskRequest {
        title: options.title.unwrap_or_else(|| task.title.clone()),
        body: replacement_body.unwrap_or_else(|| task.description.clone().unwrap_or_default()),
    };
    let response = loaded
        .context
        .client
        .update_board_task(
            loaded.team_id,
            loaded.provider_id,
            &task_id,
            &request,
            &loaded.context.session.access_token,
        )
        .await
        .map_err(|error| handle_authenticated_error("Edit task", error))?;
    writeln!(
        runtime.stdout,
        "Updated task {} ({}) — {}.",
        escape_terminal(&task_slug(&response.task)),
        escape_terminal(&response.task.id),
        escape_terminal(&response.task.title)
    )?;
    Ok(())
}

pub(super) async fn move_task(
    project_id: Uuid,
    selector: &str,
    column_selector: &str,
    team: Option<Uuid>,
    runtime: &mut AppRuntime<'_>,
) -> anyhow::Result<()> {
    let loaded = load_board(project_id, team, runtime).await?;
    let task = find_task(&loaded.board, selector)?;
    let task_id = task.id.clone();
    let display_slug = task_slug(task);
    let column = find_column(&loaded.board, column_selector)?;
    let column_name = column.name.clone();
    let request = MoveTaskRequest {
        status: column.slug.clone(),
    };
    loaded
        .context
        .client
        .move_board_task(
            loaded.team_id,
            loaded.provider_id,
            &task_id,
            &request,
            &loaded.context.session.access_token,
        )
        .await
        .map_err(|error| handle_authenticated_error("Move task", error))?;
    writeln!(
        runtime.stdout,
        "Moved task {} ({}) to {}.",
        escape_terminal(&display_slug),
        escape_terminal(&task_id),
        escape_terminal(&column_name)
    )?;
    Ok(())
}

fn read_body(
    body: Option<String>,
    body_stdin: bool,
    stdin: &mut dyn Read,
) -> anyhow::Result<Option<String>> {
    if !body_stdin {
        return Ok(body);
    }
    let mut value = String::new();
    stdin.read_to_string(&mut value)?;
    Ok(Some(value))
}
