use std::sync::Arc;

use crate::db::model_providers::ModelProvidersRepository;
use crate::db::project_configs::ProjectConfigsRepository;
use crate::db::provider_configs::IntegrationProvidersRepository;
use crate::db::teams::TeamsRepository;
use crate::models::project_configs::errors::ProjectConfigsError;
use crate::models::project_configs::model::UpdateProjectConfigRequest;
use crate::models::teams::model::{DEFAULT_PROMPT_TEMPLATE, DEFAULT_REVIEW_PROMPT_TEMPLATE};
use crate::services::model_providers::auth::device_flow::InMemoryDeviceFlowStore;
use crate::services::model_providers::auth::encryption::SecretCipher;
use crate::services::model_providers::auth::openai_chatgpt::OpenAiChatGptDeviceAuthProvider;
use crate::services::model_providers::catalog::ModelCatalogClient;
use crate::services::model_providers::service::ModelProvidersService;
use crate::services::project_configs::service::ProjectConfigsService;
use crate::services::teams::service::TeamsService;
use crate::test_helpers;

#[sqlx::test]
async fn get_by_id_rejects_cross_team_config(pool: sqlx::PgPool) {
    let svc = ProjectConfigsService::new(
        ProjectConfigsRepository::new(),
        pool.clone(),
        IntegrationProvidersRepository::new(),
        model_providers_service(pool.clone()),
        TeamsService::new(TeamsRepository::new(), pool.clone()),
    );
    let team_b = test_helpers::insert_team(&pool, "team-b").await;
    let config_id = test_helpers::insert_project_config(&pool, "cross-team-project").await;

    let err = svc
        .get_by_id(config_id, team_b)
        .await
        .expect_err("team B must not read team A project config");

    assert!(matches!(err, ProjectConfigsError::NotFound));
}

#[sqlx::test]
async fn effective_settings_uses_default_prompts_for_empty_team_prompts(pool: sqlx::PgPool) {
    let svc = ProjectConfigsService::new(
        ProjectConfigsRepository::new(),
        pool.clone(),
        IntegrationProvidersRepository::new(),
        model_providers_service(pool.clone()),
        TeamsService::new(TeamsRepository::new(), pool.clone()),
    );
    let config_id = test_helpers::insert_project_config(&pool, "empty-review-prompt").await;
    sqlx::query!(
        "UPDATE project_configs SET prompt_template = '' WHERE id = $1",
        config_id,
    )
    .execute(&pool)
    .await
    .expect("project prompt should be blank");
    let config = svc
        .find_by_id(config_id)
        .await
        .expect("project config should exist");

    let settings = svc
        .effective_settings(&config)
        .await
        .expect("settings should resolve");

    assert_eq!(settings.prompt_template, DEFAULT_PROMPT_TEMPLATE);
    assert_eq!(
        settings.review_prompt_template,
        DEFAULT_REVIEW_PROMPT_TEMPLATE
    );
    assert_eq!(settings.max_in_progress_tasks, 1);
}

#[sqlx::test]
async fn effective_settings_uses_project_capacity_override(pool: sqlx::PgPool) {
    let svc = ProjectConfigsService::new(
        ProjectConfigsRepository::new(),
        pool.clone(),
        IntegrationProvidersRepository::new(),
        model_providers_service(pool.clone()),
        TeamsService::new(TeamsRepository::new(), pool.clone()),
    );
    let config_id = test_helpers::insert_project_config(&pool, "capacity-override").await;
    sqlx::query!(
        "UPDATE project_configs SET max_in_progress_tasks = 3 WHERE id = $1",
        config_id,
    )
    .execute(&pool)
    .await
    .expect("capacity override should update");
    let config = svc
        .find_by_id(config_id)
        .await
        .expect("project config should exist");

    let settings = svc
        .effective_settings(&config)
        .await
        .expect("settings should resolve");

    assert_eq!(settings.max_in_progress_tasks, 3);
}

#[sqlx::test]
async fn update_rejects_cross_team_provider(pool: sqlx::PgPool) {
    let svc = ProjectConfigsService::new(
        ProjectConfigsRepository::new(),
        pool.clone(),
        IntegrationProvidersRepository::new(),
        model_providers_service(pool.clone()),
        TeamsService::new(TeamsRepository::new(), pool.clone()),
    );
    test_helpers::ensure_default_team(&pool).await;
    let team_b = test_helpers::insert_team(&pool, "provider-team-b").await;
    let provider_id = uuid::Uuid::new_v4();
    sqlx::query!(
        "INSERT INTO integration_providers (id, team_id, name, instance_url, api_key) VALUES ($1, $2, $3, $4, $5)",
        provider_id,
        team_b,
        "team-b-provider",
        "cloud.kaneo.app",
        "team-b-key",
    )
    .execute(&pool)
    .await
    .expect("provider should insert");
    let config_id = test_helpers::insert_project_config(&pool, "cross-team-provider-update").await;

    let err = svc
        .update(
            config_id,
            test_helpers::DEFAULT_TEAM_ID,
            UpdateProjectConfigRequest {
                provider_id: Some(provider_id),
                ..Default::default()
            },
        )
        .await
        .expect_err("team A must not attach team B provider");

    assert!(matches!(err, ProjectConfigsError::NoProvider));
}

fn model_providers_service(pool: sqlx::PgPool) -> ModelProvidersService {
    ModelProvidersService::new(
        ModelProvidersRepository::new(),
        pool,
        ModelCatalogClient::new(),
        SecretCipher::new("AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=").expect("test cipher"),
        Arc::new(InMemoryDeviceFlowStore::new()),
        Arc::new(OpenAiChatGptDeviceAuthProvider::new()),
    )
}
