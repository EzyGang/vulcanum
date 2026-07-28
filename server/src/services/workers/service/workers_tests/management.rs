use super::{
    svc, CreateWorkerParams, Duration, RenameWorkerRequest, Utc, WorkerStatus, WorkersError,
    DEFAULT_MAX_CONCURRENT_JOBS, DEFAULT_TEAM_ID,
};

#[sqlx::test]
async fn list_all_returns_workers(pool: sqlx::PgPool) {
    let svc = svc(pool.clone()).await;
    let expiry = Utc::now() + Duration::days(30);
    let capabilities = serde_json::json!({});

    svc.repo
        .create(
            &pool,
            CreateWorkerParams {
                team_id: DEFAULT_TEAM_ID,
                name: "l1",
                refresh_token_hash: "h1",
                refresh_expires_at: expiry,
                capabilities: &capabilities,
                max_concurrent_jobs: DEFAULT_MAX_CONCURRENT_JOBS,
            },
        )
        .await
        .unwrap();
    svc.repo
        .create(
            &pool,
            CreateWorkerParams {
                team_id: DEFAULT_TEAM_ID,
                name: "l2",
                refresh_token_hash: "h2",
                refresh_expires_at: expiry,
                capabilities: &capabilities,
                max_concurrent_jobs: DEFAULT_MAX_CONCURRENT_JOBS,
            },
        )
        .await
        .unwrap();

    let workers = svc.list_all(DEFAULT_TEAM_ID).await.expect("Should list");
    assert_eq!(workers.len(), 2);
}

#[sqlx::test]
async fn rename_worker_preserves_identity_and_operational_state(pool: sqlx::PgPool) {
    let svc = svc(pool.clone()).await;
    let expiry = Utc::now() + Duration::days(30);
    let capabilities = serde_json::json!({"isolation_backends": ["docker"]});
    let worker = svc
        .repo
        .create(
            &pool,
            CreateWorkerParams {
                team_id: DEFAULT_TEAM_ID,
                name: "registration-name",
                refresh_token_hash: "refresh-hash",
                refresh_expires_at: expiry,
                capabilities: &capabilities,
                max_concurrent_jobs: 2,
            },
        )
        .await
        .expect("worker should be created");
    sqlx::query!(
        "UPDATE workers SET status = 'busy'::worker_status, active_jobs = 1 WHERE id = $1",
        worker.id,
    )
    .execute(&pool)
    .await
    .expect("worker should be updated");
    let project_config_id =
        crate::test_helpers::project_configs::insert_project_config(&pool, "rename-project").await;
    let work_run_id = crate::test_helpers::work_runs::insert_running_work_run(
        &pool,
        project_config_id,
        "RENAME-1",
        worker.id,
    )
    .await;
    let before = svc
        .repo
        .find_by_id(&pool, worker.id)
        .await
        .expect("worker should exist before rename");

    let renamed = svc
        .rename_worker(
            worker.id,
            DEFAULT_TEAM_ID,
            RenameWorkerRequest {
                name: "meaningful-name".to_owned(),
            },
        )
        .await
        .expect("worker should be renamed");
    let after = svc
        .repo
        .find_by_id(&pool, worker.id)
        .await
        .expect("worker should exist after rename");
    let run = sqlx::query!("SELECT worker_id FROM work_runs WHERE id = $1", work_run_id)
        .fetch_one(&pool)
        .await
        .expect("work run should exist");

    assert_eq!(renamed.id, worker.id);
    assert_eq!(renamed.name, "meaningful-name");
    assert_eq!(after.id, before.id);
    assert_eq!(after.team_id, before.team_id);
    assert_eq!(after.refresh_token_hash, before.refresh_token_hash);
    assert_eq!(after.refresh_expires_at, before.refresh_expires_at);
    assert!(matches!(before.status, WorkerStatus::Busy));
    assert!(matches!(after.status, WorkerStatus::Busy));
    assert_eq!(after.max_concurrent_jobs, before.max_concurrent_jobs);
    assert_eq!(after.capabilities, before.capabilities);
    assert_eq!(after.active_jobs, before.active_jobs);
    assert_eq!(run.worker_id, Some(worker.id));
}

#[sqlx::test]
async fn rename_worker_rejects_cross_team_worker(pool: sqlx::PgPool) {
    let svc = svc(pool.clone()).await;
    let team_b = crate::test_helpers::teams::insert_team(&pool, "rename-team-b").await;
    let worker_id = crate::test_helpers::workers::insert_worker(&pool, "team-a-worker").await;

    let err = svc
        .rename_worker(
            worker_id,
            team_b,
            RenameWorkerRequest {
                name: "other-team-name".to_owned(),
            },
        )
        .await
        .expect_err("team B must not rename team A worker");

    assert!(matches!(err, WorkersError::WorkerNotFound));
}

#[sqlx::test]
async fn delete_worker_rejects_cross_team_worker(pool: sqlx::PgPool) {
    let svc = svc(pool.clone()).await;
    let team_b = crate::test_helpers::teams::insert_team(&pool, "workers-team-b").await;
    let worker_id = crate::test_helpers::workers::insert_worker(&pool, "team-a-worker").await;

    let err = svc
        .delete_worker(worker_id, team_b)
        .await
        .expect_err("team B must not delete team A worker");

    assert!(matches!(err, WorkersError::WorkerNotFound));
}

#[sqlx::test]
async fn delete_self_resets_assigned_runs_and_deletes_worker(pool: sqlx::PgPool) {
    let svc = svc(pool.clone()).await;
    let project_config_id =
        crate::test_helpers::project_configs::insert_project_config(&pool, "ext-project").await;
    let worker_id = crate::test_helpers::workers::insert_worker(&pool, "self-delete-worker").await;
    let work_run_id = crate::test_helpers::work_runs::insert_running_work_run(
        &pool,
        project_config_id,
        "TASK-123",
        worker_id,
    )
    .await;

    svc.delete_self(worker_id)
        .await
        .expect("Should delete self");

    let worker = sqlx::query!(
        "SELECT COUNT(*) AS count FROM workers WHERE id = $1",
        worker_id
    )
    .fetch_one(&pool)
    .await
    .expect("Should query worker");
    assert_eq!(worker.count.unwrap(), 0);

    let run = sqlx::query!(
        r#"SELECT worker_id, status::text as "status!: String" FROM work_runs WHERE id = $1"#,
        work_run_id
    )
    .fetch_one(&pool)
    .await
    .expect("Should query run");
    assert!(run.worker_id.is_none());
    assert_eq!(run.status, "pending");
}
