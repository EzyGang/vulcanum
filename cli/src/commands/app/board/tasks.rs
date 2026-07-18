use std::io::Read;

use uuid::Uuid;
use vulcanum_shared::api::app::task_board::{
    CreateTaskRequest, MoveTaskRequest, Task, UpdateTaskRequest,
};

use super::support::{
    find_column, find_task, load_board, load_project, page_bounds, task_labels, task_slug,
};
use crate::commands::app::{handle_authenticated_error, AppRuntime};
use crate::console::{escape_terminal, render_table};

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

pub(super) struct SearchOptions {
    pub(super) project_id: Uuid,
    pub(super) query: Option<String>,
    pub(super) column: Option<String>,
    pub(super) label: Option<String>,
    pub(super) page: usize,
    pub(super) page_size: usize,
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

pub(super) async fn get(
    project_id: Uuid,
    selector: &str,
    team: Option<Uuid>,
    runtime: &mut AppRuntime<'_>,
) -> anyhow::Result<()> {
    let loaded = load_board(project_id, team, runtime).await?;
    let task = find_task(&loaded.board, selector)?;
    let column = loaded
        .board
        .board
        .columns
        .iter()
        .find(|column| column.tasks.iter().any(|candidate| candidate.id == task.id))
        .ok_or_else(|| anyhow::anyhow!("Task column was not found."))?;
    let rows = vec![
        vec![
            "Task".to_owned(),
            escape_terminal(&task_slug(task)).into_owned(),
        ],
        vec![
            "Provider ID".to_owned(),
            escape_terminal(&task.id).into_owned(),
        ],
        vec![
            "Title".to_owned(),
            escape_terminal(&task.title).into_owned(),
        ],
        vec![
            "Column".to_owned(),
            escape_terminal(&column.name).into_owned(),
        ],
        vec![
            "Status".to_owned(),
            escape_terminal(&task.status).into_owned(),
        ],
        vec![
            "Priority".to_owned(),
            escape_terminal(&task.priority).into_owned(),
        ],
        vec!["Labels".to_owned(), task_labels(task)],
        vec![
            "Assignee".to_owned(),
            task.assignee_name
                .as_deref()
                .map_or_else(|| "—".to_owned(), |name| escape_terminal(name).into_owned()),
        ],
        vec![
            "Created".to_owned(),
            escape_terminal(&task.created_at).into_owned(),
        ],
        vec![
            "Updated".to_owned(),
            task.updated_at
                .as_deref()
                .map_or_else(|| "—".to_owned(), |date| escape_terminal(date).into_owned()),
        ],
    ];
    writeln!(
        runtime.stdout,
        "{}",
        render_table(&["FIELD", "VALUE"], rows)
    )?;
    writeln!(runtime.stdout, "BODY")?;
    match task.description.as_deref() {
        Some(body) if !body.is_empty() => {
            for line in body.lines() {
                writeln!(runtime.stdout, "  {}", escape_terminal(line))?;
            }
        }
        _ => writeln!(runtime.stdout, "  —")?,
    }
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

pub(super) async fn search(
    options: SearchOptions,
    runtime: &mut AppRuntime<'_>,
) -> anyhow::Result<()> {
    let loaded = load_board(options.project_id, options.team, runtime).await?;
    let columns = match options.column.as_deref() {
        Some(selector) => vec![find_column(&loaded.board, selector)?],
        None => loaded.board.board.columns.iter().collect(),
    };
    let query = options.query.as_deref().map(str::to_lowercase);
    let mut matches: Vec<(&Task, &str)> = columns
        .into_iter()
        .flat_map(|column| {
            column
                .tasks
                .iter()
                .map(move |task| (task, column.name.as_str()))
        })
        .filter(|(task, _)| matches_query(task, query.as_deref()))
        .filter(|(task, _)| matches_label(task, options.label.as_deref()))
        .collect();
    matches.sort_by_cached_key(|(task, _)| task_slug(task));
    let (start, end, total_pages) = page_bounds(matches.len(), options.page, options.page_size)?;
    let rows = matches[start..end]
        .iter()
        .map(|(task, column)| {
            vec![
                escape_terminal(&task_slug(task)).into_owned(),
                escape_terminal(&task.title).into_owned(),
                escape_terminal(column).into_owned(),
                task_labels(task),
            ]
        })
        .collect();
    writeln!(
        runtime.stdout,
        "{} tasks, page {}/{total_pages}\n{}",
        matches.len(),
        options.page,
        render_table(&["TASK", "TITLE", "COLUMN", "LABELS"], rows)
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

fn matches_query(task: &Task, query: Option<&str>) -> bool {
    let Some(query) = query else {
        return true;
    };
    task_slug(task).to_lowercase().contains(query)
        || task.title.to_lowercase().contains(query)
        || task
            .description
            .as_deref()
            .is_some_and(|body| body.to_lowercase().contains(query))
}

fn matches_label(task: &Task, label: Option<&str>) -> bool {
    let Some(label) = label else {
        return true;
    };
    task.labels.iter().any(|task_label| {
        task_label.id.eq_ignore_ascii_case(label) || task_label.name.eq_ignore_ascii_case(label)
    })
}
