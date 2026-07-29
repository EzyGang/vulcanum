use super::{
    insert_team_with_member, invite_service, test_helpers, user_principal, AgentBackend,
    TeamPrincipal, TeamsError, TeamsRepository, TeamsService, UpdateTeamRequest, Utc,
};

#[sqlx::test]
async fn create_for_user_adds_owner_membership(pool: sqlx::PgPool) {
    let user_id = "team-owner-user";
    test_helpers::teams::insert_user(&pool, user_id).await;
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
    assert_eq!(team.review_prompt_template, "");
    assert_eq!(team.prompt_template, "");
}

#[sqlx::test]
async fn member_cannot_rename_team(pool: sqlx::PgPool) {
    let owner_id = "rename-owner";
    let member_id = "rename-member";
    test_helpers::teams::insert_user(&pool, owner_id).await;
    test_helpers::teams::insert_user(&pool, member_id).await;
    let team_id = test_helpers::teams::insert_team(&pool, "rename-team").await;
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
            &UpdateTeamRequest {
                name: Some("new name".to_owned()),
                prompt_template: None,
                agents_md: None,
                primary_model_provider_key: None,
                primary_model_id: None,
                small_model_provider_key: None,
                small_model_id: None,
                review_primary_model_provider_key: None,
                review_primary_model_id: None,
                review_small_model_provider_key: None,
                review_small_model_id: None,
                review_enabled: None,
                review_max_turns: None,
                review_prompt_template: None,
                max_in_progress_tasks: None,
                agent_backend: None,
            },
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
async fn owner_can_change_team_agent_backend(pool: sqlx::PgPool) {
    let owner_id = "backend-owner";
    let team_id = insert_team_with_member(&pool, owner_id, "owner").await;
    let svc = TeamsService::new(TeamsRepository::new(), pool);

    let team = svc
        .update_for_principal(
            team_id,
            &UpdateTeamRequest {
                name: None,
                prompt_template: None,
                agents_md: None,
                primary_model_provider_key: None,
                primary_model_id: None,
                small_model_provider_key: None,
                small_model_id: None,
                review_primary_model_provider_key: None,
                review_primary_model_id: None,
                review_small_model_provider_key: None,
                review_small_model_id: None,
                review_enabled: None,
                review_max_turns: None,
                review_prompt_template: None,
                max_in_progress_tasks: None,
                agent_backend: Some(AgentBackend::OmpRpc),
            },
            &user_principal(owner_id),
            false,
        )
        .await
        .expect("owner should update team backend");

    assert_eq!(team.agent_backend, "omp_rpc");
}

#[sqlx::test]
async fn owner_cannot_save_an_incomplete_review_model_pair(pool: sqlx::PgPool) {
    let owner_id = "review-model-owner";
    let team_id = insert_team_with_member(&pool, owner_id, "owner").await;
    let svc = TeamsService::new(TeamsRepository::new(), pool);

    let err = svc
        .update_for_principal(
            team_id,
            &UpdateTeamRequest {
                name: None,
                prompt_template: None,
                agents_md: None,
                primary_model_provider_key: None,
                primary_model_id: None,
                small_model_provider_key: None,
                small_model_id: None,
                review_primary_model_provider_key: Some(Some("openai".to_owned())),
                review_primary_model_id: None,
                review_small_model_provider_key: None,
                review_small_model_id: None,
                review_enabled: None,
                review_max_turns: None,
                review_prompt_template: None,
                max_in_progress_tasks: None,
                agent_backend: None,
            },
            &user_principal(owner_id),
            false,
        )
        .await
        .expect_err("incomplete review selection should be rejected");

    assert!(matches!(err, TeamsError::InvalidOperation(_)));
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
