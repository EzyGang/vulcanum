use crate::db::github_app::GithubAppRepository;
use crate::models::github_app::errors::GithubAppError;
use crate::test_helpers;

#[sqlx::test]
async fn insert_installation_does_not_reassign_cross_team_installation(pool: sqlx::PgPool) {
    let repo = GithubAppRepository::new();
    test_helpers::ensure_default_team(&pool).await;
    let team_b = test_helpers::insert_team(&pool, "github-app-team-b").await;
    let installation_id = 123_456;

    repo.insert_installation(
        &pool,
        test_helpers::DEFAULT_TEAM_ID,
        None,
        installation_id,
        "owner-a",
    )
    .await
    .expect("initial installation should insert");

    let err = repo
        .insert_installation(&pool, team_b, None, installation_id, "owner-b")
        .await
        .expect_err("cross-team installation reassignment must fail");

    assert!(matches!(err, GithubAppError::InstallationAlreadyLinked));

    let stored = repo
        .get_installation(&pool, test_helpers::DEFAULT_TEAM_ID)
        .await
        .expect("installation lookup should succeed")
        .expect("installation should remain linked to original team");

    assert_eq!(stored.team_id, test_helpers::DEFAULT_TEAM_ID);
    assert_eq!(stored.account_login, "owner-a");
}

#[sqlx::test]
async fn link_review_identity_updates_only_selected_installation(pool: sqlx::PgPool) {
    let repo = GithubAppRepository::new();
    test_helpers::ensure_default_team(&pool).await;
    let first = repo
        .insert_installation(
            &pool,
            test_helpers::DEFAULT_TEAM_ID,
            None,
            111,
            "first-owner",
        )
        .await
        .expect("insert first installation");
    let second = repo
        .insert_installation(
            &pool,
            test_helpers::DEFAULT_TEAM_ID,
            None,
            222,
            "second-owner",
        )
        .await
        .expect("insert second installation");

    repo.link_review_identity(
        &pool,
        test_helpers::DEFAULT_TEAM_ID,
        first.id,
        "github-user-1",
        "reviewer",
    )
    .await
    .expect("link selected installation");

    let identities = sqlx::query_as::<_, (i64, Option<String>)>(
        "SELECT id, review_identity_user_id FROM github_installations WHERE team_id = $1",
    )
    .bind(test_helpers::DEFAULT_TEAM_ID)
    .fetch_all(&pool)
    .await
    .expect("load installation identities");
    assert_eq!(
        identities
            .iter()
            .find(|row| row.0 == first.id)
            .and_then(|row| row.1.as_deref()),
        Some("github-user-1")
    );
    assert_eq!(
        identities
            .iter()
            .find(|row| row.0 == second.id)
            .and_then(|row| row.1.as_deref()),
        None
    );
}
