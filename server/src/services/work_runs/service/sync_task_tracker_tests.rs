use vulcanum_shared::runtime::types::FinishStatus;

use crate::services::work_runs::model::WorkRunStatus;
use crate::services::work_runs::service::sync_task_tracker::implementation_result_column;

#[test]
fn implementation_success_moves_to_target_column() {
    let column = implementation_result_column(
        Some(FinishStatus::Completed),
        WorkRunStatus::Completed,
        "to-do",
        "in-review",
    );

    assert_eq!(column, "in-review");
}

#[test]
fn implementation_failed_finish_moves_to_pickup_column() {
    let column = implementation_result_column(
        Some(FinishStatus::Failed),
        WorkRunStatus::Failed,
        "to-do",
        "in-review",
    );

    assert_eq!(column, "to-do");
}

#[test]
fn implementation_exit_code_success_moves_to_target_column() {
    let column = implementation_result_column(None, WorkRunStatus::Completed, "to-do", "in-review");

    assert_eq!(column, "in-review");
}
