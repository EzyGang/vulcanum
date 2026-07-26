use super::*;

#[sqlx::test]
async fn submit_result_marks_completed(pool: sqlx::PgPool) {
    let svc = build_service(pool.clone());
    let worker_id = test_helpers::workers::insert_worker(&pool, "result-worker").await;
    let project_id =
        test_helpers::project_configs::insert_project_config(&pool, "kaneo-result-1").await;
    let wr_id =
        test_helpers::work_runs::insert_pending_work_run(&pool, project_id, "task-result").await;
    sqlx::query!(
        "INSERT INTO work_run_repos (work_run_id, repo_full_name, repo_url, position) VALUES ($1, $2, $3, $4)",
        wr_id,
        "example/repo",
        "https://github.com/example/repo",
        0_i32,
    )
    .execute(&pool)
    .await
    .expect("Should insert work run repo");

    let dispatch_repo = crate::db::dispatcher::DispatchRepository;
    dispatch_repo
        .dispatch_to_worker(&pool, wr_id, worker_id)
        .await
        .expect("Should dispatch");
    dispatch_repo
        .increment_worker_jobs(&pool, worker_id)
        .await
        .expect("Should reserve worker capacity");
    svc.ack_job(wr_id, worker_id).await.expect("Should ack");

    let params = SubmitResultRequest {
        pr_urls: vec!["https://github.com/example/repo/pull/1".to_owned()],
        exit_code: 0,
        tokens_used: 500,
        duration_ms: 30000,
        input_tokens: 0,
        output_tokens: 0,
        cache_read_tokens: 0,
        cache_write_tokens: 0,
        model_used: None,
        finish_status: None,
        result_summary: None,
        review_url: None,
        review_body: None,
        review_already_exists: false,
    };
    let job = svc
        .submit_result(wr_id, worker_id, params)
        .await
        .expect("Should succeed");

    assert!(matches!(job.status, WorkRunStatus::Completed));
    assert_eq!(
        job.result_pr_url.as_deref(),
        Some("https://github.com/example/repo/pull/1")
    );
    assert_eq!(job.result_exit_code, Some(0));
    assert_eq!(job.tokens_used, Some(500));
    assert_eq!(job.duration_ms, Some(30000));
    let task_pr_count = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM task_prs WHERE source_work_run_id = $1",
        wr_id,
    )
    .fetch_one(&pool)
    .await
    .expect("Should count persisted task PRs")
    .unwrap_or_default();
    assert_eq!(task_pr_count, 1);
}

#[sqlx::test]
async fn submit_result_marks_failed_on_nonzero_exit(pool: sqlx::PgPool) {
    let svc = build_service(pool.clone());
    let worker_id = test_helpers::workers::insert_worker(&pool, "fail-worker").await;
    let project_id =
        test_helpers::project_configs::insert_project_config(&pool, "kaneo-fail-1").await;
    let wr_id =
        test_helpers::work_runs::insert_pending_work_run(&pool, project_id, "task-fail").await;

    let dispatch_repo = crate::db::dispatcher::DispatchRepository;
    dispatch_repo
        .dispatch_to_worker(&pool, wr_id, worker_id)
        .await
        .expect("Should dispatch");
    dispatch_repo
        .increment_worker_jobs(&pool, worker_id)
        .await
        .expect("Should reserve worker capacity");
    svc.ack_job(wr_id, worker_id).await.expect("Should ack");

    let params = SubmitResultRequest {
        pr_urls: Vec::new(),
        exit_code: 1,
        tokens_used: 0,
        duration_ms: 5000,
        input_tokens: 0,
        output_tokens: 0,
        cache_read_tokens: 0,
        cache_write_tokens: 0,
        model_used: None,
        finish_status: None,
        result_summary: None,
        review_url: None,
        review_body: None,
        review_already_exists: false,
    };
    let job = svc
        .submit_result(wr_id, worker_id, params)
        .await
        .expect("Should succeed");

    assert!(matches!(job.status, WorkRunStatus::Failed));
}

#[sqlx::test]
async fn submit_result_completes_when_worker_active_jobs_already_zero(pool: sqlx::PgPool) {
    let svc = build_service(pool.clone());
    let worker_id = test_helpers::workers::insert_worker(&pool, "stale-submit-worker").await;
    let project_id =
        test_helpers::project_configs::insert_project_config(&pool, "kaneo-stale-submit").await;
    let wr_id = test_helpers::work_runs::insert_running_work_run(
        &pool,
        project_id,
        "task-stale-submit",
        worker_id,
    )
    .await;

    zero_worker_active_jobs(&pool, worker_id).await;

    let job = svc
        .submit_result(wr_id, worker_id, completed_result_request())
        .await
        .expect("Should complete even when active_jobs is already zero");

    assert!(matches!(job.status, WorkRunStatus::Completed));
}

