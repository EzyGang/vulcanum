use uuid::Uuid;

use crate::services::work_runs::model::WorkRunStatus;
use crate::services::work_runs::repository::work_runs::InsertWorkRunParams;
use crate::services::work_runs::repository::WorkRunsRepository;

pub async fn insert_worker(pool: &sqlx::PgPool, name: &str) -> Uuid {
    let id = Uuid::new_v4();
    let hash = hex::encode([0u8; 32]);

    sqlx::query!(
        "INSERT INTO workers (id, name, refresh_token_hash, refresh_expires_at, status) VALUES ($1, $2, $3, NOW() + INTERVAL '30 days', 'idle'::worker_status)",
        id,
        name,
        hash,
    )
    .execute(pool)
    .await
    .expect("Should insert worker");

    id
}

pub async fn insert_project_config(pool: &sqlx::PgPool, kaneo_project_id: &str) -> Uuid {
    let id = Uuid::new_v4();

    sqlx::query!(
        "INSERT INTO project_configs (id, kaneo_project_id, prompt_template) VALUES ($1, $2, 'Review {{task_title}}')",
        id,
        kaneo_project_id,
    )
    .execute(pool)
    .await
    .expect("Should insert project config");

    id
}

pub async fn insert_pending_work_run(
    pool: &sqlx::PgPool,
    project_config_id: Uuid,
    task_ref: &str,
) -> Uuid {
    let repo = WorkRunsRepository::new();
    let params = InsertWorkRunParams {
        external_task_ref: task_ref.to_owned(),
        project_config_id,
        prompt_text: "Review the PR".to_owned(),
        status: WorkRunStatus::Pending,
    };

    repo.insert_work_run(pool, params)
        .await
        .expect("Should insert work_run")
        .id
}
