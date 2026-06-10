use std::sync::Arc;

use crate::config::AppConfig;
use crate::services::workers::errors::WorkersError;
use crate::services::workers::registration_code_store::InMemoryCodeStore;
use crate::services::workers::repository::queries::CreateWorkerParams;
use crate::services::workers::repository::WorkersRepository;
use crate::services::workers::service::WorkersService;
use crate::test_helpers::DEFAULT_TEAM_ID;
use chrono::{Duration, Utc};

fn cfg() -> AppConfig {
    AppConfig {
        db_url: String::new(),
        max_conns: 1,
        poll_period_secs: 30,
        jwt_secret: "test-secret".to_owned(),
        stale_worker_threshold_secs: 120,
        unhealthy_threshold: 3,
        stalled_running_threshold_secs: 1800,
        instance_password: "test-password".to_owned(),
        is_single_user: true,
        redis_url: "redis://127.0.0.1:6379".to_owned(),
        github_app_id: None,
        github_app_private_key: None,
        github_app_slug: None,
        github_oauth_client_id: None,
        github_oauth_client_secret: None,
        github_oauth_redirect_url: None,
    }
}

fn svc(pool: sqlx::PgPool) -> WorkersService {
    let c = cfg();
    WorkersService::new(
        WorkersRepository::new(),
        crate::services::work_runs::repository::WorkRunsRepository::new(),
        pool,
        &c,
        Arc::new(InMemoryCodeStore::new()),
    )
}

#[sqlx::test]
async fn generate_code_returns_new_code(pool: sqlx::PgPool) {
    let svc = svc(pool);
    let resp = svc
        .generate_code(DEFAULT_TEAM_ID)
        .await
        .expect("should generate");
    assert_eq!(resp.code.len(), 16);
    assert!(resp.expires_at > Utc::now());
}

#[sqlx::test]
async fn connect_with_valid_code_creates_worker(pool: sqlx::PgPool) {
    let svc = svc(pool);
    let code = svc
        .generate_code(DEFAULT_TEAM_ID)
        .await
        .expect("should generate");
    let resp = svc
        .connect(crate::services::workers::model::ConnectRequest {
            code: code.code,
            worker_name: "test-runner".to_owned(),
            max_concurrent_jobs: None,
        })
        .await
        .expect("Should connect");

    assert_eq!(resp.name, "test-runner");
    assert_eq!(resp.refresh_token.len(), 64);
    assert!(!resp.access_token.is_empty());
    assert_eq!(
        resp.max_concurrent_jobs,
        crate::services::workers::model::DEFAULT_MAX_CONCURRENT_JOBS
    );
}

#[sqlx::test]
async fn connect_with_capacity_creates_worker_with_capacity(pool: sqlx::PgPool) {
    let svc = svc(pool);
    let code = svc
        .generate_code(DEFAULT_TEAM_ID)
        .await
        .expect("should generate");
    let resp = svc
        .connect(crate::services::workers::model::ConnectRequest {
            code: code.code,
            worker_name: "capacity-runner".to_owned(),
            max_concurrent_jobs: Some(2),
        })
        .await
        .expect("Should connect");

    assert_eq!(resp.max_concurrent_jobs, 2);
}

#[sqlx::test]
async fn connect_with_invalid_code_fails(pool: sqlx::PgPool) {
    let svc = svc(pool);
    let err = svc
        .connect(crate::services::workers::model::ConnectRequest {
            code: "badcode".to_owned(),
            worker_name: "x".to_owned(),
            max_concurrent_jobs: None,
        })
        .await
        .expect_err("Should fail");

    assert!(matches!(err, WorkersError::CodeNotFound));
}

#[sqlx::test]
async fn connect_with_expired_code_fails(pool: sqlx::PgPool) {
    let svc = svc(pool.clone());

    svc.code_store
        .save(
            "expired",
            Utc::now() - Duration::minutes(1),
            DEFAULT_TEAM_ID,
        )
        .await
        .expect("pre-insert should succeed");

    let err = svc
        .connect(crate::services::workers::model::ConnectRequest {
            code: "expired".to_owned(),
            worker_name: "x".to_owned(),
            max_concurrent_jobs: None,
        })
        .await
        .expect_err("Should fail");

    assert!(matches!(err, WorkersError::CodeExpired));
}

#[sqlx::test]
async fn refresh_rotates_token(pool: sqlx::PgPool) {
    let svc = svc(pool);
    let code = svc
        .generate_code(DEFAULT_TEAM_ID)
        .await
        .expect("should generate");
    let connect = svc
        .connect(crate::services::workers::model::ConnectRequest {
            code: code.code,
            worker_name: "refresh-test".to_owned(),
            max_concurrent_jobs: None,
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
    let code = svc
        .generate_code(DEFAULT_TEAM_ID)
        .await
        .expect("should generate");
    let connect = svc
        .connect(crate::services::workers::model::ConnectRequest {
            code: code.code,
            worker_name: "rotation-test".to_owned(),
            max_concurrent_jobs: None,
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
    let capabilities = serde_json::json!({});

    svc.repo
        .create(
            &pool,
            CreateWorkerParams {
                team_id: DEFAULT_TEAM_ID,
                name: "l1",
                refresh_token_hash: "h1",
                refresh_expires_at: expiry,
                capabilities: &capabilities,
                max_concurrent_jobs: crate::services::workers::model::DEFAULT_MAX_CONCURRENT_JOBS,
            },
        )
        .await
        .unwrap();
    svc.repo
        .create(
            &pool,
            CreateWorkerParams {
                team_id: DEFAULT_TEAM_ID,
                name: "l2",
                refresh_token_hash: "h2",
                refresh_expires_at: expiry,
                capabilities: &capabilities,
                max_concurrent_jobs: crate::services::workers::model::DEFAULT_MAX_CONCURRENT_JOBS,
            },
        )
        .await
        .unwrap();

    let workers = svc.list_all(DEFAULT_TEAM_ID).await.expect("Should list");
    assert_eq!(workers.len(), 2);
}
