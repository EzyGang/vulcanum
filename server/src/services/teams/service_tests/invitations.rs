use super::*;

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
    test_helpers::teams::insert_user(&pool, user_id).await;
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
    test_helpers::teams::insert_user(&pool, user_id).await;
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
    test_helpers::teams::insert_user(&pool, user_id).await;
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
    test_helpers::teams::insert_user(&pool, user_id).await;
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
