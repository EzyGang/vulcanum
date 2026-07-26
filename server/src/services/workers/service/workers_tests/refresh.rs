use super::{svc, ConnectRequest, RefreshRequest, Utc, WorkersError, DEFAULT_TEAM_ID};

#[sqlx::test]
async fn refresh_rotates_token(pool: sqlx::PgPool) {
    let svc = svc(pool).await;
    let code = svc
        .generate_code(DEFAULT_TEAM_ID)
        .await
        .expect("should generate");
    let connect = svc
        .connect(ConnectRequest {
            code: code.code,
            worker_name: "refresh-test".to_owned(),
            max_concurrent_jobs: None,
            capabilities: Default::default(),
        })
        .await
        .unwrap();

    let old_refresh = connect.refresh_token.clone();

    let refresh = svc
        .refresh(RefreshRequest {
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
    let svc = svc(pool).await;
    let code = svc
        .generate_code(DEFAULT_TEAM_ID)
        .await
        .expect("should generate");
    let connect = svc
        .connect(ConnectRequest {
            code: code.code,
            worker_name: "rotation-test".to_owned(),
            max_concurrent_jobs: None,
            capabilities: Default::default(),
        })
        .await
        .unwrap();

    // First refresh consumes the old token
    svc.refresh(RefreshRequest {
        refresh_token: connect.refresh_token.clone(),
    })
    .await
    .expect("First refresh should succeed");

    // Reusing the old token should fail
    let err = svc
        .refresh(RefreshRequest {
            refresh_token: connect.refresh_token,
        })
        .await
        .expect_err("Old token should be revoked");

    assert!(matches!(err, WorkersError::InvalidRefreshToken));
}

#[sqlx::test]
async fn refresh_with_invalid_token_fails(pool: sqlx::PgPool) {
    let svc = svc(pool).await;
    let err = svc
        .refresh(RefreshRequest {
            refresh_token: "garbage".to_owned(),
        })
        .await
        .expect_err("Should fail");

    assert!(matches!(err, WorkersError::InvalidRefreshToken));
}
