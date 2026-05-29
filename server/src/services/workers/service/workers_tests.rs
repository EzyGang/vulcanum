use std::sync::Arc;

use crate::config::AppConfig;
use crate::services::workers::code_store::InMemoryCodeStore;
use crate::services::workers::errors::WorkersError;
use crate::services::workers::repository::WorkersRepository;
use crate::services::workers::service::WorkersService;
use chrono::{Duration, Utc};

fn cfg() -> AppConfig {
    AppConfig {
        db_url: String::new(),
        max_conns: 1,
        poll_period_secs: 30,
        jwt_secret: "test-secret".to_owned(),
        stale_worker_threshold_secs: 120,
        instance_password: "test-password".to_owned(),
        redis_url: String::new(),
    }
}

fn svc(pool: sqlx::PgPool) -> WorkersService {
    let c = cfg();
    WorkersService::new(
        WorkersRepository::new(),
        pool,
        &c,
        Arc::new(InMemoryCodeStore::new()),
    )
}

#[sqlx::test]
async fn generate_code_returns_new_code(pool: sqlx::PgPool) {
    let svc = svc(pool);
    let resp = svc.generate_code().await.expect("should generate");
    assert_eq!(resp.code.len(), 16);
    assert!(resp.expires_at > Utc::now());
}

#[sqlx::test]
async fn connect_with_valid_code_creates_worker(pool: sqlx::PgPool) {
    let svc = svc(pool);
    let code = svc.generate_code().await.expect("should generate");
    let resp = svc
        .connect(crate::services::workers::model::ConnectRequest {
            code: code.code,
            worker_name: "test-runner".to_owned(),
        })
        .await
        .expect("Should connect");

    assert_eq!(resp.name, "test-runner");
    assert_eq!(resp.refresh_token.len(), 64);
    assert!(!resp.access_token.is_empty());
}

#[sqlx::test]
async fn connect_with_invalid_code_fails(pool: sqlx::PgPool) {
    let svc = svc(pool);
    let err = svc
        .connect(crate::services::workers::model::ConnectRequest {
            code: "badcode".to_owned(),
            worker_name: "x".to_owned(),
        })
        .await
        .expect_err("Should fail");

    assert!(matches!(err, WorkersError::CodeNotFound));
}

#[sqlx::test]
async fn connect_with_expired_code_fails(pool: sqlx::PgPool) {
    let svc = svc(pool.clone());

    svc.code_store
        .save("expired", Utc::now() - Duration::minutes(1))
        .await
        .expect("pre-insert should succeed");

    let err = svc
        .connect(crate::services::workers::model::ConnectRequest {
            code: "expired".to_owned(),
            worker_name: "x".to_owned(),
        })
        .await
        .expect_err("Should fail");

    assert!(matches!(err, WorkersError::CodeExpired));
}

#[sqlx::test]
async fn refresh_rotates_token(pool: sqlx::PgPool) {
    let svc = svc(pool);
    let code = svc.generate_code().await.expect("should generate");
    let connect = svc
        .connect(crate::services::workers::model::ConnectRequest {
            code: code.code,
            worker_name: "refresh-test".to_owned(),
        })
        .await
        .unwrap();

    let old_refresh = connect.refresh_token.clone();

    let refresh = svc
        .refresh(crate::services::workers::model::RefreshRequest {
            refresh_token: old_refresh,
        })
        .await
        .expect("Should refresh");

    assert!(!refresh.access_token.is_empty());
    assert!(!refresh.refresh_token.is_empty());
    assert_ne!(refresh.refresh_token, connect.refresh_token);
    assert!(refresh.expires_at > Utc::now());
}

#[sqlx::test]
async fn refresh_old_token_revoked(pool: sqlx::PgPool) {
    let svc = svc(pool);
    let code = svc.generate_code().await.expect("should generate");
    let connect = svc
        .connect(crate::services::workers::model::ConnectRequest {
            code: code.code,
            worker_name: "rotation-test".to_owned(),
        })
        .await
        .unwrap();

    // First refresh consumes the old token
    svc.refresh(crate::services::workers::model::RefreshRequest {
        refresh_token: connect.refresh_token.clone(),
    })
    .await
    .expect("First refresh should succeed");

    // Reusing the old token should fail
    let err = svc
        .refresh(crate::services::workers::model::RefreshRequest {
            refresh_token: connect.refresh_token,
        })
        .await
        .expect_err("Old token should be revoked");

    assert!(matches!(err, WorkersError::InvalidRefreshToken));
}

#[sqlx::test]
async fn refresh_with_invalid_token_fails(pool: sqlx::PgPool) {
    let svc = svc(pool);
    let err = svc
        .refresh(crate::services::workers::model::RefreshRequest {
            refresh_token: "garbage".to_owned(),
        })
        .await
        .expect_err("Should fail");

    assert!(matches!(err, WorkersError::InvalidRefreshToken));
}

#[sqlx::test]
async fn list_all_returns_workers(pool: sqlx::PgPool) {
    let svc = svc(pool.clone());
    let expiry = Utc::now() + Duration::days(30);

    svc.repo
        .create(&pool, "l1", "h1", expiry, &serde_json::json!({}))
        .await
        .unwrap();
    svc.repo
        .create(&pool, "l2", "h2", expiry, &serde_json::json!({}))
        .await
        .unwrap();

    let workers = svc.list_all().await.expect("Should list");
    assert_eq!(workers.len(), 2);
}
