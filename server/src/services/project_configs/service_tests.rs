use crate::services::model_providers::catalog::ModelCatalogClient;
use crate::services::model_providers::repository::ModelProvidersRepository;
use crate::services::model_providers::service::ModelProvidersService;
use crate::services::project_configs::errors::ProjectConfigsError;
use crate::services::project_configs::repository::ProjectConfigsRepository;
use crate::services::project_configs::service::ProjectConfigsService;
use crate::services::provider_configs::repository::IntegrationProvidersRepository;
use crate::services::teams::repository::TeamsRepository;
use crate::services::teams::service::TeamsService;
use crate::test_helpers;

#[sqlx::test]
async fn get_by_id_rejects_cross_team_config(pool: sqlx::PgPool) {
    let svc = ProjectConfigsService::new(
        ProjectConfigsRepository::new(),
        pool.clone(),
        IntegrationProvidersRepository::new(),
        ModelProvidersService::new(
            ModelProvidersRepository::new(),
            pool.clone(),
            ModelCatalogClient::new(),
        ),
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
