use kaneo_cli::api::types::{BoardColumn, BoardData, BoardResponse, Column, Task};

use crate::services::providers::kaneo::mapping::{
    column_name_to_slug, kaneo_board_to_integration, kaneo_column_slug, kaneo_column_to_integration,
};

fn task(id: &str, status: &str) -> Task {
    Task {
        id: id.to_owned(),
        project_id: "project-1".to_owned(),
        position: None,
        number: Some(7),
        user_id: None,
        title: format!("Task {id}"),
        description: Some("body".to_owned()),
        status: status.to_owned(),
        priority: "low".to_owned(),
        due_date: None,
        created_at: "2026-01-01T00:00:00Z".to_owned(),
        start_date: None,
        updated_at: Some("2026-01-02T00:00:00Z".to_owned()),
        column_id: None,
        assignee_name: Some("Agent".to_owned()),
        assignee_id: None,
        assignee_image: None,
    }
}

#[test]
fn column_name_to_slug_uses_lowercase_hyphenated_name() {
    assert_eq!(column_name_to_slug("To Do"), "to-do");
    assert_eq!(column_name_to_slug("  In   Progress  "), "in-progress");
}

#[test]
fn kaneo_column_slug_prefers_status_slug() {
    assert_eq!(kaneo_column_slug("Pickup", Some("to-do")), "to-do");
    assert_eq!(kaneo_column_slug("In Progress", None), "in-progress");
}

#[test]
fn kaneo_column_mapping_prefers_status_slug() {
    let column = Column {
        id: "column-1".to_owned(),
        project_id: "project-1".to_owned(),
        name: "In Review".to_owned(),
        position: 2,
        status: Some("in-review".to_owned()),
        icon: Some("eye".to_owned()),
        color: Some("purple".to_owned()),
        is_final: Some(false),
    };

    let result = kaneo_column_to_integration(&column);

    assert_eq!(result.id, "column-1");
    assert_eq!(result.name, "In Review");
    assert_eq!(result.slug, "in-review");
    assert_eq!(result.is_final, Some(false));
}

#[test]
fn kaneo_board_mapping_keeps_provider_columns_and_overflow_tasks() {
    let board = BoardResponse {
        data: BoardData {
            id: "project-1".to_owned(),
            name: "Project".to_owned(),
            slug: "project".to_owned(),
            columns: vec![BoardColumn {
                id: "column-1".to_owned(),
                name: "In Progress".to_owned(),
                status: Some("in-progress".to_owned()),
                is_final: Some(false),
                tasks: vec![task("task-1", "in-progress")],
            }],
            planned_tasks: vec![task("task-2", "planned")],
            archived_tasks: vec![task("task-3", "archived")],
        },
    };

    let result = kaneo_board_to_integration(board);

    assert_eq!(result.project.id, "project-1");
    assert_eq!(result.columns.len(), 3);
    assert_eq!(result.columns[0].slug, "in-progress");
    assert_eq!(
        result.columns[0].tasks[0].assignee_name.as_deref(),
        Some("Agent")
    );
    assert_eq!(result.columns[1].slug, "planned");
    assert_eq!(result.columns[2].slug, "archived");
    assert_eq!(result.columns[2].is_final, Some(true));
}
