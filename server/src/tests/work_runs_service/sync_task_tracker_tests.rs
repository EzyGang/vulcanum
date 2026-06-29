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
        "in-review",
        false,
    );

    assert_eq!(column, Some("in-review"));
}

#[test]
fn implementation_success_with_review_work_stays_in_current_column() {
    let column = implementation_result_column(
        Some(FinishStatus::Completed),
        WorkRunStatus::Completed,
        "in-review",
        true,
    );

    assert_eq!(column, None);
}

#[test]
fn implementation_failed_finish_stays_in_current_column() {
    let column = implementation_result_column(
        Some(FinishStatus::Failed),
        WorkRunStatus::Failed,
        "in-review",
        false,
    );

    assert_eq!(column, None);
}

#[test]
fn implementation_exit_code_success_moves_to_target_column() {
    let column = implementation_result_column(None, WorkRunStatus::Completed, "in-review", false);

    assert_eq!(column, Some("in-review"));
}

#[test]
fn implementation_blocked_finish_does_not_move_columns() {
    let column = implementation_result_column(
        Some(FinishStatus::Blocked),
        WorkRunStatus::Failed,
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
        "in-review",
    );

    assert_eq!(column, Some("in-review"));
}

#[test]
fn review_failed_finish_does_not_move_columns() {
    let column = review_result_column(
        Some(FinishStatus::Failed),
        WorkRunStatus::Failed,
        "in-review",
    );

    assert_eq!(column, None);
}

#[test]
fn review_blocked_finish_does_not_move_columns() {
    let column = review_result_column(
        Some(FinishStatus::Blocked),
        WorkRunStatus::Failed,
        "in-review",
    );

    assert_eq!(column, None);
}
