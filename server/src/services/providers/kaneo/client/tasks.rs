mod board;
mod mutations;

#[cfg(test)]
pub(crate) use mutations::update_task_request;

use std::collections::HashSet;

use kaneo_cli::api::types::{Comment, CreateTaskBody};

use crate::services::providers::kaneo::client::types::{KaneoBoardResponse, KaneoTask};
use crate::services::providers::kaneo::client::{log_kaneo_result, KaneoClient};
use crate::services::providers::kaneo::errors::{api_err, KaneoError};

const FETCH_TASKS_LIMIT: u32 = 200;

fn board_task_count(board: &KaneoBoardResponse) -> usize {
    board
        .data
        .columns
        .iter()
        .map(|column| column.tasks.len())
        .sum::<usize>()
        + board.data.planned_tasks.len()
        + board.data.archived_tasks.len()
}

fn collect_seen_task_ids(board: &KaneoBoardResponse, seen_task_ids: &mut HashSet<String>) {
    for column in &board.data.columns {
        for task in &column.tasks {
            seen_task_ids.insert(task.id.clone());
        }
    }
    for task in board
        .data
        .planned_tasks
        .iter()
        .chain(&board.data.archived_tasks)
    {
        seen_task_ids.insert(task.id.clone());
    }
}

fn merge_board_page(
    merged: &mut KaneoBoardResponse,
    page: &mut KaneoBoardResponse,
    seen_task_ids: &mut HashSet<String>,
) -> usize {
    let mut new_task_count = 0;

    for page_column in &mut page.data.columns {
        let Some(merged_column) = merged
            .data
            .columns
            .iter_mut()
            .find(|column| column.id == page_column.id)
        else {
            new_task_count += push_new_tasks(
                &mut merged.data.planned_tasks,
                &mut page_column.tasks,
                seen_task_ids,
            );
            continue;
        };

        new_task_count += push_new_tasks(
            &mut merged_column.tasks,
            &mut page_column.tasks,
            seen_task_ids,
        );
    }

    new_task_count += push_new_tasks(
        &mut merged.data.planned_tasks,
        &mut page.data.planned_tasks,
        seen_task_ids,
    );
    new_task_count += push_new_tasks(
        &mut merged.data.archived_tasks,
        &mut page.data.archived_tasks,
        seen_task_ids,
    );

    new_task_count
}

fn push_new_tasks(
    target: &mut Vec<KaneoTask>,
    tasks: &mut Vec<KaneoTask>,
    seen_task_ids: &mut HashSet<String>,
) -> usize {
    let original_len = target.len();
    target.extend(
        tasks
            .drain(..)
            .filter(|task| seen_task_ids.insert(task.id.clone())),
    );
    target.len() - original_len
}

pub(crate) fn filter_tasks_in_column(
    board: KaneoBoardResponse,
    column_slug: &str,
) -> Vec<KaneoTask> {
    board
        .data
        .columns
        .into_iter()
        .flat_map(|column| column.tasks)
        .chain(board.data.planned_tasks)
        .chain(board.data.archived_tasks)
        .filter(|task| task.status == column_slug)
        .collect()
}
