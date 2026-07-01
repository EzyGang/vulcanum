use kaneo_cli::api::types::{Column as KaneoColumn, Label as KaneoLabel};

use crate::models::providers::model::{
    IntegrationBoard, IntegrationBoardColumn, IntegrationColumn, IntegrationLabel,
    IntegrationProject, IntegrationTask,
};
use crate::services::providers::kaneo::client::types::{
    KaneoBoardColumn, KaneoBoardResponse, KaneoTask, KaneoTaskLabel,
};

#[must_use]
pub(crate) fn column_name_to_slug(name: &str) -> String {
    name.trim()
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<&str>>()
        .join("-")
}

#[must_use]
pub(crate) fn kaneo_label_to_integration(label: &KaneoLabel) -> IntegrationLabel {
    IntegrationLabel {
        id: label.id.clone(),
        name: label.name.clone(),
        color: label.color.clone(),
    }
}

#[must_use]
pub(crate) fn kaneo_task_label_to_integration(label: &KaneoTaskLabel) -> IntegrationLabel {
    IntegrationLabel {
        id: label.id.clone(),
        name: label.name.clone(),
        color: label.color.clone(),
    }
}

#[must_use]
pub(crate) fn kaneo_task_to_integration(
    task: &KaneoTask,
    project_slug: Option<&str>,
) -> IntegrationTask {
    IntegrationTask {
        id: task.id.clone(),
        title: task.title.clone(),
        project_id: task.project_id.clone(),
        description: task.description.clone(),
        status: task.status.clone(),
        priority: task.priority.clone(),
        number: task.number,
        project_slug: project_slug.map(str::to_owned),
        assignee_name: task.assignee_name.clone(),
        created_at: task.created_at.clone(),
        updated_at: task.updated_at.clone(),
        labels: task
            .labels
            .iter()
            .map(kaneo_task_label_to_integration)
            .collect(),
    }
}

#[must_use]
pub(crate) fn kaneo_column_slug(name: &str, status: Option<&str>) -> String {
    status
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .unwrap_or_else(|| column_name_to_slug(name))
}

#[must_use]
pub(crate) fn kaneo_column_to_integration(column: &KaneoColumn) -> IntegrationColumn {
    IntegrationColumn {
        id: column.id.clone(),
        name: column.name.clone(),
        slug: kaneo_column_slug(&column.name, column.status.as_deref()),
        is_final: column.is_final,
    }
}

#[must_use]
pub(crate) fn kaneo_board_column_to_integration(
    column: &KaneoBoardColumn,
    project_slug: &str,
) -> IntegrationBoardColumn {
    IntegrationBoardColumn {
        id: column.id.clone(),
        name: column.name.clone(),
        slug: kaneo_column_slug(&column.name, column.status.as_deref()),
        is_final: column.is_final,
        tasks: column
            .tasks
            .iter()
            .map(|task| kaneo_task_to_integration(task, Some(project_slug)))
            .collect(),
    }
}

#[must_use]
pub(crate) fn kaneo_board_to_integration(board: KaneoBoardResponse) -> IntegrationBoard {
    let data = board.data;
    let project = IntegrationProject {
        id: data.id,
        name: data.name,
        slug: data.slug,
        workspace_id: None,
    };
    let mut columns = data
        .columns
        .iter()
        .map(|column| kaneo_board_column_to_integration(column, &project.slug))
        .collect::<Vec<IntegrationBoardColumn>>();

    if !data.planned_tasks.is_empty() {
        columns.push(IntegrationBoardColumn {
            id: "planned".to_owned(),
            name: "Planned".to_owned(),
            slug: "planned".to_owned(),
            is_final: Some(false),
            tasks: data
                .planned_tasks
                .iter()
                .map(|task| kaneo_task_to_integration(task, Some(&project.slug)))
                .collect(),
        });
    }

    if !data.archived_tasks.is_empty() {
        columns.push(IntegrationBoardColumn {
            id: "archived".to_owned(),
            name: "Archived".to_owned(),
            slug: "archived".to_owned(),
            is_final: Some(true),
            tasks: data
                .archived_tasks
                .iter()
                .map(|task| kaneo_task_to_integration(task, Some(&project.slug)))
                .collect(),
        });
    }

    IntegrationBoard {
        project,
        columns,
        labels: Vec::new(),
    }
}
