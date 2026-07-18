pub(crate) mod args;
pub(in crate::commands::app) mod support;
mod tasks;
#[cfg(test)]
mod tests;

use std::io::{self, Read};

use uuid::Uuid;
use vulcanum_shared::state::app as app_state;

use crate::commands::app::board::args::{BoardCommand, BoardTasksCommand};
use crate::commands::app::board::support::{
    find_column, load_board, page_bounds, project_mark, task_labels, task_slug, task_summary_row,
};
use crate::commands::app::AppRuntime;
use crate::console::{escape_terminal, render_table};

pub async fn run(command: BoardCommand) -> anyhow::Result<()> {
    let mut stdout = io::stdout();
    let mut stdin = io::stdin();
    let mut load_session = app_state::load_state;
    let mut save_session = app_state::save_state;
    let mut runtime = AppRuntime {
        stdout: &mut stdout,
        load_session: &mut load_session,
        save_session: &mut save_session,
    };
    run_with(command, &mut runtime, &mut stdin).await
}

async fn run_with(
    command: BoardCommand,
    runtime: &mut AppRuntime<'_>,
    stdin: &mut dyn Read,
) -> anyhow::Result<()> {
    match command {
        BoardCommand::View {
            project_id,
            limit,
            team,
        } => view(project_id, limit, team, runtime).await,
        BoardCommand::Column {
            project_id,
            column: column_selector,
            page,
            page_size,
            team,
        } => column(project_id, &column_selector, page, page_size, team, runtime).await,
        BoardCommand::Tasks { cmd } => match cmd {
            BoardTasksCommand::Create {
                project_id,
                title,
                body,
                body_stdin,
                status,
                priority,
                team,
            } => {
                tasks::create(
                    tasks::CreateOptions {
                        project_id,
                        title,
                        body,
                        body_stdin,
                        status,
                        priority,
                        team,
                    },
                    runtime,
                    stdin,
                )
                .await
            }
            BoardTasksCommand::Get {
                project_id,
                task,
                team,
            } => tasks::get(project_id, &task, team, runtime).await,
            BoardTasksCommand::Edit {
                project_id,
                task,
                title,
                body,
                body_stdin,
                team,
            } => {
                tasks::edit(
                    tasks::EditOptions {
                        project_id,
                        task,
                        title,
                        body,
                        body_stdin,
                        team,
                    },
                    runtime,
                    stdin,
                )
                .await
            }
            BoardTasksCommand::Move {
                project_id,
                task,
                column,
                team,
            } => tasks::move_task(project_id, &task, &column, team, runtime).await,
            BoardTasksCommand::Search {
                project_id,
                query,
                column,
                label,
                page,
                page_size,
                team,
            } => {
                tasks::search(
                    tasks::SearchOptions {
                        project_id,
                        query,
                        column,
                        label,
                        page,
                        page_size,
                        team,
                    },
                    runtime,
                )
                .await
            }
        },
    }
}

async fn view(
    project_id: Uuid,
    limit: usize,
    team: Option<Uuid>,
    runtime: &mut AppRuntime<'_>,
) -> anyhow::Result<()> {
    if limit == 0 {
        anyhow::bail!("--limit must be greater than zero.");
    }
    let loaded = load_board(project_id, team, runtime).await?;
    let mut rows: Vec<Vec<String>> = Vec::new();
    for column in &loaded.board.board.columns {
        if column.tasks.is_empty() {
            rows.push(vec![
                escape_terminal(&column.name).into_owned(),
                escape_terminal(&column.slug).into_owned(),
                "—".to_owned(),
                "—".to_owned(),
                "No tasks".to_owned(),
                "—".to_owned(),
            ]);
            continue;
        }
        rows.extend(column.tasks.iter().take(limit).map(|task| {
            vec![
                escape_terminal(&column.name).into_owned(),
                escape_terminal(&column.slug).into_owned(),
                escape_terminal(&task_slug(task)).into_owned(),
                escape_terminal(&task.id).into_owned(),
                escape_terminal(&task.title).into_owned(),
                task_labels(task),
            ]
        }));
        let hidden = column.tasks.len().saturating_sub(limit);
        if hidden > 0 {
            rows.push(vec![
                escape_terminal(&column.name).into_owned(),
                escape_terminal(&column.slug).into_owned(),
                "…".to_owned(),
                "—".to_owned(),
                format!("{hidden} more; use `vulcanum board column`"),
                "—".to_owned(),
            ]);
        }
    }
    let settings = render_table(
        &["AUTOMATION", "PICKUP", "IN PROGRESS", "IN REVIEW", "DONE"],
        vec![vec![
            if loaded.project.enabled {
                "enabled".to_owned()
            } else {
                "disabled".to_owned()
            },
            project_mark(&loaded.board, &loaded.project.pickup_column),
            project_mark(&loaded.board, &loaded.project.progress_column),
            project_mark(&loaded.board, &loaded.project.review_column),
            project_mark(&loaded.board, &loaded.project.done_column),
        ]],
    );
    let table = render_table(
        &[
            "COLUMN",
            "COLUMN SLUG",
            "TASK",
            "PROVIDER ID",
            "TITLE",
            "LABELS",
        ],
        rows,
    );
    writeln!(
        runtime.stdout,
        "Task board — {} ({})\nAutomation workflow\n{settings}\nTasks\n{table}",
        escape_terminal(&loaded.project.name),
        loaded.project.id
    )?;
    Ok(())
}

async fn column(
    project_id: Uuid,
    column_selector: &str,
    page: usize,
    page_size: usize,
    team: Option<Uuid>,
    runtime: &mut AppRuntime<'_>,
) -> anyhow::Result<()> {
    let loaded = load_board(project_id, team, runtime).await?;
    let column = find_column(&loaded.board, column_selector)?;
    let (start, end, total_pages) = page_bounds(column.tasks.len(), page, page_size)?;
    let rows = column.tasks[start..end]
        .iter()
        .map(|task| task_summary_row(task, Some(&column.name)))
        .collect();
    let table = render_table(
        &[
            "TASK",
            "PROVIDER ID",
            "TITLE",
            "COLUMN",
            "LABELS",
            "PRIORITY",
        ],
        rows,
    );
    writeln!(
        runtime.stdout,
        "{} — {} tasks, page {page}/{total_pages}\n{table}",
        escape_terminal(&column.name),
        column.tasks.len()
    )?;
    Ok(())
}
