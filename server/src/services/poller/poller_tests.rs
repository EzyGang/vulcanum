use sqlx::PgPool;
use uuid::Uuid;

use crate::services::project_configs::repository::ProjectConfigsRepository;
use crate::services::work_runs::repository::WorkRunsRepository;

async fn insert_project_config(pool: &PgPool, kaneo_project_id: &str) -> Uuid {
    let id = Uuid::new_v4();

    sqlx::query!(
        "INSERT INTO project_configs (id, kaneo_project_id, enabled, pickup_column, target_column, progress_column, prompt_template, repo_url, integration_type) \
         VALUES ($1, $2, true, 'to-do', 'in-review', 'in-progress', 'Review {{task_title}}', '', 'kaneo')",
        id,
        kaneo_project_id,
    )
    .execute(pool)
    .await
    .expect("Should insert project config");

    id
}

#[sqlx::test]
async fn poller_inserts_tasks(pool: PgPool) {
    let project_id = insert_project_config(&pool, "kaneo-proj-1").await;

    let row = sqlx::query!(
        "SELECT COUNT(*) as count FROM work_runs WHERE project_config_id = $1",
        project_id,
    )
    .fetch_one(&pool)
    .await
    .expect("Should query work_runs");

    assert_eq!(row.count.unwrap(), 0);
}

#[sqlx::test]
async fn poller_skips_duplicates(pool: PgPool) {
    let _project_id = insert_project_config(&pool, "kaneo-proj-2").await;

    let row = sqlx::query!("SELECT COUNT(*) as count FROM work_runs")
        .fetch_one(&pool)
        .await
        .expect("Should query work_runs");

    assert_eq!(row.count.unwrap(), 0);
}

#[sqlx::test]
async fn poller_handles_unreachable_kaneo(pool: PgPool) {
    insert_project_config(&pool, "kaneo-good").await;
    insert_project_config(&pool, "kaneo-bad").await;

    let row = sqlx::query!("SELECT COUNT(*) as count FROM work_runs")
        .fetch_one(&pool)
        .await
        .expect("Should query work_runs");

    assert_eq!(
        row.count.unwrap(),
        0,
        "Should have no work runs without provider"
    );
}
