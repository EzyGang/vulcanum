use std::sync::Arc;
use uuid::Uuid;

use crate::services::dispatcher::cancel_store::InMemoryCancelStore;
use crate::services::dispatcher::dispatch_store::InMemoryDispatchStore;
use crate::services::github_app::repository::GithubAppRepository;
use crate::services::github_app::service::GithubAppManager;
use crate::services::project_configs::repository::ProjectConfigsRepository;
use crate::services::provider_configs::repository::IntegrationProvidersRepository;
use crate::services::work_runs::errors::WorkRunsError;
use crate::services::work_runs::model::WorkRunStatus;
use crate::services::work_runs::repository::WorkRunsRepository;
use crate::services::work_runs::service::WorkRunsService;
use crate::services::workers::repository::WorkersRepository;
use crate::test_helpers;
use vulcanum_shared::api_types::SubmitResultRequest;

fn build_github_manager(pool: sqlx::PgPool) -> GithubAppManager {
    let cfg = crate::config::AppConfig {
        db_url: String::new(),
        max_conns: 1,
        poll_period_secs: 30,
        jwt_secret: String::new(),
        stale_worker_threshold_secs: 120,
        unhealthy_threshold: 3,
        stalled_running_threshold_secs: 1800,
        instance_password: String::new(),
        is_single_user: true,
        redis_url: "redis://127.0.0.1:6379".to_owned(),
        github_app_id: None,
        github_app_private_key: None,
        github_app_slug: None,
        github_oauth_client_id: None,
        github_oauth_client_secret: None,
        github_oauth_redirect_url: None,
    };
    GithubAppManager::new(
        GithubAppRepository::new(),
        pool,
        "redis://127.0.0.1:6379",
        &cfg,
    )
    .expect("build github manager for tests")
}

fn build_service(pool: sqlx::PgPool) -> WorkRunsService {
    WorkRunsService::new(
        WorkRunsRepository::new(),
        WorkersRepository::new(),
        ProjectConfigsRepository::new(),
        build_github_manager(pool.clone()),
        pool,
        Arc::new(InMemoryDispatchStore::default()),
        IntegrationProvidersRepository::new(),
        Arc::new(InMemoryCancelStore::new()),
        3,
    )
}

#[sqlx::test]
async fn poll_returns_none_when_no_dispatch(pool: sqlx::PgPool) {
    let svc = build_service(pool.clone());
    let worker_id = test_helpers::insert_worker(&pool, "idle-worker").await;

    let result = svc.poll(worker_id).await.expect("Should succeed");

    assert!(
        result.is_none(),
        "Should return None when nothing dispatched"
    );
}

#[sqlx::test]
async fn poll_returns_job_id_when_dispatched(pool: sqlx::PgPool) {
    let svc = build_service(pool.clone());
    let worker_id = test_helpers::insert_worker(&pool, "busy-worker").await;
    let project_id = test_helpers::insert_project_config(&pool, "kaneo-poll-1").await;
    let wr_id = test_helpers::insert_pending_work_run(&pool, project_id, "task-poll").await;

    svc.dispatch_store
        .set_dispatched(worker_id, wr_id)
        .await
        .expect("Should set dispatched");

    let result = svc.poll(worker_id).await.expect("Should succeed");

    assert_eq!(result, Some(wr_id));
}

#[sqlx::test]
async fn list_all_rejects_cross_team_runs(pool: sqlx::PgPool) {
    let svc = build_service(pool.clone());
    let team_b = test_helpers::insert_team(&pool, "runs-team-b").await;
    let project_id = test_helpers::insert_project_config(&pool, "kaneo-cross-team-list").await;
    test_helpers::insert_pending_work_run(&pool, project_id, "task-cross-team-list").await;

    let runs = svc
        .list_all(team_b, None, 20, 0)
        .await
        .expect("Should list team B runs");

    assert!(runs.is_empty());
}

