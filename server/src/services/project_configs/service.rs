use sqlx::PgPool;
use uuid::Uuid;

use crate::services::model_providers::service::ModelProvidersService;
use crate::services::project_configs::errors::ProjectConfigsError;
use crate::services::project_configs::model::{
    ColumnInfo, CreateProjectConfigRequest, LookupProjectResult, ProjectConfig, ProjectInfo,
    UpdateProjectConfigRequest, WorkspaceInfo,
};
use crate::services::project_configs::repository::{
    ProjectConfigsRepository, UpdateProjectConfigParams,
};
use crate::services::provider_configs::repository::IntegrationProvidersRepository;
use crate::services::providers::client::IntegrationClient;
use crate::services::providers::model::{IntegrationColumn, IntegrationType};

#[derive(Clone)]
pub struct ProjectConfigsService {
    pub repo: ProjectConfigsRepository,
    pub db: PgPool,
    pub providers_repo: IntegrationProvidersRepository,
    pub model_providers: ModelProvidersService,
}

impl ProjectConfigsService {
    pub fn new(
        repo: ProjectConfigsRepository,
        db: PgPool,
        providers_repo: IntegrationProvidersRepository,
        model_providers: ModelProvidersService,
    ) -> Self {
        Self {
            repo,
            db,
            providers_repo,
            model_providers,
        }
    }

    pub async fn list_all(&self, team_id: Uuid) -> Result<Vec<ProjectConfig>, ProjectConfigsError> {
        self.repo.list_all(&self.db, team_id).await
    }

    pub async fn count_enabled(&self, team_id: Uuid) -> Result<i64, ProjectConfigsError> {
        self.repo.count_enabled(&self.db, team_id).await
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
        params.blocked_column = resolve_column_slug(&all_columns, &params.blocked_column)?;

        self.validate_model_selection(
            team_id,
            params.primary_model_provider_key.as_deref(),
            params.primary_model_id.as_deref(),
        )
        .await?;
        self.validate_model_selection(
            team_id,
            params.small_model_provider_key.as_deref(),
            params.small_model_id.as_deref(),
        )
        .await?;

        self.repo.create(&self.db, team_id, &params).await
    }

    pub async fn update(
        &self,
        id: Uuid,
        team_id: Uuid,
        mut params: UpdateProjectConfigRequest,
    ) -> Result<ProjectConfig, ProjectConfigsError> {
        let existing = self.repo.find_by_id(&self.db, id).await?;
        if existing.team_id != team_id {
            return Err(ProjectConfigsError::NotFound);
        }

        let provider_id = match params.provider_id {
            Some(pid) => pid,
            None => match existing.provider_id {
                Some(pid) => pid,
                None => return Err(ProjectConfigsError::NoProvider),
            },
        };

        if has_column_changes(&params) {
            let client = self.resolve_client(&provider_id, team_id).await?;
            let all_columns = client
                .fetch_columns(&existing.external_project_id)
                .await
                .map_err(ProjectConfigsError::Integration)?;

            resolve_column_if_set(&all_columns, &mut params.pickup_column)?;
            resolve_column_if_set(&all_columns, &mut params.progress_column)?;
            resolve_column_if_set(&all_columns, &mut params.target_column)?;
            resolve_column_if_set(&all_columns, &mut params.blocked_column)?;
        }

        self.validate_model_selection(
            team_id,
            params
                .primary_model_provider_key
                .as_deref()
                .or(existing.primary_model_provider_key.as_deref()),
            params
                .primary_model_id
                .as_deref()
                .or(existing.primary_model_id.as_deref()),
        )
        .await?;
        self.validate_model_selection(
            team_id,
            params
                .small_model_provider_key
                .as_deref()
                .or(existing.small_model_provider_key.as_deref()),
            params
                .small_model_id
                .as_deref()
                .or(existing.small_model_id.as_deref()),
        )
        .await?;

        self.repo
            .update(
                &self.db,
                id,
                &UpdateProjectConfigParams {
                    name: params.name.as_deref(),
                    pickup_column: params.pickup_column.as_deref(),
                    target_column: params.target_column.as_deref(),
                    progress_column: params.progress_column.as_deref(),
                    blocked_column: params.blocked_column.as_deref(),
                    max_turns: params.max_turns,
                    prompt_template: params.prompt_template.as_deref(),
                    repo_url: params.repo_url.as_deref(),
                    agents_md: params.agents_md.as_deref(),
                    opencode_config: params.opencode_config.as_deref(),
                    primary_model_provider_key: params.primary_model_provider_key.as_deref(),
                    primary_model_id: params.primary_model_id.as_deref(),
                    small_model_provider_key: params.small_model_provider_key.as_deref(),
                    small_model_id: params.small_model_id.as_deref(),
                    external_workspace_id: params.external_workspace_id.as_deref(),
                    enabled: params.enabled,
                    integration_type: params.integration_type,
                    provider_id: params.provider_id,
                },
            )
            .await
    }

