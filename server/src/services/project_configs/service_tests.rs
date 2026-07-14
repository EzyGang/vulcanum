use crate::db::project_configs::ProjectConfigsRepository;
use crate::db::provider_configs::IntegrationProvidersRepository;
use crate::db::teams::TeamsRepository;
use crate::models::project_configs::errors::ProjectConfigsError;
use crate::models::project_configs::model::{
    CreateProjectConfigRequest, UpdateProjectConfigRequest,
};
use crate::models::teams::model::{DEFAULT_PROMPT_TEMPLATE, DEFAULT_REVIEW_PROMPT_TEMPLATE};
use crate::services::project_configs::service::ProjectConfigsService;
use crate::services::teams::service::TeamsService;
use crate::test_helpers;

#[sqlx::test]
async fn get_by_id_rejects_cross_team_config(pool: sqlx::PgPool) {
    let svc = ProjectConfigsService::new(
        ProjectConfigsRepository::new(),
        pool.clone(),
        IntegrationProvidersRepository::new(),
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
async fn effective_settings_rejects_invalid_stored_agent_backend(pool: sqlx::PgPool) {
    let svc = ProjectConfigsService::new(
        ProjectConfigsRepository::new(),
        pool.clone(),
        IntegrationProvidersRepository::new(),
        TeamsService::new(TeamsRepository::new(), pool.clone()),
    );
    let config_id = test_helpers::insert_project_config(&pool, "invalid-agent-backend").await;
    sqlx::query!(
        "UPDATE teams SET agent_backend = 'not_a_backend' WHERE id = $1",
        test_helpers::DEFAULT_TEAM_ID,
    )
    .execute(&pool)
    .await
    .expect("team backend should update");
    let config = svc
        .find_by_id(config_id)
        .await
        .expect("project config should exist");

    let err = svc
        .effective_settings(&config)
        .await
        .expect_err("invalid stored backend should not default");

    assert!(matches!(err, ProjectConfigsError::InvalidAgentBackend(_)));
}

#[sqlx::test]
async fn create_rejects_enabled_automation_without_repos(pool: sqlx::PgPool) {
    let svc = ProjectConfigsService::new(
        ProjectConfigsRepository::new(),
        pool.clone(),
        IntegrationProvidersRepository::new(),
        TeamsService::new(TeamsRepository::new(), pool.clone()),
    );

    let err = svc
        .create(
            test_helpers::DEFAULT_TEAM_ID,
            CreateProjectConfigRequest {
                external_project_id: "missing-repos".to_owned(),
                name: String::new(),
                external_workspace_id: String::new(),
                enabled: true,
                pickup_column: "todo".to_owned(),
                progress_column: "doing".to_owned(),
                review_column: "review".to_owned(),
                done_column: "done".to_owned(),
                max_turns: 3,
                prompt_template: None,
                repo_full_names: Vec::new(),
                agents_md: None,
                review_enabled: None,
                review_max_turns: None,
                review_prompt_template: None,
                max_in_progress_tasks: None,
                integration_type: Default::default(),
                provider_id: uuid::Uuid::new_v4(),
            },
        )
        .await
        .expect_err("enabled automation without repositories must be rejected");

    assert!(matches!(err, ProjectConfigsError::RepositoriesRequired));
}

#[sqlx::test]
async fn update_rejects_cross_team_provider(pool: sqlx::PgPool) {
    let svc = ProjectConfigsService::new(
        ProjectConfigsRepository::new(),
        pool.clone(),
        IntegrationProvidersRepository::new(),
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

#[sqlx::test]
async fn update_rejects_enabling_automation_without_repos(pool: sqlx::PgPool) {
    let svc = ProjectConfigsService::new(
        ProjectConfigsRepository::new(),
        pool.clone(),
        IntegrationProvidersRepository::new(),
        TeamsService::new(TeamsRepository::new(), pool.clone()),
    );
    let config_id = test_helpers::insert_project_config(&pool, "empty-repo-enable").await;
    sqlx::query!(
        "UPDATE project_configs SET enabled = false WHERE id = $1",
        config_id
    )
    .execute(&pool)
    .await
    .expect("config should disable");

    let err = svc
        .update(
            config_id,
            test_helpers::DEFAULT_TEAM_ID,
            UpdateProjectConfigRequest {
                enabled: Some(true),
                ..Default::default()
            },
        )
        .await
        .expect_err("enabling automation without repositories must be rejected");

    assert!(matches!(err, ProjectConfigsError::RepositoriesRequired));
}

#[sqlx::test]
async fn update_allows_empty_repo_list_without_enabling_automation(pool: sqlx::PgPool) {
    let svc = ProjectConfigsService::new(
        ProjectConfigsRepository::new(),
        pool.clone(),
        IntegrationProvidersRepository::new(),
        TeamsService::new(TeamsRepository::new(), pool.clone()),
    );
    let provider_id = insert_provider(&pool).await;
    let config_id =
        test_helpers::insert_project_config_with_provider(&pool, "empty-repo-update", provider_id)
            .await;

    let config = svc
        .update(
            config_id,
            test_helpers::DEFAULT_TEAM_ID,
            UpdateProjectConfigRequest {
                repo_full_names: Some(Vec::new()),
                ..Default::default()
            },
        )
        .await
        .expect("empty repository update should be allowed unless automation is being enabled");

    assert!(config.repo_full_names.is_empty());
}

async fn insert_provider(pool: &sqlx::PgPool) -> uuid::Uuid {
    let provider_id = uuid::Uuid::new_v4();
    sqlx::query!(
        "INSERT INTO integration_providers (id, team_id, name, instance_url, api_key) VALUES ($1, $2, $3, $4, $5)",
        provider_id,
        test_helpers::DEFAULT_TEAM_ID,
        "default-team-provider",
        "cloud.kaneo.app",
        "default-team-key",
    )
    .execute(pool)
    .await
    .expect("provider should insert");

    provider_id
}
