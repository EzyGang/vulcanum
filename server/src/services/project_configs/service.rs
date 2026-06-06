use sqlx::PgPool;
use uuid::Uuid;

use crate::services::project_configs::errors::ProjectConfigsError;
use crate::services::project_configs::model::{
    ColumnInfo, CreateProjectConfigRequest, LookupProjectResult, ProjectConfig,
    UpdateProjectConfigRequest,
};
use crate::services::project_configs::repository::{
    ProjectConfigsRepository, UpdateProjectConfigParams,
};
use crate::services::provider_configs::repository::IntegrationProvidersRepository;
use crate::services::providers::client::IntegrationClient;
use crate::services::providers::kaneo::client::slugify;
use crate::services::providers::model::{IntegrationColumn, IntegrationType};

#[derive(Clone)]
pub struct ProjectConfigsService {
    pub repo: ProjectConfigsRepository,
    pub db: PgPool,
    pub providers_repo: IntegrationProvidersRepository,
}

impl ProjectConfigsService {
    pub fn new(
        repo: ProjectConfigsRepository,
        db: PgPool,
        providers_repo: IntegrationProvidersRepository,
    ) -> Self {
        Self {
            repo,
            db,
            providers_repo,
        }
    }

    pub async fn list_all(&self) -> Result<Vec<ProjectConfig>, ProjectConfigsError> {
        self.repo.list_all(&self.db).await
    }

    pub async fn count_enabled(&self) -> Result<i64, ProjectConfigsError> {
        self.repo.count_enabled(&self.db).await
    }

    pub async fn get_by_id(&self, id: Uuid) -> Result<ProjectConfig, ProjectConfigsError> {
        self.repo.find_by_id(&self.db, id).await
    }

    pub async fn create(
        &self,
        mut params: CreateProjectConfigRequest,
    ) -> Result<ProjectConfig, ProjectConfigsError> {
        let client = self.resolve_client(&params.provider_id).await?;
        let all_columns = client
            .fetch_columns(&params.external_project_id)
            .await
            .map_err(ProjectConfigsError::Integration)?;

        params.pickup_column = resolve_column_slug(&all_columns, &params.pickup_column)?;
        params.progress_column = resolve_column_slug(&all_columns, &params.progress_column)?;
        params.target_column = resolve_column_slug(&all_columns, &params.target_column)?;
        params.blocked_column = resolve_column_slug(&all_columns, &params.blocked_column)?;

        self.repo.create(&self.db, &params).await
    }

    pub async fn update(
        &self,
        id: Uuid,
        mut params: UpdateProjectConfigRequest,
    ) -> Result<ProjectConfig, ProjectConfigsError> {
        let existing = self.repo.find_by_id(&self.db, id).await?;

        let provider_id = match params.provider_id {
            Some(pid) => pid,
            None => match existing.provider_id {
                Some(pid) => pid,
                None => return Err(ProjectConfigsError::NoProvider),
            },
        };

        if has_column_changes(&params) {
            let client = self.resolve_client(&provider_id).await?;
            let all_columns = client
                .fetch_columns(&existing.external_project_id)
                .await
                .map_err(ProjectConfigsError::Integration)?;

            if let Some(ref input) = params.pickup_column {
                params.pickup_column = Some(resolve_column_slug(&all_columns, input)?);
            }
            if let Some(ref input) = params.progress_column {
                params.progress_column = Some(resolve_column_slug(&all_columns, input)?);
            }
            if let Some(ref input) = params.target_column {
                params.target_column = Some(resolve_column_slug(&all_columns, input)?);
            }
            if let Some(ref input) = params.blocked_column {
                params.blocked_column = Some(resolve_column_slug(&all_columns, input)?);
            }
        }

        self.repo
            .update(
                &self.db,
                id,
                &UpdateProjectConfigParams {
                    pickup_column: params.pickup_column.as_deref(),
                    target_column: params.target_column.as_deref(),
                    progress_column: params.progress_column.as_deref(),
                    blocked_column: params.blocked_column.as_deref(),
                    max_turns: params.max_turns,
                    prompt_template: params.prompt_template.as_deref(),
                    repo_url: params.repo_url.as_deref(),
                    agents_md: params.agents_md.as_deref(),
                    opencode_config: params.opencode_config.as_deref(),
                    external_workspace_id: params.external_workspace_id.as_deref(),
                    enabled: params.enabled,
                    integration_type: params.integration_type,
                    provider_id: params.provider_id,
                },
            )
            .await
    }

    pub async fn delete(&self, id: Uuid) -> Result<(), ProjectConfigsError> {
        self.repo.delete(&self.db, id).await
    }

    pub async fn lookup_project(
        &self,
        provider_id: &Uuid,
        external_project_id: &str,
    ) -> Result<LookupProjectResult, ProjectConfigsError> {
        let client = self.resolve_client(provider_id).await?;

        let project = client
            .lookup_project(external_project_id)
            .await
            .map_err(ProjectConfigsError::Integration)?;

        let columns = client
            .fetch_columns(external_project_id)
            .await
            .map_err(ProjectConfigsError::Integration)?;

        Ok(LookupProjectResult {
            name: project.name,
            columns: columns.iter().map(ColumnInfo::from).collect(),
        })
    }

    async fn resolve_client(
        &self,
        provider_id: &Uuid,
    ) -> Result<IntegrationClient, ProjectConfigsError> {
        let provider = self
            .providers_repo
            .find_by_id(&self.db, *provider_id)
            .await
            .map_err(|_| ProjectConfigsError::NoProvider)?;

        let client = match provider.provider_type {
            IntegrationType::Kaneo => {
                IntegrationClient::new_kaneo(provider.instance_url, provider.api_key)
            }
        };
        Ok(client)
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
    let slug = slugify(input);
    columns
        .iter()
        .find(|c| c.slug == slug)
        .map(|c| c.slug.clone())
        .ok_or_else(|| ProjectConfigsError::ColumnNotFound(input.to_owned()))
}
