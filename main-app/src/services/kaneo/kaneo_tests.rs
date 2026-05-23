use super::client::filter_tasks_in_column;
use super::client::log_kaneo_result;
use super::client::KaneoClient;
use super::errors::KaneoError;
use kaneo_cli::api::types::{BoardColumn, BoardData, BoardResponse, Task};

#[test]
fn test_filter_tasks_in_column_exact_match() {
    let task = Task {
        id: "t1".to_owned(),
        project_id: "p1".to_owned(),
        position: None,
        number: None,
        user_id: None,
        title: "Test task".to_owned(),
        description: None,
        status: "todo".to_owned(),
        priority: "low".to_owned(),
        due_date: None,
        created_at: "2024-01-01".to_owned(),
        start_date: None,
        updated_at: None,
        column_id: None,
        assignee_name: None,
        assignee_id: None,
        assignee_image: None,
    };

    let board = BoardResponse {
        data: BoardData {
            id: "p1".to_owned(),
            name: "Project".to_owned(),
            slug: "proj".to_owned(),
            columns: vec![BoardColumn {
                id: "c1".to_owned(),
                name: "To Do".to_owned(),
                is_final: None,
                tasks: vec![task.clone()],
            }],
            planned_tasks: vec![],
            archived_tasks: vec![],
        },
    };

    let result = filter_tasks_in_column(board, "To Do");
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].id, "t1");
}

#[test]
fn test_filter_tasks_in_column_case_insensitive() {
    let task = Task {
        id: "t2".to_owned(),
        project_id: "p1".to_owned(),
        position: None,
        number: None,
        user_id: None,
        title: "Another".to_owned(),
        description: None,
        status: "in_progress".to_owned(),
        priority: "high".to_owned(),
        due_date: None,
        created_at: "2024-01-01".to_owned(),
        start_date: None,
        updated_at: None,
        column_id: None,
        assignee_name: None,
        assignee_id: None,
        assignee_image: None,
    };

    let board = BoardResponse {
        data: BoardData {
            id: "p1".to_owned(),
            name: "Project".to_owned(),
            slug: "proj".to_owned(),
            columns: vec![BoardColumn {
                id: "c2".to_owned(),
                name: "In Progress".to_owned(),
                is_final: None,
                tasks: vec![task.clone()],
            }],
            planned_tasks: vec![],
            archived_tasks: vec![],
        },
    };

    let result = filter_tasks_in_column(board, "in progress");
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].id, "t2");
}

#[test]
fn test_filter_tasks_in_column_not_found_returns_empty() {
    let board = BoardResponse {
        data: BoardData {
            id: "p1".to_owned(),
            name: "Project".to_owned(),
            slug: "proj".to_owned(),
            columns: vec![BoardColumn {
                id: "c1".to_owned(),
                name: "To Do".to_owned(),
                is_final: None,
                tasks: vec![],
            }],
            planned_tasks: vec![],
            archived_tasks: vec![],
        },
    };

    let result = filter_tasks_in_column(board, "Done");
    assert!(result.is_empty());
}

#[test]
fn test_filter_tasks_in_column_multiple_columns_selects_correct_one() {
    let task_todo = Task {
        id: "t3".to_owned(),
        project_id: "p1".to_owned(),
        position: None,
        number: None,
        user_id: None,
        title: "Todo task".to_owned(),
        description: None,
        status: "todo".to_owned(),
        priority: "low".to_owned(),
        due_date: None,
        created_at: "2024-01-01".to_owned(),
        start_date: None,
        updated_at: None,
        column_id: None,
        assignee_name: None,
        assignee_id: None,
        assignee_image: None,
    };

    let task_done = Task {
        id: "t4".to_owned(),
        project_id: "p1".to_owned(),
        position: None,
        number: None,
        user_id: None,
        title: "Done task".to_owned(),
        description: None,
        status: "done".to_owned(),
        priority: "low".to_owned(),
        due_date: None,
        created_at: "2024-01-01".to_owned(),
        start_date: None,
        updated_at: None,
        column_id: None,
        assignee_name: None,
        assignee_id: None,
        assignee_image: None,
    };

    let board = BoardResponse {
        data: BoardData {
            id: "p1".to_owned(),
            name: "Project".to_owned(),
            slug: "proj".to_owned(),
            columns: vec![
                BoardColumn {
                    id: "c1".to_owned(),
                    name: "To Do".to_owned(),
                    is_final: None,
                    tasks: vec![task_todo],
                },
                BoardColumn {
                    id: "c2".to_owned(),
                    name: "Done".to_owned(),
                    is_final: Some(true),
                    tasks: vec![task_done],
                },
            ],
            planned_tasks: vec![],
            archived_tasks: vec![],
        },
    };

    let result = filter_tasks_in_column(board, "Done");
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].id, "t4");
}

#[test]
fn test_kaneo_error_display() {
    let api_err = KaneoError::Api("something went wrong".to_owned());
    assert_eq!(api_err.to_string(), "kaneo API error: something went wrong");

    let col_err = KaneoError::ColumnNotFound("backlog".to_owned());
    assert_eq!(col_err.to_string(), "column not found in project: backlog");
}

#[test]
fn test_kaneo_client_construction() {
    let client = KaneoClient::new("cloud.kaneo.app".to_owned(), "sk-test-key".to_owned());
    let _ = client;
}

#[cfg(test)]
mod tracing_tests {
    use super::*;
    use tracing_test::traced_test;

    #[traced_test]
    #[test]
    fn test_log_kaneo_result_success_emits_info() {
        let result: Result<(), KaneoError> = Ok(());
        log_kaneo_result("GET", "/test", 200, 42, &result);
        assert!(logs_contain("Kaneo API call succeeded"));
        assert!(logs_contain("GET"));
        assert!(logs_contain("/test"));
        assert!(logs_contain("42"));
    }

    #[traced_test]
    #[test]
    fn test_log_kaneo_result_error_emits_warn() {
        let result: Result<(), KaneoError> = Err(KaneoError::Api("boom".to_owned()));
        log_kaneo_result("POST", "/fail", 500, 99, &result);
        assert!(logs_contain("Kaneo API call failed"));
        assert!(logs_contain("POST"));
        assert!(logs_contain("boom"));
    }

    #[traced_test]
    #[test]
    fn test_log_kaneo_result_error_no_method_in_success() {
        let result: Result<(), KaneoError> = Err(KaneoError::Api("err".to_owned()));
        log_kaneo_result("PUT", "/put", 200, 10, &result);
        assert!(!logs_contain("succeeded"));
        assert!(logs_contain("failed"));
    }
}
