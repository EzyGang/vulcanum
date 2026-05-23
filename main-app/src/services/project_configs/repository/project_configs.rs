use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::services::project_configs::errors::ProjectConfigsError;
use crate::services::project_configs::model::{CreateProjectConfigRequest, ProjectConfig};
use crate::services::project_configs::repository::{
    map_sqlx_error, ProjectConfigsRepository, Queryer, UpdateProjectConfigParams,
};

impl ProjectConfigsRepository {
    pub async fn list_all<'c, Q: Queryer<'c>>(
        &self,
        db: Q,
    ) -> Result<Vec<ProjectConfig>, ProjectConfigsError> {
        sqlx::query_as!(
            ProjectConfig,
            r#"SELECT id, kaneo_project_id, enabled, pickup_column, target_column,
             progress_column, prompt_template, repo_url, agents_md, created_at as "created_at!: DateTime<Utc>"
             FROM project_configs ORDER BY created_at DESC"#,
        )
        .fetch_all(db)
        .await
        .map_err(ProjectConfigsError::from)
    }

    pub async fn find_by_id<'c, Q: Queryer<'c>>(
        &self,
        db: Q,
        id: Uuid,
    ) -> Result<ProjectConfig, ProjectConfigsError> {
        sqlx::query_as!(
            ProjectConfig,
            r#"SELECT id, kaneo_project_id, enabled, pickup_column, target_column,
             progress_column, prompt_template, repo_url, agents_md, created_at as "created_at!: DateTime<Utc>"
             FROM project_configs WHERE id = $1"#,
            id,
        )
        .fetch_optional(db)
        .await?
        .ok_or(ProjectConfigsError::NotFound)
    }

    #[allow(dead_code)]
    pub async fn find_by_kaneo_project_id<'c, Q: Queryer<'c>>(
        &self,
        db: Q,
        kaneo_project_id: &str,
    ) -> Result<Option<ProjectConfig>, ProjectConfigsError> {
        sqlx::query_as!(
            ProjectConfig,
            r#"SELECT id, kaneo_project_id, enabled, pickup_column, target_column,
             progress_column, prompt_template, repo_url, agents_md, created_at as "created_at!: DateTime<Utc>"
             FROM project_configs WHERE kaneo_project_id = $1"#,
            kaneo_project_id,
        )
        .fetch_optional(db)
        .await
        .map_err(ProjectConfigsError::from)
    }

    pub async fn list_enabled<'c, Q: Queryer<'c>>(
        &self,
        db: Q,
    ) -> Result<Vec<ProjectConfig>, ProjectConfigsError> {
        sqlx::query_as!(
            ProjectConfig,
            r#"SELECT id, kaneo_project_id, enabled, pickup_column, target_column,
             progress_column, prompt_template, repo_url, agents_md, created_at as "created_at!: DateTime<Utc>"
             FROM project_configs WHERE enabled = true ORDER BY created_at DESC"#,
        )
        .fetch_all(db)
        .await
        .map_err(ProjectConfigsError::from)
    }

    pub async fn create<'c, Q: Queryer<'c>>(
        &self,
        db: Q,
        params: &CreateProjectConfigRequest,
    ) -> Result<ProjectConfig, ProjectConfigsError> {
        let id = Uuid::new_v4();

        sqlx::query_as!(
            ProjectConfig,
            r#"INSERT INTO project_configs (id, kaneo_project_id, enabled, pickup_column, target_column,
             progress_column, prompt_template, repo_url, agents_md)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
             RETURNING id, kaneo_project_id, enabled, pickup_column, target_column,
             progress_column, prompt_template, repo_url, agents_md, created_at as "created_at!: DateTime<Utc>""#,
            id,
            &params.kaneo_project_id,
            params.enabled,
            &params.pickup_column,
            &params.target_column,
            &params.progress_column,
            &params.prompt_template,
            &params.repo_url,
            &params.agents_md,
        )
        .fetch_one(db)
        .await
        .map_err(map_sqlx_error)
    }

    pub async fn update<'c, Q: Queryer<'c>>(
        &self,
        db: Q,
        id: Uuid,
        params: &UpdateProjectConfigParams<'_>,
    ) -> Result<ProjectConfig, ProjectConfigsError> {
        sqlx::query_as!(
            ProjectConfig,
            r#"UPDATE project_configs SET
             pickup_column = COALESCE($2, pickup_column),
             target_column = COALESCE($3, target_column),
             progress_column = COALESCE($4, progress_column),
             prompt_template = COALESCE($5, prompt_template),
             repo_url = COALESCE($6, repo_url),
             agents_md = COALESCE($7, agents_md),
             enabled = COALESCE($8, enabled)
             WHERE id = $1
             RETURNING id, kaneo_project_id, enabled, pickup_column, target_column,
             progress_column, prompt_template, repo_url, agents_md, created_at as "created_at!: DateTime<Utc>""#,
            id,
            params.pickup_column,
            params.target_column,
            params.progress_column,
            params.prompt_template,
            params.repo_url,
            params.agents_md,
            params.enabled,
        )
        .fetch_optional(db)
        .await?
        .ok_or(ProjectConfigsError::NotFound)
    }

    pub async fn delete<'c, Q: Queryer<'c>>(
        &self,
        db: Q,
        id: Uuid,
    ) -> Result<(), ProjectConfigsError> {
        let rows = sqlx::query!("DELETE FROM project_configs WHERE id = $1", id)
            .execute(db)
            .await?
            .rows_affected();

        if rows == 0 {
            return Err(ProjectConfigsError::NotFound);
        }

        Ok(())
    }
}
