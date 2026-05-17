use sqlx::PgPool;
use uuid::Uuid;

use crate::services::work_runs::errors::WorkRunsError;
use crate::services::work_runs::repository::work_runs::InsertWorkRunParams;
use crate::services::work_runs::repository::WorkRunsRepository;

async fn insert_project_config(pool: &PgPool, kaneo_project_id: &str) -> Uuid {
    let id = Uuid::new_v4();

    sqlx::query(
        "INSERT INTO project_configs (id, kaneo_project_id, prompt_template) VALUES ($1, $2, $3)",
    )
    .bind(id)
    .bind(kaneo_project_id)
    .bind("Review {{task_title}}")
    .execute(pool)
    .await
    .expect("Should insert project config");

    id
}

#[sqlx::test]
async fn unique_active_task_prevents_duplicate_active(pool: PgPool) {
    let repo = WorkRunsRepository::new();
    let project_config_id = insert_project_config(&pool, "kaneo-proj-1").await;
    let task_ref = "task-123";

    let params = InsertWorkRunParams {
        external_task_ref: task_ref.to_owned(),
        project_config_id,
        prompt_text: "Review the PR".to_owned(),
        status: "pending".to_owned(),
    };

    repo.insert_work_run(&pool, params)
        .await
        .expect("First insert should succeed");

    let duplicate_params = InsertWorkRunParams {
        external_task_ref: task_ref.to_owned(),
        project_config_id,
        prompt_text: "Review another PR".to_owned(),
        status: "pending".to_owned(),
    };

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

    let pending_params = InsertWorkRunParams {
        external_task_ref: task_ref.to_owned(),
        project_config_id,
        prompt_text: "Review the PR".to_owned(),
        status: "pending".to_owned(),
    };

    repo.insert_work_run(&pool, pending_params)
        .await
        .expect("First insert should succeed");

    let completed_params = InsertWorkRunParams {
        external_task_ref: task_ref.to_owned(),
        project_config_id,
        prompt_text: "Review another PR".to_owned(),
        status: "completed".to_owned(),
    };

    repo.insert_work_run(&pool, completed_params)
        .await
        .expect("Insert with status 'completed' should succeed (not in active set)");
}
