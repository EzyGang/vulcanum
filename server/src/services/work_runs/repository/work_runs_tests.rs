use sqlx::PgPool;
use uuid::Uuid;

use crate::services::work_runs::errors::WorkRunsError;
use crate::services::work_runs::model::WorkRunStatus;
use crate::services::work_runs::repository::queries::InsertWorkRunParams;
use crate::services::work_runs::repository::WorkRunsRepository;
use crate::test_helpers::DEFAULT_TEAM_ID;

async fn insert_project_config(pool: &PgPool, external_project_id: &str) -> Uuid {
    let id = Uuid::new_v4();

    sqlx::query!(
        "INSERT INTO teams (id, name) VALUES ($1, $2) ON CONFLICT (id) DO NOTHING",
        DEFAULT_TEAM_ID,
        "Default team",
    )
    .execute(pool)
    .await
    .expect("Should ensure default team");

    sqlx::query!(
        "INSERT INTO project_configs (id, team_id, external_project_id, prompt_template, integration_type) VALUES ($1, $2, $3, $4, 'kaneo')",
        id,
        DEFAULT_TEAM_ID,
        external_project_id,
        "Review {{task_title}}",
    )
    .execute(pool)
    .await
    .expect("Should insert project config");

    id
}

fn insert_params(
    task_ref: &str,
    project_config_id: Uuid,
    status: WorkRunStatus,
) -> InsertWorkRunParams {
    InsertWorkRunParams {
        team_id: DEFAULT_TEAM_ID,
        external_task_ref: task_ref.to_owned(),
        project_config_id,
        prompt_text: "Review the PR".to_owned(),
        repo_url: String::new(),
        agents_md: String::new(),
        status,
        task_title: None,
        task_slug: None,
    }
}

#[sqlx::test]
async fn unique_active_task_prevents_duplicate_active(pool: PgPool) {
    let repo = WorkRunsRepository::new();
    let project_config_id = insert_project_config(&pool, "kaneo-proj-1").await;
    let task_ref = "task-123";

    let params = insert_params(task_ref, project_config_id, WorkRunStatus::Pending);

    repo.insert_work_run(&pool, params)
        .await
        .expect("First insert should succeed");

    let duplicate_params = insert_params(task_ref, project_config_id, WorkRunStatus::Pending);

    let result = repo.insert_work_run(&pool, duplicate_params).await;

    assert!(
        matches!(result, Err(WorkRunsError::Database(_))),
        "Second insert with same external_task_ref and active status should fail"
    );
}

#[sqlx::test]
async fn unique_active_task_allows_completed_with_same_ref(pool: PgPool) {
    let repo = WorkRunsRepository::new();
    let project_config_id = insert_project_config(&pool, "kaneo-proj-2").await;
    let task_ref = "task-456";

    let pending_params = insert_params(task_ref, project_config_id, WorkRunStatus::Pending);

    repo.insert_work_run(&pool, pending_params)
        .await
        .expect("First insert should succeed");

    let completed_params = insert_params(task_ref, project_config_id, WorkRunStatus::Completed);

    repo.insert_work_run(&pool, completed_params)
        .await
        .expect("Insert with status 'completed' should succeed (not in active set)");
}
