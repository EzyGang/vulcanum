use std::collections::HashSet;

use crate::routes::team_auth::TeamPrincipal;
use crate::services::teams::repository::TeamsRepository;
use crate::services::teams::service::TeamsService;
use crate::test_helpers;

#[sqlx::test]
async fn ensure_personal_team_is_idempotent(pool: sqlx::PgPool) {
    test_helpers::insert_user(&pool, "personal-idempotent").await;
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
    assert_single_personal_team(&pool, "personal-idempotent").await;
}

#[sqlx::test]
async fn ensure_personal_team_handles_concurrent_first_login(pool: sqlx::PgPool) {
    test_helpers::insert_user(&pool, "personal-concurrent").await;
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
    let selected_team_id = test_helpers::insert_team(&pool, "selected-instance-team").await;
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

#[sqlx::test]
async fn create_for_user_adds_owner_membership(pool: sqlx::PgPool) {
    let user_id = "team-owner-user";
    test_helpers::insert_user(&pool, user_id).await;
    let svc = TeamsService::new(TeamsRepository::new(), pool.clone());

    let team = svc
        .create_for_principal(
            "Owned team",
            &TeamPrincipal::User {
                user_id: user_id.to_owned(),
                team_id: None,
            },
            false,
        )
        .await
        .expect("user should create team");
    let role = sqlx::query_scalar!(
        "SELECT role FROM team_members WHERE team_id = $1 AND user_id = $2",
        team.id,
        user_id,
    )
    .fetch_one(&pool)
    .await
    .expect("owner membership should exist");

    assert_eq!(role, "owner");
}

#[sqlx::test]
async fn member_cannot_rename_team(pool: sqlx::PgPool) {
    let owner_id = "rename-owner";
    let member_id = "rename-member";
    test_helpers::insert_user(&pool, owner_id).await;
    test_helpers::insert_user(&pool, member_id).await;
    let team_id = test_helpers::insert_team(&pool, "rename-team").await;
    let repo = TeamsRepository::new();
    repo.add_member(&pool, team_id, owner_id, "owner")
        .await
        .expect("owner membership should be added");
    repo.add_member(&pool, team_id, member_id, "member")
        .await
        .expect("member membership should be added");
    let svc = TeamsService::new(repo, pool.clone());

    let result = svc
        .update_for_principal(
            team_id,
            "new name",
            &TeamPrincipal::User {
                user_id: member_id.to_owned(),
                team_id: None,
            },
            false,
        )
        .await;

    assert!(result.is_err());
}

async fn assert_single_personal_team(pool: &sqlx::PgPool, user_id: &str) {
    let team_count = sqlx::query_scalar!(
        "SELECT COUNT(*) as \"count!: i64\" FROM teams WHERE personal_user_id = $1",
        user_id,
    )
    .fetch_one(pool)
    .await
    .expect("count personal teams");
    let membership_count = sqlx::query_scalar!(
        "SELECT COUNT(*) as \"count!: i64\"
         FROM team_members tm
         INNER JOIN teams t ON t.id = tm.team_id
         WHERE t.personal_user_id = $1 AND tm.user_id = $1",
        user_id,
    )
    .fetch_one(pool)
    .await
    .expect("count personal memberships");

    assert_eq!(team_count, 1);
    assert_eq!(membership_count, 1);
}
