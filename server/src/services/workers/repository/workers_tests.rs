use chrono::{Duration, Utc};

use crate::services::workers::model::{Worker, WorkerStatus};
use crate::services::workers::repository::queries::CreateWorkerParams;
use crate::services::workers::repository::WorkersRepository;
use crate::test_helpers::DEFAULT_TEAM_ID;

const DEFAULT_TEST_CAPACITY: i32 = 3;

fn default_expires_at() -> chrono::DateTime<Utc> {
    Utc::now() + Duration::days(30)
}

async fn create_worker(
    repo: &WorkersRepository,
    pool: &sqlx::PgPool,
    name: &str,
    refresh_token_hash: &str,
    refresh_expires_at: chrono::DateTime<Utc>,
) -> Worker {
    let capabilities = serde_json::json!({});
    repo.create(
        pool,
        CreateWorkerParams {
            team_id: DEFAULT_TEAM_ID,
            name,
            refresh_token_hash,
            refresh_expires_at,
            capabilities: &capabilities,
            max_concurrent_jobs: DEFAULT_TEST_CAPACITY,
        },
    )
    .await
    .expect("Should create worker")
}

#[sqlx::test]
async fn create_inserts_worker(pool: sqlx::PgPool) {
    let repo = WorkersRepository::new();
    let worker = create_worker(
        &repo,
        &pool,
        "test-worker",
        "refresh-hash",
        default_expires_at(),
    )
    .await;

    assert_eq!(worker.name, "test-worker");
    assert_eq!(worker.refresh_token_hash, "refresh-hash");
    assert_eq!(worker.max_concurrent_jobs, DEFAULT_TEST_CAPACITY);
    assert!(matches!(worker.status, WorkerStatus::Idle));
}

#[sqlx::test]
async fn find_by_id_returns_worker(pool: sqlx::PgPool) {
    let repo = WorkersRepository::new();
    let created = create_worker(&repo, &pool, "find-me", "hash1", default_expires_at()).await;

    let found = repo
        .find_by_id(&pool, created.id)
        .await
        .expect("Should find worker");
    assert_eq!(found.id, created.id);
    assert_eq!(found.name, "find-me");
}

#[sqlx::test]
async fn find_by_id_missing_returns_error(pool: sqlx::PgPool) {
    use crate::services::workers::errors::WorkersError;
    use uuid::Uuid;

    let repo = WorkersRepository::new();
    let err = repo
        .find_by_id(&pool, Uuid::new_v4())
        .await
        .expect_err("Should error");

    assert!(matches!(err, WorkersError::WorkerNotFound));
}

#[sqlx::test]
async fn find_by_refresh_token_hash_returns_worker(pool: sqlx::PgPool) {
    let repo = WorkersRepository::new();
    let created = create_worker(&repo, &pool, "rt-worker", "rt-hash", default_expires_at()).await;

    let found = repo
        .find_by_refresh_token_hash(&pool, "rt-hash")
        .await
        .expect("Should find by refresh hash");
    assert_eq!(found.id, created.id);
}

#[sqlx::test]
async fn delete_removes_worker(pool: sqlx::PgPool) {
    let repo = WorkersRepository::new();
    let created = create_worker(&repo, &pool, "del-me", "h", default_expires_at()).await;

    repo.delete(&pool, created.id).await.expect("Should delete");

    let row = sqlx::query!(
        "SELECT COUNT(*) as count FROM workers WHERE id = $1",
        created.id
    )
    .fetch_one(&pool)
    .await
    .expect("Should query");
    assert_eq!(row.count.unwrap(), 0);
}

#[sqlx::test]
async fn list_all_returns_workers(pool: sqlx::PgPool) {
    let repo = WorkersRepository::new();
    let expiry = default_expires_at();
    create_worker(&repo, &pool, "w1", "h1", expiry).await;
    create_worker(&repo, &pool, "w2", "h2", expiry).await;

    let all = repo
        .list_all(&pool, DEFAULT_TEAM_ID)
        .await
        .expect("Should list");
    assert_eq!(all.len(), 2);
}

#[sqlx::test]
async fn update_refresh_token_rotates_hash(pool: sqlx::PgPool) {
    let repo = WorkersRepository::new();
    let expiry = default_expires_at();
    let created = create_worker(&repo, &pool, "rotate-me", "old-hash", expiry).await;

    let new_expiry = Utc::now() + Duration::days(30);
    let updated = repo
        .update_refresh_token(&pool, created.id, "new-hash", new_expiry)
        .await
        .expect("Should update");

    assert_eq!(updated.refresh_token_hash, "new-hash");
    assert!(updated.refresh_expires_at > expiry);
}
