use crate::services::workers::errors::WorkersError;
use crate::services::workers::repository::WorkersRepository;
use crate::services::workers::service::WorkersService;
use chrono::{Duration, Utc};
use std::collections::HashMap;
use tokio::sync::RwLock;

fn build_service(db: sqlx::PgPool) -> WorkersService {
    WorkersService {
        repo: WorkersRepository::new(),
        db,
        jwt_secret: "test-secret".to_owned(),
        codes: RwLock::new(HashMap::new()),
    }
}

#[sqlx::test]
async fn generate_code_returns_new_code(pool: sqlx::PgPool) {
    let svc = build_service(pool);
    let resp = svc.generate_code().await;
    assert_eq!(resp.code.len(), 16);
    assert!(resp.expires_at > Utc::now());
}

#[sqlx::test]
async fn connect_with_valid_code_creates_worker(pool: sqlx::PgPool) {
    let svc = build_service(pool);
    let code = svc.generate_code().await;
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
    let svc = build_service(pool);
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
    let svc = build_service(pool);
    let mut codes = svc.codes.write().await;
    codes.insert("expired".to_owned(), Utc::now() - Duration::minutes(1));
    drop(codes);

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
async fn refresh_with_valid_token_returns_new_access(pool: sqlx::PgPool) {
    let svc = build_service(pool);
    let code = svc.generate_code().await;
    let connect = svc
        .connect(crate::services::workers::model::ConnectRequest {
            code: code.code,
            worker_name: "refresh-test".to_owned(),
        })
        .await
        .unwrap();

    let refresh = svc
        .refresh(crate::services::workers::model::RefreshRequest {
            refresh_token: connect.refresh_token,
        })
        .await
        .expect("Should refresh");

    assert!(!refresh.access_token.is_empty());
    assert!(refresh.expires_at > Utc::now());
}

#[sqlx::test]
async fn refresh_with_invalid_token_fails(pool: sqlx::PgPool) {
    let svc = build_service(pool);
    let err = svc
        .refresh(crate::services::workers::model::RefreshRequest {
            refresh_token: "garbage".to_owned(),
        })
        .await
        .expect_err("Should fail");

    assert!(matches!(err, WorkersError::InvalidRefreshToken));
}
