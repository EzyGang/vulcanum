mod invitations;
mod invite_store;
mod management;
mod resolution;

use std::collections::HashSet;
use std::sync::Arc;

use chrono::{Duration, Utc};
use uuid::Uuid;
use vulcanum_shared::api::wire::AgentBackend;

use crate::db::teams::TeamsRepository;
use crate::models::auth::model::TeamPrincipal;
use crate::models::teams::errors::TeamsError;
use crate::models::teams::model::UpdateTeamRequest;
use crate::services::teams::invite_store::{
    hash_token, invite_redis_key, InMemoryTeamInviteStore, TeamInvitePayload, TeamInviteStore,
};
use crate::services::teams::service::TeamsService;
use crate::test_helpers;
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

fn invite_service(pool: sqlx::PgPool) -> TeamsService {
    TeamsService::new_with_invite_store(
        TeamsRepository::new(),
        pool,
        Arc::new(InMemoryTeamInviteStore::new()),
    )
}

fn user_principal(user_id: &str) -> TeamPrincipal {
    TeamPrincipal::User {
        user_id: user_id.to_owned(),
        team_id: None,
    }
}

async fn insert_team_with_member(pool: &sqlx::PgPool, user_id: &str, role: &str) -> Uuid {
    test_helpers::teams::insert_user(pool, user_id).await;
    let team_id = test_helpers::teams::insert_team(pool, &format!("{user_id}-team")).await;
    TeamsRepository::new()
        .add_member(pool, team_id, user_id, role)
        .await
        .expect("membership should be inserted");
    team_id
}

async fn insert_github_identity(svc: &TeamsService, user_id: &str) {
    svc.repo
        .upsert_identity(
            &svc.db,
            user_id,
            "github",
            &format!("{user_id}-github-id"),
            user_id,
        )
        .await
        .expect("github identity should be inserted");
}