#[sqlx::test]
async fn fail_run_finishes_when_worker_active_jobs_already_zero(pool: sqlx::PgPool) {
    let svc = build_service(pool.clone());
    let worker_id = test_helpers::workers::insert_worker(&pool, "stale-fail-worker").await;
    let project_id =
        test_helpers::project_configs::insert_project_config(&pool, "kaneo-stale-fail").await;
    let wr_id =
        test_helpers::work_runs::insert_pending_work_run(&pool, project_id, "task-stale-fail")
            .await;
    let dispatch_repo = crate::db::dispatcher::DispatchRepository;
    dispatch_repo
        .dispatch_to_worker(&pool, wr_id, worker_id)
        .await
        .expect("Should dispatch");
    dispatch_repo
        .increment_worker_jobs(&pool, worker_id)
        .await
        .expect("Should reserve worker capacity");

    zero_worker_active_jobs(&pool, worker_id).await;

    let job = svc
        .fail_run(wr_id, test_helpers::DEFAULT_TEAM_ID)
        .await
        .expect("Should fail run even when active_jobs is already zero");

    assert!(matches!(job.status, WorkRunStatus::Failed));
}

#[sqlx::test]
async fn delete_run_removes_dispatched_when_worker_active_jobs_already_zero(pool: sqlx::PgPool) {
    let svc = build_service(pool.clone());
    let worker_id = test_helpers::workers::insert_worker(&pool, "stale-delete-worker").await;
    let project_id =
        test_helpers::project_configs::insert_project_config(&pool, "kaneo-stale-delete").await;
    let wr_id =
        test_helpers::work_runs::insert_pending_work_run(&pool, project_id, "task-stale-delete")
            .await;
    let dispatch_repo = crate::db::dispatcher::DispatchRepository;
    dispatch_repo
        .dispatch_to_worker(&pool, wr_id, worker_id)
        .await
        .expect("Should dispatch");
    dispatch_repo
        .increment_worker_jobs(&pool, worker_id)
        .await
        .expect("Should reserve worker capacity");

    zero_worker_active_jobs(&pool, worker_id).await;

    svc.delete_run(wr_id, test_helpers::DEFAULT_TEAM_ID)
        .await
        .expect("Should delete run even when active_jobs is already zero");

    let err = WorkRunsRepository::new()
        .find_by_id(&pool, wr_id)
        .await
        .expect_err("Deleted work run should not load");

    assert!(matches!(err, WorkRunsError::NotFound));
}

#[sqlx::test]
async fn submit_result_accumulates_task_and_project_usage_once_per_result(pool: sqlx::PgPool) {
    fn result_request(
        exit_code: i32,
        tokens_used: i64,
        input_tokens: i64,
        output_tokens: i64,
        cache_read_tokens: i64,
        cache_write_tokens: i64,
    ) -> SubmitResultRequest {
        SubmitResultRequest {
            pr_urls: Vec::new(),
            exit_code,
            tokens_used,
            duration_ms: 1000,
            input_tokens,
            output_tokens,
            cache_read_tokens,
            cache_write_tokens,
            model_used: None,
            finish_status: None,
            result_summary: None,
            review_url: None,
            review_body: None,
            review_already_exists: false,
        }
    }

    let svc = build_service(pool.clone());
    let worker_id = test_helpers::workers::insert_worker(&pool, "usage-worker").await;
    let project_id =
        test_helpers::project_configs::insert_project_config(&pool, "kaneo-usage-1").await;
    let first_wr_id = test_helpers::work_runs::insert_running_work_run(
        &pool,
        project_id,
        "task-usage",
        worker_id,
    )
    .await;

    let first_job = svc
        .submit_result(first_wr_id, worker_id, result_request(0, 100, 40, 60, 7, 3))
        .await
        .expect("Should submit first result");
    assert!(matches!(first_job.status, WorkRunStatus::Completed));
    let retried_job = svc
        .submit_result(first_wr_id, worker_id, result_request(0, 100, 40, 60, 7, 3))
        .await
        .expect("Should accept an idempotent result retry");
    assert!(matches!(retried_job.status, WorkRunStatus::Completed));

    let second_wr_id = test_helpers::work_runs::insert_running_work_run(
        &pool,
        project_id,
        "task-usage",
        worker_id,
    )
    .await;
    let second_job = svc
        .submit_result(second_wr_id, worker_id, result_request(1, 25, 10, 15, 2, 1))
        .await
        .expect("Should submit second result");
    assert!(matches!(second_job.status, WorkRunStatus::Failed));

    let task_refs = vec!["task-usage".to_owned()];
    let rows = TaskAugmentationsRepository::new()
        .list_for_task_refs(&pool, test_helpers::DEFAULT_TEAM_ID, project_id, &task_refs)
        .await
        .expect("Should load task usage augmentation");

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].external_task_ref, "task-usage");
    assert_eq!(rows[0].tokens_used, 125);
    assert_eq!(rows[0].input_tokens, 50);
    assert_eq!(rows[0].output_tokens, 75);
    assert_eq!(rows[0].cache_read_tokens, 9);
    assert_eq!(rows[0].cache_write_tokens, 4);
    assert_eq!(rows[0].finished_runs_count, 2);

    let project_usage = ProjectUsageRepository::new()
        .summary(&pool, project_id)
        .await
        .expect("Should load project usage");
    assert_eq!(project_usage.total.tokens_used, 125);
    assert_eq!(project_usage.total.input_tokens, 50);
    assert_eq!(project_usage.total.output_tokens, 75);
    assert_eq!(project_usage.total.cache_read_tokens, 9);
    assert_eq!(project_usage.total.cache_write_tokens, 4);
    assert_eq!(project_usage.total.finished_runs_count, 2);
    assert_eq!(project_usage.this_week.tokens_used, 125);
    assert_eq!(project_usage.this_week.finished_runs_count, 2);
}
