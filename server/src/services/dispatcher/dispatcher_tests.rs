use std::sync::Arc;

use sqlx::PgPool;

use crate::services::dispatcher::dispatch_store::{DispatchStore, InMemoryDispatchStore};
use crate::services::dispatcher::repository::DispatchRepository;
use crate::services::dispatcher::service::DispatcherService;
use crate::services::work_runs::model::WorkRunStatus;
use crate::services::workers::model::WorkerStatus;
use crate::test_helpers;

fn build_service(pool: PgPool) -> DispatcherService {
    DispatcherService::new(
        DispatchRepository::new(),
        crate::services::workers::repository::WorkersRepository::new(),
        crate::services::work_runs::repository::WorkRunsRepository::new(),
        pool,
        Arc::new(InMemoryDispatchStore::default()),
        DEFAULT_STALE_THRESHOLD,
        DEFAULT_STALLED_THRESHOLD,
    )
}

const DEFAULT_STALE_THRESHOLD: u64 = 120;
const DEFAULT_STALLED_THRESHOLD: u64 = 1800;

#[sqlx::test]
async fn dispatch_assigns_pending_job_to_idle_worker(pool: PgPool) {
    let svc = build_service(pool.clone());
    let worker_id = test_helpers::insert_worker(&pool, "dispatch-worker").await;
    let project_id = test_helpers::insert_project_config(&pool, "kaneo-dispatch-1").await;
    let wr_id = test_helpers::insert_pending_work_run(&pool, project_id, "task-dispatch-1").await;

    let summary = svc.dispatch_once().await.expect("Should succeed");

    assert_eq!(summary.dispatched, 1);

    let row = sqlx::query!(
        r#"SELECT worker_id, status as "status: WorkRunStatus" FROM work_runs WHERE id = $1"#,
        wr_id
    )
    .fetch_one(&pool)
    .await
    .expect("Should query work_run");

    assert_eq!(row.worker_id, Some(worker_id));
    assert!(matches!(row.status, WorkRunStatus::Dispatched));
}

#[sqlx::test]
async fn dispatch_skips_when_no_idle_workers(pool: PgPool) {
    let svc = build_service(pool.clone());
    let project_id = test_helpers::insert_project_config(&pool, "kaneo-dispatch-2").await;
    let _wr_id = test_helpers::insert_pending_work_run(&pool, project_id, "task-no-worker").await;

    let summary = svc.dispatch_once().await.expect("Should succeed");

    assert_eq!(summary.dispatched, 0);
    assert_eq!(summary.workers_available, 0);
}

#[sqlx::test]
async fn dispatch_skips_when_no_pending_jobs(pool: PgPool) {
    let svc = build_service(pool.clone());
    let _worker_id = test_helpers::insert_worker(&pool, "idle-no-work").await;

    let summary = svc.dispatch_once().await.expect("Should succeed");

    assert_eq!(summary.dispatched, 0);
    assert_eq!(summary.pending_jobs, 0);
}

#[sqlx::test]
async fn dispatch_sets_redis_flag(pool: PgPool) {
    let store = Arc::new(InMemoryDispatchStore::new());
    let worker_id = test_helpers::insert_worker(&pool, "flag-worker").await;
    let project_id = test_helpers::insert_project_config(&pool, "kaneo-dispatch-3").await;
    let _wr_id = test_helpers::insert_pending_work_run(&pool, project_id, "task-flag").await;

    let svc = DispatcherService::new(
        DispatchRepository::new(),
        crate::services::workers::repository::WorkersRepository::new(),
        crate::services::work_runs::repository::WorkRunsRepository::new(),
        pool,
        store.clone(),
        DEFAULT_STALE_THRESHOLD,
        DEFAULT_STALLED_THRESHOLD,
    );

    svc.dispatch_once().await.expect("Should succeed");

    let dispatched = store.take_dispatched(worker_id).await.expect("Should read");
    assert!(dispatched.is_some(), "Should have a dispatched work_run id");
}

#[sqlx::test]
async fn dispatch_marks_worker_busy(pool: PgPool) {
    let svc = build_service(pool.clone());
    let worker_id = test_helpers::insert_worker(&pool, "busy-after").await;
    let project_id = test_helpers::insert_project_config(&pool, "kaneo-dispatch-4").await;
    let _wr_id = test_helpers::insert_pending_work_run(&pool, project_id, "task-busy").await;

    svc.dispatch_once().await.expect("Should succeed");

    let row = sqlx::query!(
        r#"SELECT status as "status: WorkerStatus" FROM workers WHERE id = $1"#,
        worker_id
    )
    .fetch_one(&pool)
    .await
    .expect("Should query worker");

    assert!(matches!(row.status, WorkerStatus::Busy));
}
