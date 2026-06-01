use sqlx::PgPool;
use uuid::Uuid;

use crate::services::integrations::model::IntegrationType;
use crate::services::project_configs::errors::ProjectConfigsError;
use crate::services::project_configs::model::CreateProjectConfigRequest;
use crate::services::project_configs::repository::{
    ProjectConfigsRepository, UpdateProjectConfigParams,
};

async fn insert_provider(pool: &PgPool) -> Uuid {
    let id = Uuid::new_v4();

    sqlx::query!(
        "INSERT INTO integration_providers (id, name, instance_url, api_key) VALUES ($1, $2, $3, $4)",
        id,
        "test-provider",
        "cloud.kaneo.app",
        "test-key",
    )
    .execute(pool)
    .await
    .expect("Should insert provider");

    id
}

fn test_params(kaneo_project_id: &str, provider_id: Uuid) -> CreateProjectConfigRequest {
    CreateProjectConfigRequest {
        kaneo_project_id: kaneo_project_id.to_owned(),
        enabled: true,
        pickup_column: "to-do".to_owned(),
        progress_column: "in-progress".to_owned(),
        target_column: "in-review".to_owned(),
        prompt_template: "Review {{task_title}}".to_owned(),
        repo_url: String::new(),
        agents_md: String::new(),
        opencode_config: String::new(),
        kaneo_workspace_id: String::new(),
        integration_type: IntegrationType::Kaneo,
        provider_id,
    }
}

fn test_update_params() -> UpdateProjectConfigParams<'static> {
    UpdateProjectConfigParams {
        pickup_column: None,
        target_column: None,
        progress_column: None,
        prompt_template: None,
        repo_url: None,
        agents_md: None,
        opencode_config: None,
        kaneo_workspace_id: None,
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
        .create(&pool, &params)
        .await
        .expect("Should create config");

    assert_eq!(created.kaneo_project_id, params.kaneo_project_id);
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
        kaneo_project_id: "kaneo-proj-list-b".to_owned(),
        prompt_template: "Template B".to_owned(),
        ..test_params("kaneo-proj-list-b", provider_id)
    };

    repo.create(&pool, &p1).await.expect("Should create p1");
    repo.create(&pool, &p2).await.expect("Should create p2");

    let all = repo.list_all(&pool).await.expect("Should list all");
    assert!(all.len() >= 2);
}

#[sqlx::test]
async fn duplicate_kaneo_project_id_fails(pool: PgPool) {
    let repo = ProjectConfigsRepository::new();
    let provider_id = insert_provider(&pool).await;
    let params = test_params("kaneo-proj-dup", provider_id);

    repo.create(&pool, &params)
        .await
        .expect("First create should succeed");

    let result = repo.create(&pool, &params).await;
    assert!(
        matches!(result, Err(ProjectConfigsError::DuplicateKaneoProjectId)),
        "Second create with same kaneo_project_id should fail with DuplicateKaneoProjectId"
    );
}

#[sqlx::test]
async fn update_partial_fields(pool: PgPool) {
    let repo = ProjectConfigsRepository::new();
    let provider_id = insert_provider(&pool).await;
    let params = test_params("kaneo-proj-update", provider_id);

    let created = repo.create(&pool, &params).await.expect("Should create");

    let updated = repo
        .update(
            &pool,
            created.id,
            &UpdateProjectConfigParams {
                prompt_template: Some("Updated template"),
                enabled: Some(false),
                ..test_update_params()
            },
        )
        .await
        .expect("Should update");

    assert_eq!(updated.prompt_template, "Updated template");
    assert!(!updated.enabled);
    assert_eq!(updated.pickup_column, "to-do");
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
        .create(&pool, &disabled_params)
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

    repo.create(&pool, &enabled_params)
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
            .any(|c| c.kaneo_project_id == "kaneo-proj-disabled"),
        "Disabled config should not be in enabled list"
    );
}
