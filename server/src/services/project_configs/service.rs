mod lookup;
mod mutation;
mod settings;

use sqlx::PgPool;
use uuid::Uuid;

use crate::services::model_providers::service::ModelProvidersService;
use crate::services::project_configs::errors::ProjectConfigsError;
use crate::services::project_configs::model::{CreateProjectConfigRequest, ProjectConfig};
use crate::services::project_configs::repository::ProjectConfigsRepository;
use crate::services::provider_configs::repository::IntegrationProvidersRepository;
use crate::services::providers::client::IntegrationClient;
use crate::services::providers::model::{IntegrationColumn, IntegrationType};
use crate::services::teams::service::TeamsService;

#[derive(Clone)]
pub struct ProjectConfigsService {
    repo: ProjectConfigsRepository,
    pub db: PgPool,
    pub providers_repo: IntegrationProvidersRepository,
    pub model_providers: ModelProvidersService,
    pub teams: TeamsService,
}

impl ProjectConfigsService {
    pub fn new(
        repo: ProjectConfigsRepository,
        db: PgPool,
        providers_repo: IntegrationProvidersRepository,
        model_providers: ModelProvidersService,
        teams: TeamsService,
    ) -> Self {
        Self {
            repo,
            db,
            providers_repo,
            model_providers,
            teams,
        }
    }

    pub async fn list_all(&self, team_id: Uuid) -> Result<Vec<ProjectConfig>, ProjectConfigsError> {
        self.repo.list_all(&self.db, team_id).await
    }

    pub async fn count_enabled(&self, team_id: Uuid) -> Result<i64, ProjectConfigsError> {
        self.repo.count_enabled(&self.db, team_id).await
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<ProjectConfig, ProjectConfigsError> {
        self.repo.find_by_id(&self.db, id).await
    }

    pub async fn list_enabled(&self) -> Result<Vec<ProjectConfig>, ProjectConfigsError> {
        self.repo.list_enabled(&self.db).await
    }

    pub async fn get_by_id(
        &self,
        id: Uuid,
        team_id: Uuid,
    ) -> Result<ProjectConfig, ProjectConfigsError> {
        let config = self.repo.find_by_id(&self.db, id).await?;
        if config.team_id != team_id {
            return Err(ProjectConfigsError::NotFound);
        }
        Ok(config)
    }

    pub async fn create(
        &self,
        team_id: Uuid,
        mut params: CreateProjectConfigRequest,
    ) -> Result<ProjectConfig, ProjectConfigsError> {
        let client = self.resolve_client(&params.provider_id, team_id).await?;
        let all_columns = client
            .fetch_columns(&params.external_project_id)
            .await
            .map_err(ProjectConfigsError::Integration)?;

        params.pickup_column = resolve_column_slug(&all_columns, &params.pickup_column)?;
        params.progress_column = resolve_column_slug(&all_columns, &params.progress_column)?;
        params.target_column = resolve_column_slug(&all_columns, &params.target_column)?;
        resolve_column_if_set(&all_columns, &mut params.review_pickup_column)?;

        if params.primary_model_provider_config_id.is_none() {
            params.primary_model_provider_config_id = self
                .resolve_legacy_model_provider_key(
                    team_id,
                    params.primary_model_provider_key.as_deref(),
                )
                .await?;
        }
        if params.small_model_provider_config_id.is_none() {
            params.small_model_provider_config_id = self
                .resolve_legacy_model_provider_key(
                    team_id,
                    params.small_model_provider_key.as_deref(),
                )
                .await?;
        }

        self.validate_model_selection(
            team_id,
            params.primary_model_provider_config_id,
            params.primary_model_id.as_deref(),
        )
        .await?;
        self.validate_model_selection(
            team_id,
            params.small_model_provider_config_id,
            params.small_model_id.as_deref(),
        )
        .await?;

        let mut tx = self
            .db
            .begin()
            .await
            .map_err(ProjectConfigsError::Database)?;
        let config = self.repo.create(&mut *tx, team_id, &params).await?;
        self.repo
            .replace_repos(&mut tx, config.id, &params.repo_full_names)
            .await?;
        let created = self.repo.find_by_id(&mut *tx, config.id).await?;
        tx.commit().await.map_err(ProjectConfigsError::Database)?;

        Ok(created)
    }

    pub(super) async fn resolve_client(
        &self,
        provider_id: &Uuid,
        team_id: Uuid,
    ) -> Result<IntegrationClient, ProjectConfigsError> {
        let provider = self
            .providers_repo
            .find_by_id(&self.db, *provider_id, team_id)
            .await
            .map_err(|_| ProjectConfigsError::NoProvider)?;

        let client = match provider.provider_type {
            IntegrationType::Kaneo => {
                IntegrationClient::new_kaneo(provider.instance_url, provider.api_key)
            }
        };
        Ok(client)
    }

    pub(super) async fn validate_model_selection(
        &self,
        team_id: Uuid,
        provider_config_id: Option<Uuid>,
        model_id: Option<&str>,
    ) -> Result<(), ProjectConfigsError> {
        self.model_providers
            .validate_model_selection(team_id, provider_config_id, model_id)
            .await
            .map_err(ProjectConfigsError::ModelProvider)
    }

    pub(super) async fn resolve_legacy_model_provider_key(
        &self,
        team_id: Uuid,
        provider_key: Option<&str>,
    ) -> Result<Option<Uuid>, ProjectConfigsError> {
        let Some(provider_key) = provider_key.filter(|value| !value.is_empty()) else {
            return Ok(None);
        };
        self.model_providers
            .provider_config_id_for_key(team_id, provider_key)
            .await
            .map(Some)
            .map_err(ProjectConfigsError::ModelProvider)
    }
}

pub(super) fn resolve_column_slug(
    columns: &[IntegrationColumn],
    input: &str,
) -> Result<String, ProjectConfigsError> {
    columns
        .iter()
        .find(|c| c.slug == input)
        .map(|c| c.slug.clone())
        .ok_or_else(|| ProjectConfigsError::ColumnNotFound(input.to_owned()))
}

pub(super) fn resolve_column_if_set(
    columns: &[IntegrationColumn],
    column: &mut Option<String>,
) -> Result<(), ProjectConfigsError> {
    match column {
        Some(ref input) => {
            *column = Some(resolve_column_slug(columns, input)?);
            Ok(())
        }
        None => Ok(()),
    }
}
