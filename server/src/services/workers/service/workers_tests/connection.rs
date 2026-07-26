use super::*;

#[sqlx::test]
async fn generate_code_returns_new_code(pool: sqlx::PgPool) {
    let svc = svc(pool).await;
    let resp = svc
        .generate_code(DEFAULT_TEAM_ID)
        .await
        .expect("should generate");
    assert_eq!(resp.code.len(), 16);
    assert!(resp.expires_at > Utc::now());
}

#[sqlx::test]
async fn connect_with_valid_code_creates_worker(pool: sqlx::PgPool) {
    let svc = svc(pool).await;
    let code = svc
        .generate_code(DEFAULT_TEAM_ID)
        .await
        .expect("should generate");
    let resp = svc
        .connect(ConnectRequest {
            code: code.code,
            worker_name: "test-runner".to_owned(),
            max_concurrent_jobs: None,
            capabilities: Default::default(),
        })
        .await
        .expect("Should connect");

    assert_eq!(resp.name, "test-runner");
    assert_eq!(resp.refresh_token.len(), 64);
    assert!(!resp.access_token.is_empty());
    assert_eq!(resp.max_concurrent_jobs, DEFAULT_MAX_CONCURRENT_JOBS);
}

#[sqlx::test]
async fn connect_with_capacity_creates_worker_with_capacity(pool: sqlx::PgPool) {
    let svc = svc(pool).await;
    let code = svc
        .generate_code(DEFAULT_TEAM_ID)
        .await
        .expect("should generate");
    let resp = svc
        .connect(ConnectRequest {
            code: code.code,
            worker_name: "capacity-runner".to_owned(),
            max_concurrent_jobs: Some(2),
            capabilities: Default::default(),
        })
        .await
        .expect("Should connect");

    assert_eq!(resp.max_concurrent_jobs, 2);
}

#[sqlx::test]
async fn connect_persists_worker_capabilities(pool: sqlx::PgPool) {
    let svc = svc(pool.clone()).await;
    let code = svc
        .generate_code(DEFAULT_TEAM_ID)
        .await
        .expect("should generate");
    let capabilities = WorkerCapabilities {
        isolation_backends: vec!["host".to_owned(), "docker".to_owned()],
    };

    let resp = svc
        .connect(ConnectRequest {
            code: code.code,
            worker_name: "omp-runner".to_owned(),
            max_concurrent_jobs: None,
            capabilities: capabilities.clone(),
        })
        .await
        .expect("Should connect");

    let worker = WorkersRepository::new()
        .find_by_id(&pool, resp.worker_id)
        .await
        .expect("worker exists");
    let stored: WorkerCapabilities =
        serde_json::from_value(worker.capabilities).expect("capabilities deserialize");
    assert_eq!(stored, capabilities);
}

#[sqlx::test]
async fn connect_with_invalid_code_fails(pool: sqlx::PgPool) {
    let svc = svc(pool).await;
    let err = svc
        .connect(ConnectRequest {
            code: "badcode".to_owned(),
            worker_name: "x".to_owned(),
            max_concurrent_jobs: None,
            capabilities: Default::default(),
        })
        .await
        .expect_err("Should fail");

    assert!(matches!(err, WorkersError::CodeNotFound));
}

#[sqlx::test]
async fn connect_with_expired_code_fails(pool: sqlx::PgPool) {
    let svc = svc(pool.clone()).await;

    svc.code_store
        .save(
            "expired",
            Utc::now() - Duration::minutes(1),
            DEFAULT_TEAM_ID,
        )
        .await
        .expect("pre-insert should succeed");

    let err = svc
        .connect(ConnectRequest {
            code: "expired".to_owned(),
            worker_name: "x".to_owned(),
            max_concurrent_jobs: None,
            capabilities: Default::default(),
        })
        .await
        .expect_err("Should fail");

    assert!(matches!(err, WorkersError::CodeExpired));
}
