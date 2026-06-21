use std::sync::Arc;

use sqlx::PgPool;

use crate::services::poller::poller_test_support::{
    build_service, insert_active_run, insert_project_config, insert_provider, make_task,
    MockTaskFetcher,
};
use crate::services::providers::errors::IntegrationError;

#[sqlx::test]
async fn poller_inserts_tasks(pool: PgPool) {
    let mock = Arc::new(MockTaskFetcher::new());
    let provider_id = insert_provider(&pool).await;
    let project_id = insert_project_config(&pool, "kaneo-proj-1", provider_id).await;

    mock.set_tasks(
        "kaneo-proj-1",
        "to-do",
        vec![
            make_task("task-1", "Fix login bug"),
            make_task("task-2", "Add dark mode"),
        ],
    )
    .await;

    let service = build_service(mock, pool.clone());
    service.poll_once().await;

    let rows = sqlx::query!(
        "SELECT external_task_ref, prompt_text, task_slug, task_title FROM work_runs \
         WHERE project_config_id = $1 ORDER BY external_task_ref",
        project_id,
    )
    .fetch_all(&pool)
    .await
    .expect("Should query work_runs");

    assert_eq!(rows.len(), 1, "Should respect default project capacity");
    assert_eq!(rows[0].external_task_ref, "task-1");
    assert_eq!(rows[0].task_slug.as_deref(), Some("tst-1"));
    assert_eq!(rows[0].task_title.as_deref(), Some("Fix login bug"));
    assert!(rows[0].prompt_text.starts_with("Review Fix login bug"));
    assert!(rows[0].prompt_text.contains("Debian-based container"));
}

#[sqlx::test]
async fn poller_skips_duplicates(pool: PgPool) {
    let mock = Arc::new(MockTaskFetcher::new());
    let provider_id = insert_provider(&pool).await;

    insert_project_config(&pool, "kaneo-proj-2", provider_id).await;

    mock.set_tasks(
        "kaneo-proj-2",
        "to-do",
        vec![make_task("task-dup", "Fix login bug")],
    )
    .await;

    let service = build_service(mock.clone(), pool.clone());

    service.poll_once().await;
    service.poll_once().await;

    let row = sqlx::query!(
        "SELECT COUNT(*) as count FROM work_runs WHERE external_task_ref = $1",
        "task-dup",
    )
    .fetch_one(&pool)
    .await
    .expect("Should query work_runs");

    assert_eq!(
        row.count.unwrap(),
        1,
        "Should not insert duplicate work_run"
    );
}

#[sqlx::test]
async fn poller_respects_team_default_project_capacity(pool: PgPool) {
    let mock = Arc::new(MockTaskFetcher::new());
    let provider_id = insert_provider(&pool).await;
    let project_id = insert_project_config(&pool, "kaneo-proj-capacity-default", provider_id).await;
    insert_active_run(&pool, project_id, "task-active").await;

    let service = build_service(mock, pool.clone());
    service.poll_once().await;

    let row = sqlx::query!(
        "SELECT COUNT(*) as count FROM work_runs WHERE project_config_id = $1",
        project_id,
    )
    .fetch_one(&pool)
    .await
    .expect("Should count work runs");

    assert_eq!(row.count.unwrap(), 1);
}

#[sqlx::test]
async fn poller_allows_project_capacity_override(pool: PgPool) {
    let mock = Arc::new(MockTaskFetcher::new());
    let provider_id = insert_provider(&pool).await;
    let project_id =
        insert_project_config(&pool, "kaneo-proj-capacity-override", provider_id).await;
    insert_active_run(&pool, project_id, "task-active").await;
    sqlx::query!(
        "UPDATE project_configs SET max_in_progress_tasks = 2 WHERE id = $1",
        project_id,
    )
    .execute(&pool)
    .await
    .expect("Should set project override");
    mock.set_tasks(
        "kaneo-proj-capacity-override",
        "to-do",
        vec![make_task("task-new", "Add one more task")],
    )
    .await;

    let service = build_service(mock, pool.clone());
    service.poll_once().await;

    let row = sqlx::query!(
        "SELECT COUNT(*) as count FROM work_runs WHERE project_config_id = $1",
        project_id,
    )
    .fetch_one(&pool)
    .await
    .expect("Should count work runs");

    assert_eq!(row.count.unwrap(), 2);
}

#[sqlx::test]
async fn poller_handles_unreachable_kaneo(pool: PgPool) {
    let mock = Arc::new(MockTaskFetcher::new());
    let provider_id = insert_provider(&pool).await;

    insert_project_config(&pool, "kaneo-good", provider_id).await;
    insert_project_config(&pool, "kaneo-bad", provider_id).await;

    mock.set_tasks(
        "kaneo-good",
        "to-do",
        vec![make_task("task-ok", "Working project task")],
    )
    .await;
    mock.set_error(
        "kaneo-bad",
        "to-do",
        IntegrationError::Other("connection refused".to_owned()),
    )
    .await;

    let service = build_service(mock, pool.clone());
    service.poll_once().await;

    let row = sqlx::query!(
        "SELECT COUNT(*) as count FROM work_runs WHERE external_task_ref = $1",
        "task-ok",
    )
    .fetch_one(&pool)
    .await
    .expect("Should query work_runs");

    assert_eq!(
        row.count.unwrap(),
        1,
        "Should insert task from working project despite failing one"
    );
}
