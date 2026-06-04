use sqlx::PgPool;
use uuid::Uuid;

use crate::services::integration_providers::repository::IntegrationProvidersRepository;
use crate::services::integrations::client::IntegrationClient;
use crate::services::integrations::model::IntegrationType;
use crate::services::kaneo::client::slugify;
use crate::services::project_configs::errors::ProjectConfigsError;
use crate::services::project_configs::model::{
    ColumnInfo, CreateProjectConfigRequest, LookupProjectResult, ProjectConfig,
    UpdateProjectConfigRequest,
};
use crate::services::project_configs::repository::{
    ProjectConfigsRepository, UpdateProjectConfigParams,
};

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
        normalize_columns(
            &mut params.pickup_column,
            &mut params.progress_column,
            &mut params.target_column,
            &mut params.blocked_column,
        );

        let client = self.resolve_client(&params.provider_id).await?;
        let all_columns = client
            .fetch_columns(&params.kaneo_project_id)
            .await
            .map_err(ProjectConfigsError::Integration)?;

        for col_slug in [
            &params.pickup_column,
            &params.progress_column,
            &params.target_column,
        ] {
            if !all_columns.iter().any(|c| &c.slug == col_slug) {
                return Err(ProjectConfigsError::ColumnNotFound(col_slug.to_owned()));
            }
        }

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
                .fetch_columns(&existing.kaneo_project_id)
                .await
                .map_err(ProjectConfigsError::Integration)?;

            if let Some(ref col) = params.pickup_column {
                if !all_columns.iter().any(|c| &c.slug == col) {
                    return Err(ProjectConfigsError::ColumnNotFound(col.to_owned()));
                }
            }
            if let Some(ref col) = params.progress_column {
                if !all_columns.iter().any(|c| &c.slug == col) {
                    return Err(ProjectConfigsError::ColumnNotFound(col.to_owned()));
                }
            }
            if let Some(ref col) = params.target_column {
                if !all_columns.iter().any(|c| &c.slug == col) {
                    return Err(ProjectConfigsError::ColumnNotFound(col.to_owned()));
                }
            }
        }

        validate_and_normalize_optional_columns(&mut params);

        validate_and_normalize_optional_columns(&mut params);

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
                    kaneo_workspace_id: params.kaneo_workspace_id.as_deref(),
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
        kaneo_project_id: &str,
    ) -> Result<LookupProjectResult, ProjectConfigsError> {
        let client = self.resolve_client(provider_id).await?;

        let project = client
            .lookup_project(kaneo_project_id)
            .await
            .map_err(ProjectConfigsError::Integration)?;

        let columns = client
            .fetch_columns(kaneo_project_id)
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
    // max_turns is intentionally excluded — column validation is only
    // required for Kanéo column names, not for turn-limit configuration.
    params.pickup_column.is_some()
        || params.progress_column.is_some()
        || params.target_column.is_some()
        || params.blocked_column.is_some()
}

fn normalize_columns(
    pickup: &mut String,
    progress: &mut String,
    target: &mut String,
    blocked: &mut String,
) {
    *pickup = slugify(pickup);
    *progress = slugify(progress);
    *target = slugify(target);
    *blocked = slugify(blocked);
}

fn validate_and_normalize_optional_columns(params: &mut UpdateProjectConfigRequest) {
    if let Some(ref mut col) = params.pickup_column {
        *col = slugify(col);
    }
    if let Some(ref mut col) = params.progress_column {
        *col = slugify(col);
    }
    if let Some(ref mut col) = params.target_column {
        *col = slugify(col);
    }
    if let Some(ref mut col) = params.blocked_column {
        *col = slugify(col);
    }
}
