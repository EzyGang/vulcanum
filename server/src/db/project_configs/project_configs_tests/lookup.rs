use super::{
    insert_provider, insert_provider_named, test_params, test_update_params,
    CreateProjectConfigRequest, PgPool, ProjectConfigsRepository, UpdateProjectConfigParams,
    DEFAULT_TEAM_ID,
};

#[sqlx::test]
async fn list_enabled_only_returns_enabled(pool: PgPool) {
    let repo = ProjectConfigsRepository::new();
    let provider_id = insert_provider(&pool).await;

    let enabled_params = test_params("kaneo-proj-enabled", provider_id);
    let disabled_params = test_params("kaneo-proj-disabled", provider_id);

    let created = repo
        .create(&pool, DEFAULT_TEAM_ID, &disabled_params)
        .await
        .expect("Should create disabled");

    repo.update(
        &pool,
        created.id,
        &UpdateProjectConfigParams {
            enabled: Some(false),
            ..test_update_params()
        },
    )
    .await
    .expect("Should disable");

    repo.create(&pool, DEFAULT_TEAM_ID, &enabled_params)
        .await
        .expect("Should create enabled");

    let enabled_list = repo.list_enabled(&pool).await.expect("Should list enabled");

    assert!(
        enabled_list.iter().all(|c| c.enabled),
        "All returned configs should be enabled"
    );
    assert!(
        !enabled_list
            .iter()
            .any(|c| c.external_project_id == "kaneo-proj-disabled"),
        "Disabled config should not be in enabled list"
    );
}

#[sqlx::test]
async fn find_by_provider_project_returns_config_for_matching_provider_and_project(pool: PgPool) {
    let repo = ProjectConfigsRepository::new();
    let provider_a_id = insert_provider_named(&pool, "test-provider-a").await;
    let provider_b_id = insert_provider_named(&pool, "test-provider-b").await;
    let external_project_id = "shared-provider-project";

    let provider_a_config = repo
        .create(
            &pool,
            DEFAULT_TEAM_ID,
            &CreateProjectConfigRequest {
                name: "Provider A project".to_owned(),
                ..test_params(external_project_id, provider_a_id)
            },
        )
        .await
        .expect("Should create provider A config");
    let provider_b_config = repo
        .create(
            &pool,
            DEFAULT_TEAM_ID,
            &CreateProjectConfigRequest {
                name: "Provider B project".to_owned(),
                ..test_params(external_project_id, provider_b_id)
            },
        )
        .await
        .expect("Should create provider B config");

    let found_provider_a = repo
        .find_by_provider_project(&pool, DEFAULT_TEAM_ID, provider_a_id, external_project_id)
        .await
        .expect("Provider A lookup should succeed")
        .expect("Provider A config should be found");
    let found_provider_b = repo
        .find_by_provider_project(&pool, DEFAULT_TEAM_ID, provider_b_id, external_project_id)
        .await
        .expect("Provider B lookup should succeed")
        .expect("Provider B config should be found");

    assert_eq!(found_provider_a.id, provider_a_config.id);
    assert_eq!(found_provider_a.provider_id, Some(provider_a_id));
    assert_eq!(found_provider_a.name, "Provider A project");
    assert_eq!(found_provider_b.id, provider_b_config.id);
    assert_eq!(found_provider_b.provider_id, Some(provider_b_id));
    assert_eq!(found_provider_b.name, "Provider B project");
}

#[sqlx::test]
async fn find_by_provider_project_returns_none_for_missing_project_or_other_team(pool: PgPool) {
    let repo = ProjectConfigsRepository::new();
    let provider_id = insert_provider_named(&pool, "test-provider-scoped").await;
    let external_project_id = "team-scoped-provider-project";
    let other_team_id =
        crate::test_helpers::teams::insert_team(&pool, "Other provider-project team").await;

    repo.create(
        &pool,
        DEFAULT_TEAM_ID,
        &CreateProjectConfigRequest {
            name: "Default team project".to_owned(),
            ..test_params(external_project_id, provider_id)
        },
    )
    .await
    .expect("Should create default team config");

    let missing_project = repo
        .find_by_provider_project(
            &pool,
            DEFAULT_TEAM_ID,
            provider_id,
            "missing-provider-project",
        )
        .await
        .expect("Missing project lookup should succeed");
    let other_team = repo
        .find_by_provider_project(&pool, other_team_id, provider_id, external_project_id)
        .await
        .expect("Other team lookup should succeed");

    assert!(missing_project.is_none());
    assert!(other_team.is_none());
}
