use super::{build_service, test_helpers, WorkRunStatus, WorkRunsError, WorkerStatus};

#[sqlx::test]
async fn poll_returns_none_when_no_dispatch(pool: sqlx::PgPool) {
    let svc = build_service(pool.clone());
    let worker_id = test_helpers::workers::insert_worker(&pool, "idle-worker").await;

    let result = svc.poll(worker_id).await.expect("Should succeed");

    assert!(
        result.is_none(),
        "Should return None when nothing dispatched"
    );
}

#[sqlx::test]
async fn poll_recovers_disconnected_worker_to_idle(pool: sqlx::PgPool) {
    let svc = build_service(pool.clone());
    let worker_id = test_helpers::workers::insert_worker(&pool, "disconnected-worker").await;

    sqlx::query!(
        "UPDATE workers SET status = 'disconnected'::worker_status WHERE id = $1",
        worker_id,
    )
    .execute(&pool)
    .await
    .expect("Should mark worker disconnected");

    let result = svc.poll(worker_id).await.expect("Should succeed");

    assert!(
        result.is_none(),
        "Should return None when nothing dispatched"
    );

    let row = sqlx::query!(
        r#"SELECT status as "status: WorkerStatus" FROM workers WHERE id = $1"#,
        worker_id,
    )
    .fetch_one(&pool)
    .await
    .expect("Should query worker");

    assert!(matches!(row.status, WorkerStatus::Idle));
}

#[sqlx::test]
async fn poll_does_not_recover_unhealthy_worker(pool: sqlx::PgPool) {
    let svc = build_service(pool.clone());
    let worker_id = test_helpers::workers::insert_worker(&pool, "unhealthy-worker").await;

    sqlx::query!(
        "UPDATE workers SET status = 'unhealthy'::worker_status WHERE id = $1",
        worker_id,
    )
    .execute(&pool)
    .await
    .expect("Should mark worker unhealthy");

    let result = svc.poll(worker_id).await.expect("Should succeed");

    assert!(
        result.is_none(),
        "Should return None when nothing dispatched"
    );

    let row = sqlx::query!(
        r#"SELECT status as "status: WorkerStatus" FROM workers WHERE id = $1"#,
        worker_id,
    )
    .fetch_one(&pool)
    .await
    .expect("Should query worker");

    assert!(matches!(row.status, WorkerStatus::Unhealthy));
}

#[sqlx::test]
async fn poll_returns_job_id_when_dispatched(pool: sqlx::PgPool) {
    let svc = build_service(pool.clone());
    let worker_id = test_helpers::workers::insert_worker(&pool, "busy-worker").await;
    let project_id =
        test_helpers::project_configs::insert_project_config(&pool, "kaneo-poll-1").await;
    let wr_id =
        test_helpers::work_runs::insert_pending_work_run(&pool, project_id, "task-poll").await;

    svc.dispatch_store()
        .set_dispatched(worker_id, wr_id)
        .await
        .expect("Should set dispatched");

    let result = svc.poll(worker_id).await.expect("Should succeed");

    assert_eq!(result, Some(wr_id));
}

#[sqlx::test]
async fn list_all_rejects_cross_team_runs(pool: sqlx::PgPool) {
    let svc = build_service(pool.clone());
    let team_b = test_helpers::teams::insert_team(&pool, "runs-team-b").await;
    let project_id =
        test_helpers::project_configs::insert_project_config(&pool, "kaneo-cross-team-list").await;
    test_helpers::work_runs::insert_pending_work_run(&pool, project_id, "task-cross-team-list")
        .await;

    let runs = svc
        .list_all(team_b, None, Some(20), Some(0))
        .await
        .expect("Should list team B runs");

    assert!(runs.is_empty());
}

#[sqlx::test]
async fn list_all_returns_null_task_metadata_when_unset(pool: sqlx::PgPool) {
    let svc = build_service(pool.clone());
    let project_id =
        test_helpers::project_configs::insert_project_config(&pool, "kaneo-list-slug").await;
    test_helpers::work_runs::insert_pending_work_run(&pool, project_id, "task-list-slug").await;

    let runs = svc
        .list_all(test_helpers::DEFAULT_TEAM_ID, None, Some(20), Some(0))
        .await
        .expect("Should list team runs");

    assert_eq!(runs[0].external_task_ref, "task-list-slug");
    assert_eq!(runs[0].task_slug, None);
    assert_eq!(runs[0].task_title, None);
}

#[sqlx::test]
async fn poll_consumes_dispatch_flag(pool: sqlx::PgPool) {
    let svc = build_service(pool.clone());
    let worker_id = test_helpers::workers::insert_worker(&pool, "consume-worker").await;
    let project_id =
        test_helpers::project_configs::insert_project_config(&pool, "kaneo-poll-2").await;
    let wr_id =
        test_helpers::work_runs::insert_pending_work_run(&pool, project_id, "task-consume").await;

    svc.dispatch_store()
        .set_dispatched(worker_id, wr_id)
        .await
        .expect("Should set dispatched");

    let first = svc.poll(worker_id).await.expect("Should succeed");
    assert!(
        first.is_some(),
        "First poll should return the dispatched job"
    );

    let second = svc.poll(worker_id).await.expect("Should succeed");
    assert!(
        second.is_none(),
        "Second poll should return None — flag consumed"
    );
}

#[sqlx::test]
async fn ack_transitions_dispatched_to_running(pool: sqlx::PgPool) {
    let svc = build_service(pool.clone());
    let worker_id = test_helpers::workers::insert_worker(&pool, "claimer").await;
    let project_id =
        test_helpers::project_configs::insert_project_config(&pool, "kaneo-ack-1").await;
    let wr_id =
        test_helpers::work_runs::insert_pending_work_run(&pool, project_id, "task-ack").await;

    let dispatch_repo = crate::db::dispatcher::DispatchRepository;
    dispatch_repo
        .dispatch_to_worker(&pool, wr_id, worker_id)
        .await
        .expect("Should dispatch");

    let job = svc.ack_job(wr_id, worker_id).await.expect("Should ack");

    assert_eq!(job.id, wr_id);
    assert_eq!(job.worker_id, Some(worker_id));
    assert!(matches!(job.status, WorkRunStatus::Running));
    assert_eq!(job.external_task_ref, "task-ack");
}

#[sqlx::test]
async fn ack_fails_when_already_claimed(pool: sqlx::PgPool) {
    let svc = build_service(pool.clone());
    let worker_a = test_helpers::workers::insert_worker(&pool, "worker-a").await;
    let worker_b = test_helpers::workers::insert_worker(&pool, "worker-b").await;
    let project_id =
        test_helpers::project_configs::insert_project_config(&pool, "kaneo-ack-2").await;
    let wr_id =
        test_helpers::work_runs::insert_pending_work_run(&pool, project_id, "task-race").await;

    let dispatch_repo = crate::db::dispatcher::DispatchRepository;
    dispatch_repo
        .dispatch_to_worker(&pool, wr_id, worker_a)
        .await
        .expect("Should dispatch to A");

    svc.ack_job(wr_id, worker_a)
        .await
        .expect("First ack should succeed");

    let err = svc
        .ack_job(wr_id, worker_b)
        .await
        .expect_err("Second ack should fail");

    assert!(matches!(err, WorkRunsError::AlreadyClaimed));
}
