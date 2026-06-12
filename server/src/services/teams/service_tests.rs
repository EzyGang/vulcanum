use std::collections::HashSet;

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
