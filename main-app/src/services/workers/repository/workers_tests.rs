use chrono::{Duration, Utc};

use crate::services::workers::model::WorkerStatus;
use crate::services::workers::repository::WorkersRepository;

fn default_expires_at() -> chrono::DateTime<Utc> {
    Utc::now() + Duration::days(30)
}

#[sqlx::test]
async fn create_inserts_worker(pool: sqlx::PgPool) {
    let repo = WorkersRepository::new();
    let worker = repo
        .create(
            &pool,
            "test-worker",
            "refresh-hash",
            default_expires_at(),
            &serde_json::json!({}),
        )
        .await
        .expect("Should create worker");

    assert_eq!(worker.name, "test-worker");
    assert_eq!(worker.refresh_token_hash, "refresh-hash");
    assert!(matches!(worker.status, WorkerStatus::Idle));
}

#[sqlx::test]
async fn find_by_id_returns_worker(pool: sqlx::PgPool) {
    use crate::services::workers::errors::WorkersError;

    let repo = WorkersRepository::new();
    let created = repo
        .create(
            &pool,
            "find-me",
            "hash1",
            default_expires_at(),
            &serde_json::json!({}),
        )
        .await
        .unwrap();

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
    let created = repo
        .create(
            &pool,
            "rt-worker",
            "rt-hash",
            default_expires_at(),
            &serde_json::json!({}),
        )
        .await
        .unwrap();

    let found = repo
        .find_by_refresh_token_hash(&pool, "rt-hash")
        .await
        .expect("Should find by refresh hash");
    assert_eq!(found.id, created.id);
}

#[sqlx::test]
async fn delete_removes_worker(pool: sqlx::PgPool) {
    let repo = WorkersRepository::new();
    let created = repo
        .create(
            &pool,
            "del-me",
            "h",
            default_expires_at(),
            &serde_json::json!({}),
        )
        .await
        .unwrap();

    repo.delete(&pool, created.id).await.expect("Should delete");

    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM workers WHERE id = $1")
        .bind(created.id)
        .fetch_one(&pool)
        .await
        .expect("Should query");
    assert_eq!(count.0, 0);
}

#[sqlx::test]
async fn list_all_returns_workers(pool: sqlx::PgPool) {
    let repo = WorkersRepository::new();
    let expiry = default_expires_at();
    repo.create(&pool, "w1", "h1", expiry, &serde_json::json!({}))
        .await
        .unwrap();
    repo.create(&pool, "w2", "h2", expiry, &serde_json::json!({}))
        .await
        .unwrap();

    let all = repo.list_all(&pool).await.expect("Should list");
    assert_eq!(all.len(), 2);
}

#[sqlx::test]
async fn update_refresh_token_rotates_hash(pool: sqlx::PgPool) {
    let repo = WorkersRepository::new();
    let expiry = default_expires_at();
    let created = repo
        .create(
            &pool,
            "rotate-me",
            "old-hash",
            expiry,
            &serde_json::json!({}),
        )
        .await
        .unwrap();

    let new_expiry = Utc::now() + Duration::days(30);
    let updated = repo
        .update_refresh_token(&pool, created.id, "new-hash", new_expiry)
        .await
        .expect("Should update");

    assert_eq!(updated.refresh_token_hash, "new-hash");
    assert!(updated.refresh_expires_at > expiry);
}
