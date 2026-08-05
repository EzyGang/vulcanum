use chrono::Utc;
use uuid::Uuid;

use crate::db::work_runs::queries::InsertWorkRunParams;
use crate::db::work_runs::WorkRunsRepository;
use crate::models::providers::model::IntegrationLabel;
use crate::models::work_runs::model::{WorkRun, WorkRunStatus, WorkRunType};
use crate::services::work_runs::service::lifecycle_labels::result::lifecycle_label_for_terminal_result;
use crate::services::work_runs::service::lifecycle_labels::{
    lifecycle_label_template, LifecycleLabelState,
};
use crate::services::work_runs::service::spawn_review::ReviewSpawnOutcome;
use crate::test_helpers;

#[test]
fn lifecycle_template_ignores_copy_attached_to_another_task() {
    let labels = vec![
        IntegrationLabel {
            id: "attached-copy".to_owned(),
            name: "Review needed".to_owned(),
            color: "#D97706".to_owned(),
            task_id: Some("task-1".to_owned()),
        },
        IntegrationLabel {
            id: "workspace-template".to_owned(),
            name: "Review needed".to_owned(),
            color: "#D97706".to_owned(),
            task_id: None,
        },
    ];

    let template = lifecycle_label_template(&labels, LifecycleLabelState::ReviewNeeded);

    assert_eq!(
        template.map(|label| label.id.as_str()),
        Some("workspace-template")
    );
}

#[test]
fn standalone_and_tracker_backed_github_reviews_have_distinct_lifecycle_sync_behavior() {
    assert!(!review_run(None, Some("delivery-1")).syncs_lifecycle_labels());
    assert!(review_run(Some(Uuid::new_v4()), Some("delivery-2")).syncs_lifecycle_labels());
}

#[test]
fn implementation_completion_replaces_running_label_with_review_state() {
    let lifecycle_label = lifecycle_label_for_terminal_result(
        WorkRunType::Implementation,
        WorkRunStatus::Completed,
        Some(ReviewSpawnOutcome::ReviewRunning),
    );

    assert_eq!(lifecycle_label, Some(LifecycleLabelState::ReviewRunning));
}

#[sqlx::test]
async fn tracker_backed_review_completion_keeps_review_running_with_active_sibling(
    pool: sqlx::PgPool,
) {
    let (config_id, parent_id) = insert_implementation(&pool).await;
    insert_review_sibling(&pool, config_id, parent_id, WorkRunStatus::Pending).await;
    let state = test_helpers::state::build_state(pool).await;

    let lifecycle_label = state
        .jobs
        .review_completion_lifecycle_label(&review_run(Some(parent_id), Some("delivery-1")))
        .await;

    assert_eq!(lifecycle_label, Some(LifecycleLabelState::ReviewRunning));
}

#[sqlx::test]
async fn tracker_backed_review_completion_becomes_ready_after_terminal_siblings(
    pool: sqlx::PgPool,
) {
    let (config_id, parent_id) = insert_implementation(&pool).await;
    insert_review_sibling(&pool, config_id, parent_id, WorkRunStatus::Completed).await;
    let state = test_helpers::state::build_state(pool).await;

    let lifecycle_label = state
        .jobs
        .review_completion_lifecycle_label(&review_run(Some(parent_id), Some("delivery-1")))
        .await;

    assert_eq!(lifecycle_label, Some(LifecycleLabelState::ReadyForHuman));
}

#[sqlx::test]
async fn tracker_backed_review_completion_needs_attention_after_failed_sibling(pool: sqlx::PgPool) {
    let (config_id, parent_id) = insert_implementation(&pool).await;
    insert_review_sibling(&pool, config_id, parent_id, WorkRunStatus::Failed).await;
    let state = test_helpers::state::build_state(pool).await;

    let lifecycle_label = state
        .jobs
        .review_completion_lifecycle_label(&review_run(Some(parent_id), Some("delivery-1")))
        .await;

    assert_eq!(lifecycle_label, Some(LifecycleLabelState::NeedsAttention));
}

async fn insert_implementation(pool: &sqlx::PgPool) -> (Uuid, Uuid) {
    let config_id = test_helpers::project_configs::insert_project_config(pool, "project-1").await;
    let run = WorkRunsRepository::new()
        .insert_work_run(
            pool,
            InsertWorkRunParams {
                team_id: test_helpers::DEFAULT_TEAM_ID,
                external_task_ref: "task-1".to_owned(),
                task_title: None,
                task_slug: None,
                project_config_id: config_id,
                repo_full_names: Vec::new(),
                status: WorkRunStatus::Completed,
                work_type: WorkRunType::Implementation,
                parent_work_run_id: None,
                review_target_pr_url: None,
                review_target_repo_full_name: None,
                github_installation_id: None,
                github_delivery_id: None,
            },
        )
        .await
        .expect("insert implementation");

    (config_id, run.id)
}

async fn insert_review_sibling(
    pool: &sqlx::PgPool,
    config_id: Uuid,
    parent_id: Uuid,
    status: WorkRunStatus,
) {
    WorkRunsRepository::new()
        .insert_work_run(
            pool,
            InsertWorkRunParams {
                team_id: test_helpers::DEFAULT_TEAM_ID,
                external_task_ref: "task-1".to_owned(),
                task_title: None,
                task_slug: None,
                project_config_id: config_id,
                repo_full_names: Vec::new(),
                status,
                work_type: WorkRunType::PullRequestReview,
                parent_work_run_id: Some(parent_id),
                review_target_pr_url: None,
                review_target_repo_full_name: None,
                github_installation_id: Some(1),
                github_delivery_id: Some("delivery-1".to_owned()),
            },
        )
        .await
        .expect("insert review sibling");
}

fn review_run(parent_work_run_id: Option<Uuid>, github_delivery_id: Option<&str>) -> WorkRun {
    let now = Utc::now();

    WorkRun {
        id: Uuid::new_v4(),
        team_id: test_helpers::DEFAULT_TEAM_ID,
        external_task_ref: "task-1".to_owned(),
        task_title: None,
        task_slug: None,
        project_config_id: Uuid::new_v4(),
        worker_id: None,
        status: WorkRunStatus::Completed,
        work_type: WorkRunType::PullRequestReview,
        parent_work_run_id,
        review_target_pr_url: None,
        review_target_repo_full_name: None,
        github_installation_id: Some(1),
        github_delivery_id: github_delivery_id.map(str::to_owned),
        result_pr_url: None,
        result_exit_code: None,
        tokens_used: None,
        duration_ms: None,
        input_tokens: None,
        output_tokens: None,
        cache_read_tokens: None,
        cache_write_tokens: None,
        model_used: None,
        finish_status: None,
        result_summary: None,
        finish_blocked_reason: None,
        finish_next_column: None,
        created_at: now,
        updated_at: now,
    }
}
