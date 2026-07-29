use crate::test_helpers;

#[sqlx::test]
async fn pull_request_installation_matches_repository_owner(pool: sqlx::PgPool) {
    test_helpers::teams::ensure_default_team(&pool).await;
    let state = test_helpers::state::build_state(pool).await;

    state
        .github
        .repo
        .insert_installation(
            &state.github.db,
            test_helpers::DEFAULT_TEAM_ID,
            None,
            101,
            "EzyGang",
        )
        .await
        .expect("insert organization installation");
    state
        .github
        .repo
        .insert_installation(
            &state.github.db,
            test_helpers::DEFAULT_TEAM_ID,
            None,
            202,
            "Galtozzy",
        )
        .await
        .expect("insert personal installation");

    let installation = state
        .github
        .pull_request_installation(test_helpers::DEFAULT_TEAM_ID, "ezygang")
        .await
        .expect("find installation for repository owner");

    assert_eq!(installation.github_installation_id, 101);
}
