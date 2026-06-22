use sqlx::PgPool;
use uuid::Uuid;

use crate::services::project_configs::errors::ProjectConfigsError;
use crate::services::project_configs::model::CreateProjectConfigRequest;
use crate::services::project_configs::repository::{
    ProjectConfigsRepository, UpdateProjectConfigParams,
};
use crate::services::providers::model::IntegrationType;
use crate::test_helpers::DEFAULT_TEAM_ID;

async fn insert_provider(pool: &PgPool) -> Uuid {
    let id = Uuid::new_v4();

    crate::test_helpers::ensure_default_team(pool).await;

    sqlx::query!(
        "INSERT INTO integration_providers (id, team_id, name, instance_url, api_key) VALUES ($1, $2, $3, $4, $5)",
        id,
        DEFAULT_TEAM_ID,
        "test-provider",
        "cloud.kaneo.app",
        "test-key",
    )
    .execute(pool)
    .await
    .expect("Should insert provider");

    id
}

fn test_params(external_project_id: &str, provider_id: Uuid) -> CreateProjectConfigRequest {
    CreateProjectConfigRequest {
        external_project_id: external_project_id.to_owned(),
        name: String::new(),
        enabled: true,
        pickup_column: "to-do".to_owned(),
        progress_column: "in-progress".to_owned(),
        target_column: "in-review".to_owned(),
        prompt_template: Some("Review {{task_title}}".to_owned()),
        repo_full_names: Vec::new(),
        agents_md: Some(String::new()),
        primary_model_provider_key: None,
        primary_model_provider_config_id: None,
        primary_model_id: None,
        small_model_provider_key: None,
        small_model_provider_config_id: None,
        small_model_id: None,
        review_enabled: None,
        review_pickup_column: None,
        review_max_turns: None,
        review_prompt_template: None,
        max_in_progress_tasks: None,
        max_turns: 3,
        external_workspace_id: String::new(),
        integration_type: IntegrationType::Kaneo,
        provider_id,
    }
}

fn test_update_params() -> UpdateProjectConfigParams<'static> {
    UpdateProjectConfigParams {
        name: None,
        pickup_column: None,
        target_column: None,
        progress_column: None,
        max_turns: None,
        prompt_template: None,
        repo_url: None,
        agents_md: None,
        primary_model_provider_key: None,
        primary_model_provider_config_id: None,
        primary_model_id: None,
        small_model_provider_key: None,
        small_model_provider_config_id: None,
        small_model_id: None,
        review_enabled: None,
        review_pickup_column: None,
        review_max_turns: None,
        review_prompt_template: None,
        max_in_progress_tasks: None,
        external_workspace_id: None,
        enabled: None,
        integration_type: None,
        provider_id: None,
    }
}

#[sqlx::test]
async fn create_finds_and_deletes_config(pool: PgPool) {
    let repo = ProjectConfigsRepository::new();
    let provider_id = insert_provider(&pool).await;
    let params = test_params("kaneo-proj-test-create", provider_id);

    let created = repo
        .create(&pool, DEFAULT_TEAM_ID, &params)
        .await
        .expect("Should create config");

    assert_eq!(created.external_project_id, params.external_project_id);
    assert_eq!(created.pickup_column, "to-do");
    assert!(created.enabled);

    let found = repo
        .find_by_id(&pool, created.id)
        .await
        .expect("Should find by id");
    assert_eq!(found.id, created.id);

    repo.delete(&pool, created.id)
        .await
        .expect("Should delete config");

    let result = repo.find_by_id(&pool, created.id).await;
    assert!(matches!(result, Err(ProjectConfigsError::NotFound)));
}

#[sqlx::test]
async fn list_all_returns_configs(pool: PgPool) {
    let repo = ProjectConfigsRepository::new();
    let provider_id = insert_provider(&pool).await;

    let p1 = test_params("kaneo-proj-list-a", provider_id);
    let p2 = CreateProjectConfigRequest {
        external_project_id: "kaneo-proj-list-b".to_owned(),
        name: String::new(),
        prompt_template: Some("Template B".to_owned()),
        ..test_params("kaneo-proj-list-b", provider_id)
    };

    repo.create(&pool, DEFAULT_TEAM_ID, &p1)
        .await
        .expect("Should create p1");
    repo.create(&pool, DEFAULT_TEAM_ID, &p2)
        .await
        .expect("Should create p2");

    let all = repo
        .list_all(&pool, DEFAULT_TEAM_ID)
        .await
        .expect("Should list all");
    assert!(all.len() >= 2);
}

#[sqlx::test]
async fn name_round_trips_through_create_find_and_list(pool: PgPool) {
    let repo = ProjectConfigsRepository::new();
    let provider_id = insert_provider(&pool).await;
    let expected_name = "Vulcanum Project";
    let params = CreateProjectConfigRequest {
        name: expected_name.to_owned(),
        ..test_params("kaneo-proj-name-round-trip", provider_id)
    };

    let created = repo
        .create(&pool, DEFAULT_TEAM_ID, &params)
        .await
        .expect("Should create config with name");
    assert_eq!(created.name, expected_name);

    let found = repo
        .find_by_id(&pool, created.id)
        .await
        .expect("Should find config by id");
    assert_eq!(found.name, expected_name);

    let all = repo
        .list_all(&pool, DEFAULT_TEAM_ID)
        .await
        .expect("Should list configs");
    assert!(
        all.iter()
            .any(|config| config.id == created.id && config.name == expected_name),
        "List should include the created config with its name"
    );
}

