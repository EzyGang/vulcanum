use vulcanum_shared::runtime::types::FinishStatus;

use crate::services::work_runs::model::WorkRunStatus;
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

#[test]
fn actionable_review_moves_to_progress_column() {
    let body = "## CRITICAL\n- None\n\n## WARNINGS\n- Missing auth\n\n## SUGGESTIONS\n- None";
    let column = review_result_column(false, Some(body), "in-progress");

    assert_eq!(column, Some("in-progress"));
}

#[test]
fn clean_review_does_not_move_column() {
    let body = "## CRITICAL\n- None\n\n## WARNINGS\n- None\n\n## SUGGESTIONS\n- Rename later";
    let column = review_result_column(false, Some(body), "in-progress");

    assert_eq!(column, None);
}

#[test]
fn existing_review_does_not_move_column() {
    let body = "## CRITICAL\n- Bug\n\n## WARNINGS\n- None\n\n## SUGGESTIONS\n- None";
    let column = review_result_column(true, Some(body), "in-progress");

    assert_eq!(column, None);
}
