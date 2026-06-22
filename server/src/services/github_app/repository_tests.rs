use crate::services::github_app::errors::GithubAppError;
use crate::services::github_app::repository::GithubAppRepository;
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