#[sqlx::test]
async fn poll_consumes_dispatch_flag(pool: sqlx::PgPool) {
    let svc = build_service(pool.clone());
    let worker_id = test_helpers::insert_worker(&pool, "consume-worker").await;
    let project_id = test_helpers::insert_project_config(&pool, "kaneo-poll-2").await;
    let wr_id = test_helpers::insert_pending_work_run(&pool, project_id, "task-consume").await;

    svc.dispatch_store
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
    let worker_id = test_helpers::insert_worker(&pool, "claimer").await;
    let project_id = test_helpers::insert_project_config(&pool, "kaneo-ack-1").await;
    let wr_id = test_helpers::insert_pending_work_run(&pool, project_id, "task-ack").await;

    let dispatch_repo = crate::services::dispatcher::repository::DispatchRepository;
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
    let worker_a = test_helpers::insert_worker(&pool, "worker-a").await;
    let worker_b = test_helpers::insert_worker(&pool, "worker-b").await;
    let project_id = test_helpers::insert_project_config(&pool, "kaneo-ack-2").await;
    let wr_id = test_helpers::insert_pending_work_run(&pool, project_id, "task-race").await;

    let dispatch_repo = crate::services::dispatcher::repository::DispatchRepository;
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

#[sqlx::test]
async fn submit_result_marks_completed(pool: sqlx::PgPool) {
    let svc = build_service(pool.clone());
    let worker_id = test_helpers::insert_worker(&pool, "result-worker").await;
    let project_id = test_helpers::insert_project_config(&pool, "kaneo-result-1").await;
    let wr_id = test_helpers::insert_pending_work_run(&pool, project_id, "task-result").await;

    let dispatch_repo = crate::services::dispatcher::repository::DispatchRepository;
    dispatch_repo
        .dispatch_to_worker(&pool, wr_id, worker_id)
        .await
        .expect("Should dispatch");
    svc.ack_job(wr_id, worker_id).await.expect("Should ack");

    let params = SubmitResultRequest {
        pr_url: "https://github.com/example/pr/1".to_owned(),
        exit_code: 0,
        tokens_used: 500,
        duration_ms: 30000,
        input_tokens: 0,
        output_tokens: 0,
        cache_read_tokens: 0,
        cache_write_tokens: 0,
        model_used: None,
        finish_status: None,
        finish_summary: None,
        finish_blocked_reason: None,
        finish_next_column: None,
    };
    let job = svc
        .submit_result(wr_id, worker_id, params)
        .await
        .expect("Should succeed");

    assert!(matches!(job.status, WorkRunStatus::Completed));
    assert_eq!(
        job.result_pr_url.as_deref(),
        Some("https://github.com/example/pr/1")
    );
    assert_eq!(job.result_exit_code, Some(0));
    assert_eq!(job.tokens_used, Some(500));
    assert_eq!(job.duration_ms, Some(30000));
}

#[sqlx::test]
async fn submit_result_marks_failed_on_nonzero_exit(pool: sqlx::PgPool) {
    let svc = build_service(pool.clone());
    let worker_id = test_helpers::insert_worker(&pool, "fail-worker").await;
    let project_id = test_helpers::insert_project_config(&pool, "kaneo-fail-1").await;
    let wr_id = test_helpers::insert_pending_work_run(&pool, project_id, "task-fail").await;

    let dispatch_repo = crate::services::dispatcher::repository::DispatchRepository;
    dispatch_repo
        .dispatch_to_worker(&pool, wr_id, worker_id)
        .await
        .expect("Should dispatch");
    svc.ack_job(wr_id, worker_id).await.expect("Should ack");

    let params = SubmitResultRequest {
        pr_url: String::new(),
        exit_code: 1,
        tokens_used: 0,
        duration_ms: 5000,
        input_tokens: 0,
        output_tokens: 0,
        cache_read_tokens: 0,
        cache_write_tokens: 0,
        model_used: None,
        finish_status: None,
        finish_summary: None,
        finish_blocked_reason: None,
        finish_next_column: None,
    };
    let job = svc
        .submit_result(wr_id, worker_id, params)
        .await
        .expect("Should succeed");

    assert!(matches!(job.status, WorkRunStatus::Failed));
}

#[sqlx::test]
async fn submit_result_fails_if_not_running(pool: sqlx::PgPool) {
    let svc = build_service(pool.clone());
    let worker_id = test_helpers::insert_worker(&pool, "early-result").await;
    let project_id = test_helpers::insert_project_config(&pool, "kaneo-early-1").await;
    let wr_id = test_helpers::insert_pending_work_run(&pool, project_id, "task-early").await;

    let params = SubmitResultRequest {
        pr_url: String::new(),
        exit_code: 0,
        tokens_used: 0,
        duration_ms: 0,
        input_tokens: 0,
        output_tokens: 0,
        cache_read_tokens: 0,
        cache_write_tokens: 0,
        model_used: None,
        finish_status: None,
        finish_summary: None,
        finish_blocked_reason: None,
        finish_next_column: None,
    };
    let err = svc
        .submit_result(wr_id, worker_id, params)
        .await
        .expect_err("Should fail on pending job");

    assert!(matches!(err, WorkRunsError::InvalidStatusTransition));
}

#[sqlx::test]
async fn submit_result_fails_if_not_owner(pool: sqlx::PgPool) {
    let svc = build_service(pool.clone());
    let worker_a = test_helpers::insert_worker(&pool, "owner-a").await;
    let worker_b = test_helpers::insert_worker(&pool, "intruder-b").await;
    let project_id = test_helpers::insert_project_config(&pool, "kaneo-owner-1").await;
    let wr_id = test_helpers::insert_pending_work_run(&pool, project_id, "task-owner").await;

    let dispatch_repo = crate::services::dispatcher::repository::DispatchRepository;
    dispatch_repo
        .dispatch_to_worker(&pool, wr_id, worker_a)
        .await
        .expect("Should dispatch");
    svc.ack_job(wr_id, worker_a)
        .await
        .expect("Worker A should ack");

    let params = SubmitResultRequest {
        pr_url: String::new(),
        exit_code: 0,
        tokens_used: 0,
        duration_ms: 0,
        input_tokens: 0,
        output_tokens: 0,
        cache_read_tokens: 0,
        cache_write_tokens: 0,
        model_used: None,
        finish_status: None,
        finish_summary: None,
        finish_blocked_reason: None,
        finish_next_column: None,
    };
    let err = svc
        .submit_result(wr_id, worker_b, params)
        .await
        .expect_err("Worker B should not submit result");

    assert!(matches!(err, WorkRunsError::NotOwned));
}

#[sqlx::test]
async fn get_job_returns_full_details(pool: sqlx::PgPool) {
    let svc = build_service(pool.clone());
    let worker_id = test_helpers::insert_worker(&pool, "get-job-worker").await;
    let project_id = test_helpers::insert_project_config(&pool, "kaneo-get-1").await;
    let wr_id = test_helpers::insert_pending_work_run(&pool, project_id, "task-get").await;
    sqlx::query!(
        "UPDATE work_runs SET worker_id = $1 WHERE id = $2",
        worker_id,
        wr_id
    )
    .execute(&pool)
    .await
    .expect("Should assign worker");

    let job = svc.get_job(wr_id, worker_id).await.expect("Should get job");

    assert_eq!(job.external_task_ref, "task-get");
    assert_eq!(job.prompt_text, "Review the PR");
    assert_eq!(job.repo_url, "");
}

#[sqlx::test]
async fn get_job_returns_not_found(pool: sqlx::PgPool) {
    let svc = build_service(pool.clone());
    let worker_id = test_helpers::insert_worker(&pool, "missing-job-worker").await;

    let err = svc
        .get_job(Uuid::new_v4(), worker_id)
        .await
        .expect_err("Should fail");

    assert!(matches!(err, WorkRunsError::NotFound));
}

#[sqlx::test]
async fn get_job_with_repo_url_and_no_installation_fails(pool: sqlx::PgPool) {
    let svc = build_service(pool.clone());
    let worker_id = test_helpers::insert_worker(&pool, "github-job-worker").await;
    let project_id = test_helpers::insert_project_config(&pool, "kaneo-get-2").await;

    sqlx::query!(
        "UPDATE project_configs SET repo_url = $1 WHERE id = $2",
        "https://github.com/org/repo",
        project_id
    )
    .execute(&pool)
    .await
    .expect("Should update repo_url");

    let wr_id = test_helpers::insert_pending_work_run(&pool, project_id, "task-get-2").await;
    sqlx::query!(
        "UPDATE work_runs SET worker_id = $1 WHERE id = $2",
        worker_id,
        wr_id
    )
    .execute(&pool)
    .await
    .expect("Should assign worker");

    let err = svc
        .get_job(wr_id, worker_id)
        .await
        .expect_err("Should fail without GitHub installation");

    assert!(
        matches!(err, WorkRunsError::GithubApp(_)),
        "Expected GithubApp error, got {err:?}"
    );
}
