use crate::services::model_providers::catalog::ModelCatalogClient;
use crate::services::model_providers::errors::ModelProvidersError;
use crate::services::model_providers::repository::ModelProvidersRepository;
use crate::services::model_providers::service::ModelProvidersService;
use crate::services::project_configs::errors::ProjectConfigsError;
use crate::services::project_configs::model::UpdateProjectConfigRequest;
use crate::services::project_configs::repository::ProjectConfigsRepository;
use crate::services::project_configs::service::ProjectConfigsService;
use crate::services::provider_configs::repository::IntegrationProvidersRepository;
use crate::services::teams::model::DEFAULT_REVIEW_PROMPT_TEMPLATE;
use crate::services::teams::repository::TeamsRepository;
use crate::services::teams::service::TeamsService;
use crate::test_helpers;

#[sqlx::test]
async fn get_by_id_rejects_cross_team_config(pool: sqlx::PgPool) {
    let svc = build_service(pool.clone());
    let team_b = test_helpers::insert_team(&pool, "team-b").await;
    let config_id = test_helpers::insert_project_config(&pool, "cross-team-project").await;

    let err = svc
        .get_by_id(config_id, team_b)
        .await
        .expect_err("team B must not read team A project config");

    assert!(matches!(err, ProjectConfigsError::NotFound));
}

#[sqlx::test]
async fn effective_settings_uses_default_review_prompt_for_empty_team_prompt(pool: sqlx::PgPool) {
    let svc = build_service(pool.clone());
    let config_id = test_helpers::insert_project_config(&pool, "empty-review-prompt").await;
    let config = svc
        .find_by_id(config_id)
        .await
        .expect("project config should exist");

    let settings = svc
        .effective_settings(&config)
        .await
        .expect("settings should resolve");

    assert_eq!(
        settings.review_prompt_template,
        DEFAULT_REVIEW_PROMPT_TEMPLATE
    );
    assert_eq!(settings.max_in_progress_tasks, 1);
}

#[sqlx::test]
async fn effective_settings_uses_project_capacity_override(pool: sqlx::PgPool) {
    let svc = build_service(pool.clone());
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
    let svc = build_service(pool.clone());
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

#[sqlx::test]
async fn update_rejects_cross_team_model_provider_config_without_model(pool: sqlx::PgPool) {
    let svc = build_service(pool.clone());
    let provider_id = insert_default_integration_provider(&pool).await;
    let config_id = test_helpers::insert_project_config_with_provider(
        &pool,
        "foreign-model-provider",
        provider_id,
    )
    .await;
    let team_b = test_helpers::insert_team(&pool, "model-provider-team-b").await;
    let foreign_model_provider_id = insert_model_provider_config(&pool, team_b).await;

    let err = svc
        .update(
            config_id,
            test_helpers::DEFAULT_TEAM_ID,
            UpdateProjectConfigRequest {
                primary_model_provider_config_id: Some(Some(foreign_model_provider_id)),
                primary_model_id: Some(None),
                ..Default::default()
            },
        )
        .await
        .expect_err("team A must not attach team B model provider config");

    assert!(matches!(
        err,
        ProjectConfigsError::ModelProvider(ModelProvidersError::NotFound)
    ));
}

async fn insert_default_integration_provider(pool: &sqlx::PgPool) -> uuid::Uuid {
    let provider_id = uuid::Uuid::new_v4();
    test_helpers::ensure_default_team(pool).await;
    sqlx::query!(
        "INSERT INTO integration_providers (id, team_id, name, instance_url, api_key) VALUES ($1, $2, $3, $4, $5)",
        provider_id,
        test_helpers::DEFAULT_TEAM_ID,
        "default-provider",
        "cloud.kaneo.app",
        "default-key",
    )
    .execute(pool)
    .await
    .expect("provider should insert");
    provider_id
}

fn build_service(pool: sqlx::PgPool) -> ProjectConfigsService {
    ProjectConfigsService::new(
        ProjectConfigsRepository::new(),
        pool.clone(),
        IntegrationProvidersRepository::new(),
        ModelProvidersService::new(
            ModelProvidersRepository::new(),
            pool.clone(),
            ModelCatalogClient::new(),
            "test-secret",
        ),
        TeamsService::new(TeamsRepository::new(), pool),
    )
}

async fn insert_model_provider_config(pool: &sqlx::PgPool, team_id: uuid::Uuid) -> uuid::Uuid {
    let id = uuid::Uuid::new_v4();
    sqlx::query!(
        r#"INSERT INTO model_provider_configs (
            id, team_id, provider_key, display_name, credentials
        ) VALUES ($1, $2, $3, $4, $5)"#,
        id,
        team_id,
        "anthropic",
        "Anthropic",
        serde_json::json!({ "ANTHROPIC_API_KEY": "secret" }),
    )
    .execute(pool)
    .await
    .expect("model provider should insert");
    id
}
