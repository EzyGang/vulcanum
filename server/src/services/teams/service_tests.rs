use std::collections::HashSet;
use std::sync::Arc;

use chrono::{Duration, Utc};
use uuid::Uuid;

use crate::routes::team_auth::TeamPrincipal;
use crate::services::teams::errors::TeamsError;
use crate::services::teams::invite_store::{
    hash_token, invite_redis_key, InMemoryTeamInviteStore, TeamInvitePayload, TeamInviteStore,
};
use crate::services::teams::repository::TeamsRepository;
use crate::services::teams::service::TeamsService;
use crate::test_helpers;

#[tokio::test]
async fn invite_store_hashes_keys_and_consumes_once() {
    let store = InMemoryTeamInviteStore::new();
    let token = "raw-token-value";
    let payload = TeamInvitePayload {
        team_id: Uuid::new_v4(),
        created_by_user_id: "owner".to_owned(),
        role: "member".to_owned(),
        expires_at: Utc::now() + Duration::minutes(30),
    };

    store
        .save(token, &payload)
        .await
        .expect("invite payload should save");

    assert!(!store.contains_raw_key(token).await);
    assert_eq!(
        invite_redis_key(token),
        format!("vulcanum:team_invite:{}", hash_token(token))
    );
    assert_eq!(store.get(token).await.expect("get invite"), Some(payload));
    assert!(store
        .consume(token)
        .await
        .expect("consume invite")
        .is_some());
    assert!(store.consume(token).await.expect("consume again").is_none());
}

#[tokio::test]
async fn invite_store_hides_expired_payloads() {
    let store = InMemoryTeamInviteStore::new();
    let payload = TeamInvitePayload {
        team_id: Uuid::new_v4(),
        created_by_user_id: "owner".to_owned(),
        role: "member".to_owned(),
        expires_at: Utc::now() - Duration::minutes(1),
    };

    store
        .save("expired", &payload)
        .await
        .expect("invite payload should save");

    assert!(store.get("expired").await.expect("get expired").is_none());
    assert!(store
        .consume("expired")
        .await
        .expect("consume expired")
        .is_none());
}

