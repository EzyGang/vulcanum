use sqlx::PgPool;
use uuid::Uuid;

use crate::services::kaneo::client::{slugify, KaneoClient};
use crate::services::project_configs::errors::ProjectConfigsError;
use crate::services::project_configs::model::{
    ColumnInfo, CreateProjectConfigRequest, ProjectConfig, UpdateProjectConfigRequest,
};
use crate::services::project_configs::repository::{
    ProjectConfigsRepository, UpdateProjectConfigParams,
};

#[derive(Clone)]
pub struct ProjectConfigsService {
    pub repo: ProjectConfigsRepository,
    pub db: PgPool,
    pub kaneo: KaneoClient,
}

impl ProjectConfigsService {
    pub fn new(repo: ProjectConfigsRepository, db: PgPool, kaneo: KaneoClient) -> Self {
        Self { repo, db, kaneo }
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

    #[allow(dead_code)]
    pub async fn list_enabled(&self) -> Result<Vec<ProjectConfig>, ProjectConfigsError> {
        self.repo.list_enabled(&self.db).await
    }

    pub async fn create(
        &self,
        mut params: CreateProjectConfigRequest,
    ) -> Result<ProjectConfig, ProjectConfigsError> {
        normalize_columns(
            &mut params.pickup_column,
            &mut params.progress_column,
            &mut params.target_column,
        );

        self.validate_columns_exist(&params.kaneo_project_id, &params.pickup_column)
            .await?;
        self.validate_columns_exist(&params.kaneo_project_id, &params.progress_column)
            .await?;
        self.validate_columns_exist(&params.kaneo_project_id, &params.target_column)
            .await?;

        self.repo.create(&self.db, &params).await
    }

    pub async fn update(
        &self,
        id: Uuid,
        mut params: UpdateProjectConfigRequest,
    ) -> Result<ProjectConfig, ProjectConfigsError> {
        let existing = self.repo.find_by_id(&self.db, id).await?;

        if let Some(ref col) = params.pickup_column {
            self.validate_columns_exist(&existing.kaneo_project_id, col)
                .await?;
        }
        if let Some(ref col) = params.progress_column {
            self.validate_columns_exist(&existing.kaneo_project_id, col)
                .await?;
        }
        if let Some(ref col) = params.target_column {
            self.validate_columns_exist(&existing.kaneo_project_id, col)
                .await?;
        }

        validate_and_normalize_optional_columns(&mut params);

        self.repo
            .update(
                &self.db,
                id,
                &UpdateProjectConfigParams {
                    pickup_column: params.pickup_column.as_deref(),
                    target_column: params.target_column.as_deref(),
                    progress_column: params.progress_column.as_deref(),
                    prompt_template: params.prompt_template.as_deref(),
                    repo_url: params.repo_url.as_deref(),
                    agents_md: params.agents_md.as_deref(),
                    kaneo_workspace_id: params.kaneo_workspace_id.as_deref(),
                    enabled: params.enabled,
                },
            )
            .await
    }

    pub async fn delete(&self, id: Uuid) -> Result<(), ProjectConfigsError> {
        self.repo.delete(&self.db, id).await
    }

    pub async fn fetch_columns(&self, id: Uuid) -> Result<Vec<ColumnInfo>, ProjectConfigsError> {
        let config = self.repo.find_by_id(&self.db, id).await?;
        let columns = self.kaneo.fetch_columns(&config.kaneo_project_id).await?;

        Ok(columns.iter().map(ColumnInfo::from_kaneo).collect())
    }

    pub async fn fetch_columns_by_kaneo_id(
        &self,
        kaneo_project_id: &str,
    ) -> Result<Vec<ColumnInfo>, ProjectConfigsError> {
        let columns = self.kaneo.fetch_columns(kaneo_project_id).await?;
        Ok(columns.iter().map(ColumnInfo::from_kaneo).collect())
    }

    async fn validate_columns_exist(
        &self,
        project_id: &str,
        column_slug: &str,
    ) -> Result<(), ProjectConfigsError> {
        let columns = self.kaneo.fetch_columns(project_id).await?;

        let found = columns.iter().any(|col| {
            slugify(&col.name) == column_slug
                || col.name.to_lowercase() == column_slug.to_lowercase()
        });

        if found {
            Ok(())
        } else {
            Err(ProjectConfigsError::ColumnNotFound(column_slug.to_owned()))
        }
    }
}

fn normalize_columns(pickup: &mut String, progress: &mut String, target: &mut String) {
    *pickup = slugify(pickup);
    *progress = slugify(progress);
    *target = slugify(target);
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
}
