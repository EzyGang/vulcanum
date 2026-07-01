use super::client::filter_tasks_in_column;
use super::client::types::{KaneoBoardColumn, KaneoBoardData, KaneoBoardResponse, KaneoTask};
use super::client::{log_kaneo_result, KaneoClient};
use super::errors::KaneoError;

fn task(id: &str, status: &str) -> KaneoTask {
    KaneoTask {
        id: id.to_owned(),
        project_id: "p1".to_owned(),
        number: None,
        title: format!("Task {id}"),
        description: None,
        status: status.to_owned(),
        priority: "low".to_owned(),
        created_at: "2024-01-01".to_owned(),
        updated_at: None,
        assignee_name: None,
        labels: Vec::new(),
    }
}

fn board(columns: Vec<KaneoBoardColumn>) -> KaneoBoardResponse {
    KaneoBoardResponse {
        data: KaneoBoardData {
            id: "p1".to_owned(),
            name: "Project".to_owned(),
            slug: "proj".to_owned(),
            columns,
            planned_tasks: Vec::new(),
            archived_tasks: Vec::new(),
        },
    }
}

fn column(id: &str, name: &str, status: Option<&str>, tasks: Vec<KaneoTask>) -> KaneoBoardColumn {
    KaneoBoardColumn {
        id: id.to_owned(),
        name: name.to_owned(),
        status: status.map(str::to_owned),
        is_final: None,
        tasks,
    }
}

#[test]
fn filter_tasks_in_column_exact_match() {
    let board = board(vec![column(
        "c1",
        "To Do",
        Some("to-do"),
        vec![task("t1", "to-do")],
    )]);

    let result = filter_tasks_in_column(board, "to-do");

    assert_eq!(result.len(), 1);
    assert_eq!(result[0].id, "t1");
}

#[test]
fn filter_tasks_in_column_uses_task_status_when_column_status_missing() {
    let board = board(vec![column(
        "c1",
        "To Do",
        None,
        vec![task("t-missing-column-status", "to-do")],
    )]);

    let result = filter_tasks_in_column(board, "to-do");

    assert_eq!(result.len(), 1);
    assert_eq!(result[0].id, "t-missing-column-status");
}

#[test]
fn filter_tasks_in_column_not_found_returns_empty() {
    let board = board(vec![column("c1", "To Do", Some("to-do"), Vec::new())]);

    let result = filter_tasks_in_column(board, "done");

    assert!(result.is_empty());
}

#[test]
fn filter_tasks_in_column_multiple_columns_selects_correct_one() {
    let mut done = column("c2", "Done", Some("done"), vec![task("t4", "done")]);
    done.is_final = Some(true);
    let board = board(vec![
        column("c1", "To Do", Some("to-do"), vec![task("t3", "todo")]),
        done,
    ]);

    let result = filter_tasks_in_column(board, "done");

    assert_eq!(result.len(), 1);
    assert_eq!(result[0].id, "t4");
}

#[test]
fn filter_tasks_in_column_uses_status_slug() {
    let board = board(vec![column(
        "c3",
        "In Review",
        Some("in-review"),
        vec![task("t5", "in-review")],
    )]);

    let result = filter_tasks_in_column(board, "in-review");

    assert_eq!(result.len(), 1);
    assert_eq!(result[0].id, "t5");
}

#[test]
fn board_deserializes_embedded_task_labels_without_metadata() {
    let board: KaneoBoardResponse = serde_json::from_str(
        r##"{
            "data": {
                "id": "p1",
                "name": "Project",
                "slug": "PROJ",
                "columns": [
                    {
                        "id": "done",
                        "slug": "done",
                        "name": "Done",
                        "isFinal": true,
                        "tasks": [
                            {
                                "id": "t1",
                                "projectId": "p1",
                                "number": 1,
                                "title": "Task with label",
                                "description": null,
                                "status": "done",
                                "priority": "high",
                                "createdAt": "2026-06-16T19:19:51.735Z",
                                "labels": [
                                    {
                                        "id": "label-1",
                                        "name": "Regression",
                                        "color": "#ff00ff"
                                    }
                                ]
                            }
                        ]
                    }
                ],
                "plannedTasks": [],
                "archivedTasks": []
            }
        }"##,
    )
    .unwrap();

    let task = &board.data.columns[0].tasks[0];

    assert_eq!(task.labels.len(), 1);
    assert_eq!(task.labels[0].id, "label-1");
    assert_eq!(task.labels[0].name, "Regression");
    assert_eq!(task.labels[0].color, "#ff00ff");
}

#[test]
fn kaneo_error_display() {
    let api_err = KaneoError::Api("something went wrong".to_owned());
    assert_eq!(api_err.to_string(), "kaneo API error: something went wrong");

    let col_err = KaneoError::ColumnNotFound("backlog".to_owned());
    assert_eq!(col_err.to_string(), "column not found in project: backlog");
}

#[test]
fn kaneo_client_construction() {
    let client = KaneoClient::new("cloud.kaneo.app".to_owned(), "sk-test-key".to_owned());
    let _ = client;
}

#[cfg(test)]
mod tracing_tests {
    use super::{log_kaneo_result, KaneoError};
    use tracing_test::traced_test;

    #[traced_test]
    #[test]
    fn log_kaneo_result_success_emits_info() {
        let result: Result<(), KaneoError> = Ok(());
        log_kaneo_result("GET", "/test", 42, &result);
        assert!(logs_contain("Kaneo API request succeeded"));
        assert!(logs_contain("GET"));
        assert!(logs_contain("/test"));
        assert!(logs_contain("42"));
    }

    #[traced_test]
    #[test]
    fn log_kaneo_result_error_emits_warn() {
        let result: Result<(), KaneoError> = Err(KaneoError::Api("boom".to_owned()));
        log_kaneo_result("POST", "/fail", 99, &result);
        assert!(logs_contain("Kaneo API request failed"));
        assert!(logs_contain("POST"));
        assert!(logs_contain("boom"));
    }

    #[traced_test]
    #[test]
    fn log_kaneo_result_error_no_method_in_success() {
        let result: Result<(), KaneoError> = Err(KaneoError::Api("err".to_owned()));
        log_kaneo_result("PUT", "/put", 10, &result);
        assert!(!logs_contain("succeeded"));
        assert!(logs_contain("failed"));
    }
}