#[sqlx::test]
async fn duplicate_external_project_id_fails(pool: PgPool) {
    let repo = ProjectConfigsRepository::new();
    let provider_id = insert_provider(&pool).await;
    let params = test_params("kaneo-proj-dup", provider_id);

    repo.create(&pool, DEFAULT_TEAM_ID, &params)
        .await
        .expect("First create should succeed");

    let result = repo.create(&pool, DEFAULT_TEAM_ID, &params).await;
    assert!(
        matches!(result, Err(ProjectConfigsError::DuplicateExternalProjectId)),
        "Second create with same external_project_id should fail with DuplicateExternalProjectId"
    );
}

#[sqlx::test]
async fn update_partial_fields(pool: PgPool) {
    let repo = ProjectConfigsRepository::new();
    let provider_id = insert_provider(&pool).await;
    let params = test_params("kaneo-proj-update", provider_id);

    let created = repo
        .create(&pool, DEFAULT_TEAM_ID, &params)
        .await
        .expect("Should create");

    let updated = repo
        .update(
            &pool,
            created.id,
            &UpdateProjectConfigParams {
                prompt_template: Some(Some("Updated template")),
                enabled: Some(false),
                ..test_update_params()
            },
        )
        .await
        .expect("Should update");

    assert_eq!(updated.prompt_template.as_deref(), Some("Updated template"));
    assert!(!updated.enabled);
    assert_eq!(updated.pickup_column, "to-do");
}

#[sqlx::test]
async fn replace_repos_allows_replacing_existing_repo(pool: PgPool) {
    let repo = ProjectConfigsRepository::new();
    let provider_id = insert_provider(&pool).await;
    let params = test_params("kaneo-proj-replace-same-repo", provider_id);
    let created = repo
        .create(&pool, DEFAULT_TEAM_ID, &params)
        .await
        .expect("Should create");
    let repo_names = vec!["owner/repo".to_owned()];

    let mut tx = pool.begin().await.expect("Should begin transaction");
    repo.replace_repos(&mut tx, created.id, &repo_names)
        .await
        .expect("Should insert repos");
    tx.commit().await.expect("Should commit transaction");

    let mut tx = pool.begin().await.expect("Should begin transaction");
    repo.replace_repos(&mut tx, created.id, &repo_names)
        .await
        .expect("Should replace existing repos with same repo");
    tx.commit().await.expect("Should commit transaction");

    let found = repo
        .find_by_id(&pool, created.id)
        .await
        .expect("Should find config");

    assert_eq!(found.repo_full_names, repo_names);
}

#[sqlx::test]
async fn replace_repos_deduplicates_repo_names(pool: PgPool) {
    let repo = ProjectConfigsRepository::new();
    let provider_id = insert_provider(&pool).await;
    let params = test_params("kaneo-proj-replace-duplicate-repos", provider_id);
    let created = repo
        .create(&pool, DEFAULT_TEAM_ID, &params)
        .await
        .expect("Should create");
    let repo_names = vec![
        "owner/repo".to_owned(),
        "owner/other".to_owned(),
        "owner/repo".to_owned(),
    ];

    let mut tx = pool.begin().await.expect("Should begin transaction");
    repo.replace_repos(&mut tx, created.id, &repo_names)
        .await
        .expect("Should insert deduplicated repos");
    tx.commit().await.expect("Should commit transaction");

    let found = repo
        .find_by_id(&pool, created.id)
        .await
        .expect("Should find config");

    assert_eq!(
        found.repo_full_names,
        vec!["owner/repo".to_owned(), "owner/other".to_owned()]
    );
}

#[sqlx::test]
async fn update_name_persists(pool: PgPool) {
    let repo = ProjectConfigsRepository::new();
    let provider_id = insert_provider(&pool).await;
    let params = test_params("kaneo-proj-update-name", provider_id);
    let updated_name = "Updated Project Name";

    let created = repo
        .create(&pool, DEFAULT_TEAM_ID, &params)
        .await
        .expect("Should create");
    let updated = repo
        .update(
            &pool,
            created.id,
            &UpdateProjectConfigParams {
                name: Some(updated_name),
                ..test_update_params()
            },
        )
        .await
        .expect("Should update name");

    assert_eq!(updated.name, updated_name);
}

#[sqlx::test]
async fn update_nonexistent_returns_not_found(pool: PgPool) {
    let repo = ProjectConfigsRepository::new();
    let nonexistent_id = Uuid::new_v4();

    let result = repo
        .update(&pool, nonexistent_id, &test_update_params())
        .await;

    assert!(matches!(result, Err(ProjectConfigsError::NotFound)));
}

#[sqlx::test]
async fn delete_nonexistent_returns_not_found(pool: PgPool) {
    let repo = ProjectConfigsRepository::new();
    let nonexistent_id = Uuid::new_v4();

    let result = repo.delete(&pool, nonexistent_id).await;

    assert!(matches!(result, Err(ProjectConfigsError::NotFound)));
}

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
