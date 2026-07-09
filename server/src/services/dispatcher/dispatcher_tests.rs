use std::sync::Arc;

use sqlx::PgPool;

use crate::db::dispatcher::DispatchRepository;
use crate::db::work_runs::WorkRunsRepository;
use crate::models::work_runs::model::WorkRunStatus;
use crate::models::workers::model::WorkerStatus;
use crate::services::dispatcher::dispatch_store::{DispatchStore, InMemoryDispatchStore};
use crate::services::dispatcher::service::DispatcherService;
use crate::test_helpers;

fn build_service(pool: PgPool) -> DispatcherService {
    DispatcherService::new(
        DispatchRepository::new(),
        crate::db::workers::WorkersRepository::new(),
        crate::db::work_runs::WorkRunsRepository::new(),
        pool,
        Arc::new(InMemoryDispatchStore::default()),
        DEFAULT_STALE_THRESHOLD,
        DEFAULT_STALLED_THRESHOLD,
    )
}

const DEFAULT_STALE_THRESHOLD: u64 = 120;
const DEFAULT_STALLED_THRESHOLD: u64 = 1800;

async fn worker_load(pool: &PgPool, worker_id: uuid::Uuid) -> (WorkerStatus, i32) {
    sqlx::query_as::<_, (WorkerStatus, i32)>(
        "SELECT status, active_jobs FROM workers WHERE id = $1",
    )
    .bind(worker_id)
    .fetch_one(pool)
    .await
    .expect("Should query worker load")
}

async fn age_work_run(pool: &PgPool, work_run_id: uuid::Uuid) {
    sqlx::query("ALTER TABLE work_runs DISABLE TRIGGER trg_work_runs_updated_at")
        .execute(pool)
        .await
        .expect("Should disable work_runs updated_at trigger");

    sqlx::query("UPDATE work_runs SET updated_at = NOW() - INTERVAL '10 minutes' WHERE id = $1")
        .bind(work_run_id)
        .execute(pool)
        .await
        .expect("Should age work run");

    sqlx::query("ALTER TABLE work_runs ENABLE TRIGGER trg_work_runs_updated_at")
        .execute(pool)
        .await
        .expect("Should enable work_runs updated_at trigger");
}

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
        crate::db::workers::WorkersRepository::new(),
        crate::db::work_runs::WorkRunsRepository::new(),
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

#[sqlx::test]
async fn dispatch_recovers_capacity_from_orphaned_dispatched_run(pool: PgPool) {
    let svc = build_service(pool.clone());
    let worker_id = test_helpers::insert_worker(&pool, "orphaned-dispatch-worker").await;
    let project_id = test_helpers::insert_project_config(&pool, "kaneo-orphaned-dispatch").await;
    let wr_id =
        test_helpers::insert_pending_work_run(&pool, project_id, "task-orphaned-dispatch").await;

    svc.dispatch_once().await.expect("Should dispatch");
    age_work_run(&pool, wr_id).await;

    let summary = svc.dispatch_once().await.expect("Should recover stale run");

    assert_eq!(summary.orphaned, 1);
    assert_eq!(summary.dispatched, 1);

    let (run_status, run_worker_id) = sqlx::query_as::<_, (WorkRunStatus, Option<uuid::Uuid>)>(
        "SELECT status, worker_id FROM work_runs WHERE id = $1",
    )
    .bind(wr_id)
    .fetch_one(&pool)
    .await
    .expect("Should query redispatched run");
    let (status, active_jobs) = worker_load(&pool, worker_id).await;

    assert!(matches!(run_status, WorkRunStatus::Dispatched));
    assert_eq!(run_worker_id, Some(worker_id));
    assert!(matches!(status, WorkerStatus::Busy));
    assert_eq!(active_jobs, 1);
}

#[sqlx::test]
async fn reset_stalled_running_releases_worker_capacity(pool: PgPool) {
    let worker_id = test_helpers::insert_worker(&pool, "stalled-running-worker").await;
    let project_id = test_helpers::insert_project_config(&pool, "kaneo-stalled-running").await;
    let wr_id =
        test_helpers::insert_running_work_run(&pool, project_id, "task-stalled-running", worker_id)
            .await;

    age_work_run(&pool, wr_id).await;
    let reset = WorkRunsRepository::new()
        .reset_stalled_running(&pool, 60)
        .await
        .expect("Should reset stalled run");

    let (run_status, run_worker_id) = sqlx::query_as::<_, (WorkRunStatus, Option<uuid::Uuid>)>(
        "SELECT status, worker_id FROM work_runs WHERE id = $1",
    )
    .bind(wr_id)
    .fetch_one(&pool)
    .await
    .expect("Should query reset run");
    let (status, active_jobs) = worker_load(&pool, worker_id).await;

    assert_eq!(reset, 1);
    assert!(matches!(run_status, WorkRunStatus::Pending));
    assert_eq!(run_worker_id, None);
    assert!(matches!(status, WorkerStatus::Idle));
    assert_eq!(active_jobs, 0);
}