#[sqlx::test]
async fn migrations_create_default_team(pool: sqlx::PgPool) {
    let team = TeamsRepository::new()
        .get_default_team(&pool)
        .await
        .expect("default team should exist after migrations");

    assert_eq!(team.id, test_helpers::DEFAULT_TEAM_ID);
    assert_eq!(team.name, "Default team");
    assert_eq!(team.personal_user_id, None);
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
    test_helpers::insert_user(&pool, user_id).await;
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

#[sqlx::test]
async fn owner_can_create_invite(pool: sqlx::PgPool) {
    let owner_id = "invite-owner";
    let svc = invite_service(pool.clone());
    let team_id = insert_team_with_member(&pool, owner_id, "owner").await;

    let invite = svc
        .create_invite_for_principal(team_id, &user_principal(owner_id), false)
        .await
        .expect("owner should create invite");

    assert_eq!(invite.token.len(), 64);
    assert!(invite.expires_at > Utc::now());
}

#[sqlx::test]
async fn member_cannot_create_invite(pool: sqlx::PgPool) {
    let member_id = "invite-member";
    let svc = invite_service(pool.clone());
    let team_id = insert_team_with_member(&pool, member_id, "member").await;

    let err = svc
        .create_invite_for_principal(team_id, &user_principal(member_id), false)
        .await
        .expect_err("member should not create invite");

    assert!(matches!(err, TeamsError::AccessDenied));
}

#[sqlx::test]
async fn create_invite_rejects_single_user_mode(pool: sqlx::PgPool) {
    let owner_id = "single-user-invite-owner";
    let svc = invite_service(pool.clone());
    let team_id = insert_team_with_member(&pool, owner_id, "owner").await;

    let err = svc
        .create_invite_for_principal(team_id, &user_principal(owner_id), true)
        .await
        .expect_err("single user mode should reject invite creation");

    assert!(matches!(err, TeamsError::AccessDenied));
}

#[sqlx::test]
async fn preview_invite_is_generic_and_does_not_consume(pool: sqlx::PgPool) {
    let owner_id = "preview-owner";
    let svc = invite_service(pool.clone());
    let team_id = insert_team_with_member(&pool, owner_id, "owner").await;
    let invite = svc
        .create_invite_for_principal(team_id, &user_principal(owner_id), false)
        .await
        .expect("owner should create invite");

    let preview = svc
        .preview_invite(&invite.token, false)
        .await
        .expect("preview should succeed");
    let second_preview = svc
        .preview_invite(&invite.token, false)
        .await
        .expect("preview should not consume invite");

    assert_eq!(preview.expires_at, invite.expires_at);
    assert_eq!(second_preview.expires_at, invite.expires_at);
}

#[sqlx::test]
async fn accept_invite_requires_user_principal(pool: sqlx::PgPool) {
    let owner_id = "principal-owner";
    let svc = invite_service(pool.clone());
    let team_id = insert_team_with_member(&pool, owner_id, "owner").await;
    let invite = svc
        .create_invite_for_principal(team_id, &user_principal(owner_id), false)
        .await
        .expect("owner should create invite");

    let err = svc
        .accept_invite_for_principal(
            &invite.token,
            &TeamPrincipal::Instance { team_id: None },
            false,
        )
        .await
        .expect_err("instance principal should not accept invite");

    assert!(matches!(err, TeamsError::AccessDenied));
}

#[sqlx::test]
async fn accept_invite_requires_github_identity(pool: sqlx::PgPool) {
    let owner_id = "github-required-owner";
    let user_id = "github-required-user";
    test_helpers::insert_user(&pool, user_id).await;
    let svc = invite_service(pool.clone());
    let team_id = insert_team_with_member(&pool, owner_id, "owner").await;
    let invite = svc
        .create_invite_for_principal(team_id, &user_principal(owner_id), false)
        .await
        .expect("owner should create invite");

    let err = svc
        .accept_invite_for_principal(&invite.token, &user_principal(user_id), false)
        .await
        .expect_err("github identity should be required");

    assert!(matches!(err, TeamsError::InvalidOperation(_)));
}

#[sqlx::test]
async fn accept_invite_consumes_once_and_adds_member(pool: sqlx::PgPool) {
    let owner_id = "consume-once-owner";
    let user_id = "consume-once-user";
    test_helpers::insert_user(&pool, user_id).await;
    let svc = invite_service(pool.clone());
    let team_id = insert_team_with_member(&pool, owner_id, "owner").await;
    insert_github_identity(&svc, user_id).await;
    let invite = svc
        .create_invite_for_principal(team_id, &user_principal(owner_id), false)
        .await
        .expect("owner should create invite");

    let accepted = svc
        .accept_invite_for_principal(&invite.token, &user_principal(user_id), false)
        .await
        .expect("invite should be accepted");
    let err = svc
        .accept_invite_for_principal(&invite.token, &user_principal(user_id), false)
        .await
        .expect_err("invite should be single-use");
    let role = svc
        .repo
        .get_member_role(&pool, team_id, user_id)
        .await
        .expect("membership should exist");

    assert_eq!(accepted.team_id, team_id);
    assert_eq!(role, "member");
    assert!(matches!(err, TeamsError::InviteInvalid));
}

#[sqlx::test]
async fn accept_invite_existing_member_succeeds(pool: sqlx::PgPool) {
    let owner_id = "existing-member-owner";
    let user_id = "existing-member-user";
    test_helpers::insert_user(&pool, user_id).await;
    let svc = invite_service(pool.clone());
    let team_id = insert_team_with_member(&pool, owner_id, "owner").await;
    svc.repo
        .add_member(&pool, team_id, user_id, "member")
        .await
        .expect("member should be inserted");
    insert_github_identity(&svc, user_id).await;
    let invite = svc
        .create_invite_for_principal(team_id, &user_principal(owner_id), false)
        .await
        .expect("owner should create invite");

    let accepted = svc
        .accept_invite_for_principal(&invite.token, &user_principal(user_id), false)
        .await
        .expect("existing member should accept valid invite");

    assert_eq!(accepted.team_id, team_id);
}

#[sqlx::test]
async fn accept_invite_preserves_owner_role(pool: sqlx::PgPool) {
    let owner_id = "preserve-owner";
    let svc = invite_service(pool.clone());
    let team_id = insert_team_with_member(&pool, owner_id, "owner").await;
    insert_github_identity(&svc, owner_id).await;
    let invite = svc
        .create_invite_for_principal(team_id, &user_principal(owner_id), false)
        .await
        .expect("owner should create invite");

    svc.accept_invite_for_principal(&invite.token, &user_principal(owner_id), false)
        .await
        .expect("owner accepting invite should succeed");
    let role = svc
        .repo
        .get_member_role(&pool, team_id, owner_id)
        .await
        .expect("owner membership should exist");

    assert_eq!(role, "owner");
}

#[sqlx::test]
async fn deleted_team_invite_is_invalid(pool: sqlx::PgPool) {
    let owner_id = "deleted-team-owner";
    let user_id = "deleted-team-user";
    test_helpers::insert_user(&pool, user_id).await;
    let svc = invite_service(pool.clone());
    let team_id = insert_team_with_member(&pool, owner_id, "owner").await;
    insert_github_identity(&svc, user_id).await;
    let invite = svc
        .create_invite_for_principal(team_id, &user_principal(owner_id), false)
        .await
        .expect("owner should create invite");
    svc.repo
        .delete(&pool, team_id)
        .await
        .expect("team should delete");

    let preview_err = svc
        .preview_invite(&invite.token, false)
        .await
        .expect_err("deleted team preview should be invalid");
    let accept_err = svc
        .accept_invite_for_principal(&invite.token, &user_principal(user_id), false)
        .await
        .expect_err("deleted team accept should be invalid");

    assert!(matches!(preview_err, TeamsError::InviteInvalid));
    assert!(matches!(accept_err, TeamsError::InviteInvalid));
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
    test_helpers::insert_user(pool, user_id).await;
    let team_id = test_helpers::insert_team(pool, &format!("{user_id}-team")).await;
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
