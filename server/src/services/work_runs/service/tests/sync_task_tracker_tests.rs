use vulcanum_shared::runtime::types::FinishStatus;

use crate::models::work_runs::model::WorkRunStatus;
use crate::services::work_runs::service::sync_task_tracker::{
    implementation_result_column, review_result_column,
};

#[test]
fn implementation_success_moves_to_target_column() {
    let column = implementation_result_column(
        Some(FinishStatus::Completed),
        WorkRunStatus::Completed,
        "to-do",
        "in-review",
        false,
    );

    assert_eq!(column, Some("in-review"));
}

#[test]
fn implementation_success_defers_target_column_when_review_cycle_is_pending() {
    let column = implementation_result_column(
        Some(FinishStatus::Completed),
        WorkRunStatus::Completed,
        "to-do",
        "in-review",
        true,
    );

    assert_eq!(column, None);
}

#[test]
fn implementation_failed_finish_moves_to_pickup_column() {
    let column = implementation_result_column(
        Some(FinishStatus::Failed),
        WorkRunStatus::Failed,
        "to-do",
        "in-review",
        false,
    );

    assert_eq!(column, Some("to-do"));
}

#[test]
fn implementation_exit_code_success_moves_to_target_column() {
    let column =
        implementation_result_column(None, WorkRunStatus::Completed, "to-do", "in-review", false);

    assert_eq!(column, Some("in-review"));
}

#[test]
fn implementation_blocked_finish_does_not_move_columns() {
    let column = implementation_result_column(
        Some(FinishStatus::Blocked),
        WorkRunStatus::Failed,
        "to-do",
        "in-review",
        false,
    );

    assert_eq!(column, None);
}

#[test]
fn review_success_moves_to_target_column() {
    let column = review_result_column(
        Some(FinishStatus::Completed),
        WorkRunStatus::Completed,
        "review-fix",
        "in-review",
    );

    assert_eq!(column, Some("in-review"));
}

#[test]
fn review_failed_finish_moves_to_review_pickup_column() {
    let column = review_result_column(
        Some(FinishStatus::Failed),
        WorkRunStatus::Failed,
        "review-fix",
        "in-review",
    );

    assert_eq!(column, Some("review-fix"));
}

#[test]
fn review_blocked_finish_does_not_move_columns() {
    let column = review_result_column(
        Some(FinishStatus::Blocked),
        WorkRunStatus::Failed,
        "review-fix",
        "in-review",
    );

    assert_eq!(column, None);
}
