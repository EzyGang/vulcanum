use uuid::Uuid;
use vulcanum_shared::api::app::task_board::Task;

use super::support::{find_column, find_task, load_board, page_bounds, task_labels, task_slug};
use crate::commands::app::AppRuntime;
use crate::console::{escape_terminal, render_table};

pub(super) struct SearchOptions {
    pub(super) project_id: Uuid,
    pub(super) query: Option<String>,
    pub(super) column: Option<String>,
    pub(super) label: Option<String>,
    pub(super) page: usize,
    pub(super) page_size: usize,
    pub(super) team: Option<Uuid>,
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