    pub async fn delete(&self, id: Uuid, team_id: Uuid) -> Result<(), ProjectConfigsError> {
        let existing = self.repo.find_by_id(&self.db, id).await?;
        if existing.team_id != team_id {
            return Err(ProjectConfigsError::NotFound);
        }
        self.repo.delete(&self.db, id).await
    }

    pub async fn lookup_project(
        &self,
        provider_id: &Uuid,
        team_id: Uuid,
        external_project_id: &str,
    ) -> Result<LookupProjectResult, ProjectConfigsError> {
        let client = self.resolve_client(provider_id, team_id).await?;

        let project = client
            .lookup_project(external_project_id)
            .await
            .map_err(ProjectConfigsError::Integration)?;

        let columns = client
            .fetch_columns(external_project_id)
            .await
            .map_err(ProjectConfigsError::Integration)?;

        Ok(LookupProjectResult {
            id: project.id,
            name: project.name,
            slug: project.slug,
            columns: columns.iter().map(ColumnInfo::from).collect(),
        })
    }

    pub async fn fetch_workspaces(
        &self,
        provider_id: &Uuid,
        team_id: Uuid,
    ) -> Result<Vec<WorkspaceInfo>, ProjectConfigsError> {
        let client = self.resolve_client(provider_id, team_id).await?;
        let workspaces = client
            .fetch_workspaces()
            .await
            .map_err(ProjectConfigsError::Integration)?;

        Ok(workspaces.into_iter().map(WorkspaceInfo::from).collect())
    }

    pub async fn fetch_projects(
        &self,
        provider_id: &Uuid,
        team_id: Uuid,
        workspace_id: &str,
    ) -> Result<Vec<ProjectInfo>, ProjectConfigsError> {
        let client = self.resolve_client(provider_id, team_id).await?;
        let projects = client
            .fetch_projects(workspace_id)
            .await
            .map_err(ProjectConfigsError::Integration)?;

        Ok(projects.into_iter().map(ProjectInfo::from).collect())
    }

    async fn resolve_client(
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

    async fn validate_model_selection(
        &self,
        team_id: Uuid,
        provider_key: Option<&str>,
        model_id: Option<&str>,
    ) -> Result<(), ProjectConfigsError> {
        self.model_providers
            .validate_model_selection(team_id, provider_key, model_id)
            .await
            .map_err(ProjectConfigsError::ModelProvider)
    }
}

fn has_column_changes(params: &UpdateProjectConfigRequest) -> bool {
    params.pickup_column.is_some()
        || params.progress_column.is_some()
        || params.target_column.is_some()
        || params.blocked_column.is_some()
}

fn resolve_column_slug(
    columns: &[IntegrationColumn],
    input: &str,
) -> Result<String, ProjectConfigsError> {
    columns
        .iter()
        .find(|c| c.slug == input)
        .map(|c| c.slug.clone())
        .ok_or_else(|| ProjectConfigsError::ColumnNotFound(input.to_owned()))
}

fn resolve_column_if_set(
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
