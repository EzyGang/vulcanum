use super::{
    assert_single_personal_team, test_helpers, HashSet, TeamPrincipal, TeamsRepository,
    TeamsService,
};

#[sqlx::test]
async fn migrations_create_default_team(pool: sqlx::PgPool) {
    let team = TeamsRepository::new()
        .get_default_team(&pool)
        .await
        .expect("default team should exist after migrations");

    assert_eq!(team.id, test_helpers::DEFAULT_TEAM_ID);
    assert_eq!(team.name, "Default team");
    assert_eq!(team.personal_user_id, None);
    assert_eq!(team.prompt_template, "");
    assert_eq!(team.review_prompt_template, "");
}

#[sqlx::test]
async fn resolve_team_uses_default_team_in_single_user_mode(pool: sqlx::PgPool) {
    let svc = TeamsService::new(TeamsRepository::new(), pool);

    let resolved = svc
        .resolve_team(&TeamPrincipal::Instance { team_id: None }, true)
        .await
        .expect("instance without team header should resolve default team");

    assert_eq!(resolved, test_helpers::DEFAULT_TEAM_ID);
}

#[sqlx::test]
async fn multi_user_mode_resolves_user_personal_team(pool: sqlx::PgPool) {
    let user_id = "multi-user-after-single-user";
    test_helpers::teams::insert_user(&pool, user_id).await;
    let svc = TeamsService::new(TeamsRepository::new(), pool.clone());
    let personal_team = svc
        .ensure_personal_team(user_id, "octo")
        .await
        .expect("multi-user login should create personal team");

    let resolved = svc
        .resolve_team(
            &TeamPrincipal::User {
                user_id: user_id.to_owned(),
                team_id: None,
            },
            false,
        )
        .await
        .expect("user without team header should resolve first membership");

    assert_eq!(resolved, personal_team.id);
    assert_ne!(resolved, test_helpers::DEFAULT_TEAM_ID);
}

#[sqlx::test]
async fn ensure_personal_team_is_idempotent(pool: sqlx::PgPool) {
    test_helpers::teams::insert_user(&pool, "personal-idempotent").await;
    let svc = TeamsService::new(TeamsRepository::new(), pool.clone());

    let first = svc
        .ensure_personal_team("personal-idempotent", "octo")
        .await
        .expect("first ensure should create team");
    let second = svc
        .ensure_personal_team("personal-idempotent", "octo-renamed")
        .await
        .expect("second ensure should reuse team");

    assert_eq!(first.id, second.id);
    assert_eq!(first.prompt_template, "");
    assert_eq!(first.review_prompt_template, "");
    assert_single_personal_team(&pool, "personal-idempotent").await;
}

#[sqlx::test]
async fn ensure_personal_team_handles_concurrent_first_login(pool: sqlx::PgPool) {
    test_helpers::teams::insert_user(&pool, "personal-concurrent").await;
    let svc = TeamsService::new(TeamsRepository::new(), pool.clone());

    let mut joins = Vec::new();
    for _ in 0..8 {
        let svc = svc.clone();
        joins.push(tokio::spawn(async move {
            svc.ensure_personal_team("personal-concurrent", "octo")
                .await
                .expect("concurrent ensure should succeed")
        }));
    }

    let mut team_ids = HashSet::new();
    for join in joins {
        team_ids.insert(join.await.expect("task should complete").id);
    }

    assert_eq!(team_ids.len(), 1);
    assert_single_personal_team(&pool, "personal-concurrent").await;
}

#[sqlx::test]
async fn resolve_team_uses_instance_header_in_single_user_mode(pool: sqlx::PgPool) {
    let selected_team_id = test_helpers::teams::insert_team(&pool, "selected-instance-team").await;
    let svc = TeamsService::new(TeamsRepository::new(), pool.clone());

    let resolved = svc
        .resolve_team(
            &TeamPrincipal::Instance {
                team_id: Some(selected_team_id),
            },
            true,
        )
        .await
        .expect("instance team header should resolve");

    assert_eq!(resolved, selected_team_id);
}
