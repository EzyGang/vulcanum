use std::sync::Arc;
use uuid::Uuid;

use crate::services::dispatcher::cancel_store::InMemoryCancelStore;
use crate::services::dispatcher::flag_store::InMemoryDispatchStore;
use crate::services::integration_providers::repository::IntegrationProvidersRepository;
use crate::services::project_configs::repository::ProjectConfigsRepository;
use crate::services::work_runs::errors::WorkRunsError;
use crate::services::work_runs::model::WorkRunStatus;
use crate::services::work_runs::repository::WorkRunsRepository;
use crate::services::work_runs::service::WorkRunsService;
use crate::services::workers::repository::WorkersRepository;
use crate::test_helpers;
use vulcanum_shared::api_types::SubmitResultRequest;

fn build_service(pool: sqlx::PgPool) -> WorkRunsService {
    WorkRunsService::new(
        WorkRunsRepository::new(),
        WorkersRepository::new(),
        ProjectConfigsRepository::new(),
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
    let project_id = test_helpers::insert_project_config(&pool, "kaneo-get-1").await;
    let wr_id = test_helpers::insert_pending_work_run(&pool, project_id, "task-get").await;

    let job = svc.get_job(wr_id).await.expect("Should get job");

    assert_eq!(job.external_task_ref, "task-get");
    assert_eq!(job.prompt_text, "Review the PR");
    assert_eq!(job.repo_url, "");
}

#[sqlx::test]
async fn get_job_returns_not_found(pool: sqlx::PgPool) {
    let svc = build_service(pool);

    let err = svc.get_job(Uuid::new_v4()).await.expect_err("Should fail");

    assert!(matches!(err, WorkRunsError::NotFound));
}
