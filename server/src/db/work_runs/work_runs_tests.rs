use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::db::work_runs::queries::InsertWorkRunParams;
use crate::db::work_runs::WorkRunsRepository;
use crate::models::work_runs::errors::WorkRunsError;
use crate::models::work_runs::model::{WorkRunStatus, WorkRunType};
use crate::test_helpers::DEFAULT_TEAM_ID;

async fn insert_project_config(pool: &PgPool, external_project_id: &str) -> Uuid {
    let id = Uuid::new_v4();

    crate::test_helpers::ensure_default_team(pool).await;

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
        repo_full_names: Vec::new(),
        status,
        work_type: WorkRunType::Implementation,
        parent_work_run_id: None,
        review_target_pr_url: None,
        review_target_repo_full_name: None,
    }
}

async fn insert_work_run_at(
    pool: &PgPool,
    repo: &WorkRunsRepository,
    team_id: Uuid,
    project_config_id: Uuid,
    task_ref: &str,
    created_at: DateTime<Utc>,
    model_used: &str,
) -> Uuid {
    let mut params = insert_params(task_ref, project_config_id, WorkRunStatus::Completed);
    params.team_id = team_id;

    let id = repo
        .insert_work_run(pool, params)
        .await
        .expect("Should insert completed work run")
        .id;

    sqlx::query(
        "UPDATE work_runs
         SET created_at = $1,
             updated_at = $1,
             model_used = $2
         WHERE id = $3",
    )
    .bind(created_at)
    .bind(model_used)
    .bind(id)
    .execute(pool)
    .await
    .expect("Should set deterministic work run ordering fields");

    id
}

fn timestamp(value: &str) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(value)
        .expect("timestamp should parse")
        .with_timezone(&Utc)
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

#[sqlx::test]
async fn latest_related_runs_returns_latest_three_per_task_in_board_order(pool: PgPool) {
    let repo = WorkRunsRepository::new();
    let project_config_id = insert_project_config(&pool, "kaneo-proj-related-latest").await;

    insert_work_run_at(
        &pool,
        &repo,
        DEFAULT_TEAM_ID,
        project_config_id,
        "task-a",
        timestamp("2026-01-01T00:00:00Z"),
        "task-a-dropped-oldest",
    )
    .await;
    insert_work_run_at(
        &pool,
        &repo,
        DEFAULT_TEAM_ID,
        project_config_id,
        "task-a",
        timestamp("2026-01-01T00:01:00Z"),
        "task-a-third-newest",
    )
    .await;
    insert_work_run_at(
        &pool,
        &repo,
        DEFAULT_TEAM_ID,
        project_config_id,
        "task-a",
        timestamp("2026-01-01T00:02:00Z"),
        "task-a-second-newest",
    )
    .await;
    insert_work_run_at(
        &pool,
        &repo,
        DEFAULT_TEAM_ID,
        project_config_id,
        "task-a",
        timestamp("2026-01-01T00:03:00Z"),
        "task-a-newest",
    )
    .await;
    insert_work_run_at(
        &pool,
        &repo,
        DEFAULT_TEAM_ID,
        project_config_id,
        "task-b",
        timestamp("2026-01-01T00:00:30Z"),
        "task-b-oldest",
    )
    .await;
    insert_work_run_at(
        &pool,
        &repo,
        DEFAULT_TEAM_ID,
        project_config_id,
        "task-b",
        timestamp("2026-01-01T00:03:30Z"),
        "task-b-newest",
    )
    .await;

    let task_refs = vec!["task-b".to_owned(), "task-a".to_owned()];
    let rows = repo
        .list_latest_related_for_task_refs(&pool, DEFAULT_TEAM_ID, project_config_id, &task_refs, 3)
        .await
        .expect("Should list related work runs");

    let returned_refs = rows
        .iter()
        .map(|row| row.external_task_ref.as_str())
        .collect::<Vec<_>>();
    let returned_models = rows
        .iter()
        .map(|row| row.model_used.as_deref())
        .collect::<Vec<_>>();

    assert_eq!(
        returned_refs,
        vec!["task-b", "task-b", "task-a", "task-a", "task-a"]
    );
    assert_eq!(
        returned_models,
        vec![
            Some("task-b-newest"),
            Some("task-b-oldest"),
            Some("task-a-newest"),
            Some("task-a-second-newest"),
            Some("task-a-third-newest"),
        ]
    );
}

#[sqlx::test]
async fn latest_related_runs_excludes_same_task_ref_from_other_project_or_team(pool: PgPool) {
    let repo = WorkRunsRepository::new();
    let project_config_id = insert_project_config(&pool, "kaneo-proj-related-main").await;
    let other_project_config_id =
        insert_project_config(&pool, "kaneo-proj-related-other-project").await;
    let other_team_id = crate::test_helpers::insert_team(&pool, "Other related-runs team").await;

    insert_work_run_at(
        &pool,
        &repo,
        DEFAULT_TEAM_ID,
        project_config_id,
        "task-shared",
        timestamp("2026-01-01T00:01:00Z"),
        "included-main-run",
    )
    .await;
    insert_work_run_at(
        &pool,
        &repo,
        DEFAULT_TEAM_ID,
        other_project_config_id,
        "task-shared",
        timestamp("2026-01-01T00:02:00Z"),
        "excluded-other-project",
    )
    .await;
    insert_work_run_at(
        &pool,
        &repo,
        other_team_id,
        project_config_id,
        "task-shared",
        timestamp("2026-01-01T00:03:00Z"),
        "excluded-other-team",
    )
    .await;

    let task_refs = vec!["task-shared".to_owned()];
    let rows = repo
        .list_latest_related_for_task_refs(&pool, DEFAULT_TEAM_ID, project_config_id, &task_refs, 3)
        .await
        .expect("Should list scoped related work runs");

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].external_task_ref, "task-shared");
    assert_eq!(rows[0].model_used.as_deref(), Some("included-main-run"));
}

#[sqlx::test]
async fn latest_related_runs_returns_empty_for_empty_task_refs(pool: PgPool) {
    let repo = WorkRunsRepository::new();
    let project_config_id = insert_project_config(&pool, "kaneo-proj-related-empty").await;

    insert_work_run_at(
        &pool,
        &repo,
        DEFAULT_TEAM_ID,
        project_config_id,
        "task-present",
        timestamp("2026-01-01T00:01:00Z"),
        "should-not-be-returned",
    )
    .await;

    let rows = repo
        .list_latest_related_for_task_refs(&pool, DEFAULT_TEAM_ID, project_config_id, &[], 3)
        .await
        .expect("Should accept empty task ref input");

    assert!(rows.is_empty());
}
