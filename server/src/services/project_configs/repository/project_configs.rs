use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::queryer::Queryer;
use crate::services::integrations::model::IntegrationType;
use crate::services::project_configs::errors::ProjectConfigsError;
use crate::services::project_configs::model::{CreateProjectConfigRequest, ProjectConfig};
use crate::services::project_configs::repository::{
    map_sqlx_error, ProjectConfigsRepository, UpdateProjectConfigParams,
};

impl ProjectConfigsRepository {
    pub async fn list_all<'c, Q: Queryer<'c>>(
        &self,
        db: Q,
    ) -> Result<Vec<ProjectConfig>, ProjectConfigsError> {
        sqlx::query_as!(
            ProjectConfig,
            r#"SELECT id, kaneo_project_id, kaneo_workspace_id, integration_type as "integration_type!: _", enabled, pickup_column, target_column,
             progress_column, prompt_template, repo_url, agents_md, opencode_config, created_at as "created_at!: DateTime<Utc>", provider_id
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
            r#"SELECT id, kaneo_project_id, kaneo_workspace_id, integration_type as "integration_type!: _", enabled, pickup_column, target_column,
             progress_column, prompt_template, repo_url, agents_md, opencode_config, created_at as "created_at!: DateTime<Utc>", provider_id
             FROM project_configs WHERE id = $1"#,
            id,
        )
        .fetch_optional(db)
        .await?
        .ok_or(ProjectConfigsError::NotFound)
    }

    pub async fn list_enabled<'c, Q: Queryer<'c>>(
        &self,
        db: Q,
    ) -> Result<Vec<ProjectConfig>, ProjectConfigsError> {
        sqlx::query_as!(
            ProjectConfig,
            r#"SELECT id, kaneo_project_id, kaneo_workspace_id, integration_type as "integration_type!: _", enabled, pickup_column, target_column,
             progress_column, prompt_template, repo_url, agents_md, opencode_config, created_at as "created_at!: DateTime<Utc>", provider_id
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
            r#"INSERT INTO project_configs (id, kaneo_project_id, kaneo_workspace_id, integration_type, enabled, pickup_column, target_column,
             progress_column, prompt_template, repo_url, agents_md, opencode_config, provider_id)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
             RETURNING id, kaneo_project_id, kaneo_workspace_id, integration_type as "integration_type!: _", enabled, pickup_column, target_column,
             progress_column, prompt_template, repo_url, agents_md, opencode_config, created_at as "created_at!: DateTime<Utc>", provider_id"#,
            id,
            &params.kaneo_project_id,
            &params.kaneo_workspace_id,
            &params.integration_type as &IntegrationType,
            params.enabled,
            &params.pickup_column,
            &params.target_column,
            &params.progress_column,
            &params.prompt_template,
            &params.repo_url,
            &params.agents_md,
            &params.opencode_config,
            params.provider_id,
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
             enabled = COALESCE($8, enabled),
             kaneo_workspace_id = COALESCE($9, kaneo_workspace_id),
             integration_type = COALESCE($10, integration_type),
             provider_id = COALESCE($11, provider_id),
             opencode_config = COALESCE($12, opencode_config)
             WHERE id = $1
             RETURNING id, kaneo_project_id, kaneo_workspace_id, integration_type as "integration_type!: _", enabled, pickup_column, target_column,
             progress_column, prompt_template, repo_url, agents_md, opencode_config, created_at as "created_at!: DateTime<Utc>", provider_id"#,
            id,
            params.pickup_column,
            params.target_column,
            params.progress_column,
            params.prompt_template,
            params.repo_url,
            params.agents_md,
            params.enabled,
            params.kaneo_workspace_id,
            params.integration_type.as_ref() as Option<&IntegrationType>,
            params.provider_id,
            params.opencode_config,
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

    pub async fn count_enabled<'c, Q: Queryer<'c>>(
        &self,
        db: Q,
    ) -> Result<i64, ProjectConfigsError> {
        sqlx::query_scalar!(
            "SELECT COUNT(*) as \"count!: i64\" FROM project_configs WHERE enabled = true"
        )
        .fetch_one(db)
        .await
        .map_err(ProjectConfigsError::from)
    }
}
