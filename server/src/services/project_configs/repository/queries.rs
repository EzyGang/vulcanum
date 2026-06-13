use uuid::Uuid;

use crate::queryer::Queryer;
use crate::services::project_configs::errors::ProjectConfigsError;
use crate::services::project_configs::model::{CreateProjectConfigRequest, ProjectConfig};
use crate::services::project_configs::repository::{
    map_sqlx_error, ProjectConfigsRepository, UpdateProjectConfigParams,
};

impl ProjectConfigsRepository {
    pub async fn list_all<'c, Q>(
        &self,
        db: Q,
        team_id: Uuid,
    ) -> Result<Vec<ProjectConfig>, ProjectConfigsError>
    where
        Q: Queryer<'c>,
    {
        sqlx::query_as!(
            ProjectConfig,
            r#"SELECT id, team_id, external_project_id, name, external_workspace_id, integration_type as "integration_type!: _", enabled, pickup_column, target_column,
             progress_column, blocked_column, max_turns, prompt_template, repo_url, agents_md, opencode_config, primary_model_provider_key, primary_model_id,
             small_model_provider_key, small_model_id, created_at, provider_id as "provider_id?"
             FROM project_configs WHERE team_id = $1 ORDER BY created_at DESC"#,
            team_id,
        )
        .fetch_all(db)
        .await
        .map_err(ProjectConfigsError::from)
    }

    pub async fn find_by_id<'c, Q>(
        &self,
        db: Q,
        id: Uuid,
    ) -> Result<ProjectConfig, ProjectConfigsError>
    where
        Q: Queryer<'c>,
    {
        sqlx::query_as!(
            ProjectConfig,
            r#"SELECT id, team_id, external_project_id, name, external_workspace_id, integration_type as "integration_type!: _", enabled, pickup_column, target_column,
             progress_column, blocked_column, max_turns, prompt_template, repo_url, agents_md, opencode_config, primary_model_provider_key, primary_model_id,
             small_model_provider_key, small_model_id, created_at, provider_id as "provider_id?"
             FROM project_configs WHERE id = $1"#,
            id,
        )
        .fetch_optional(db)
        .await?
        .ok_or(ProjectConfigsError::NotFound)
    }

    pub async fn list_enabled<'c, Q>(
        &self,
        db: Q,
    ) -> Result<Vec<ProjectConfig>, ProjectConfigsError>
    where
        Q: Queryer<'c>,
    {
        sqlx::query_as!(
            ProjectConfig,
            r#"SELECT id, team_id, external_project_id, name, external_workspace_id, integration_type as "integration_type!: _", enabled, pickup_column, target_column,
             progress_column, blocked_column, max_turns, prompt_template, repo_url, agents_md, opencode_config, primary_model_provider_key, primary_model_id,
             small_model_provider_key, small_model_id, created_at, provider_id as "provider_id?"
             FROM project_configs WHERE enabled = true ORDER BY created_at DESC"#,
        )
        .fetch_all(db)
        .await
        .map_err(ProjectConfigsError::from)
    }

    pub async fn create<'c, Q>(
        &self,
        db: Q,
        team_id: Uuid,
        params: &CreateProjectConfigRequest,
    ) -> Result<ProjectConfig, ProjectConfigsError>
    where
        Q: Queryer<'c>,
    {
        let id = Uuid::new_v4();

        sqlx::query_as!(
            ProjectConfig,
            r#"INSERT INTO project_configs (id, team_id, external_project_id, name, external_workspace_id, integration_type, enabled, pickup_column, target_column,
             progress_column, blocked_column, max_turns, prompt_template, repo_url, agents_md, opencode_config, provider_id, primary_model_provider_key, primary_model_id,
             small_model_provider_key, small_model_id)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21)
             RETURNING id, team_id, external_project_id, name, external_workspace_id, integration_type as "integration_type!: _", enabled, pickup_column, target_column,
             progress_column, blocked_column, max_turns, prompt_template, repo_url, agents_md, opencode_config, primary_model_provider_key, primary_model_id,
             small_model_provider_key, small_model_id, created_at, provider_id as "provider_id?""#,
            id,
            team_id,
            params.external_project_id,
            params.name,
            params.external_workspace_id,
            params.integration_type as _,
            params.enabled,
            params.pickup_column,
            params.target_column,
            params.progress_column,
            params.blocked_column,
            params.max_turns,
            params.prompt_template,
            params.repo_url,
            params.agents_md,
            params.opencode_config,
            params.provider_id,
            params.primary_model_provider_key.as_deref(),
            params.primary_model_id.as_deref(),
            params.small_model_provider_key.as_deref(),
            params.small_model_id.as_deref(),
        )
        .fetch_one(db)
        .await
        .map_err(map_sqlx_error)
    }

    pub async fn update<'c, Q>(
        &self,
        db: Q,
        id: Uuid,
        params: &UpdateProjectConfigParams<'_>,
    ) -> Result<ProjectConfig, ProjectConfigsError>
    where
        Q: Queryer<'c>,
    {
        sqlx::query_as!(
            ProjectConfig,
            r#"UPDATE project_configs SET
             name = COALESCE($2, name),
             pickup_column = COALESCE($3, pickup_column),
             target_column = COALESCE($4, target_column),
             progress_column = COALESCE($5, progress_column),
             blocked_column = COALESCE($6, blocked_column),
             max_turns = COALESCE($7, max_turns),
             prompt_template = COALESCE($8, prompt_template),
             repo_url = COALESCE($9, repo_url),
             agents_md = COALESCE($10, agents_md),
             enabled = COALESCE($11, enabled),
             external_workspace_id = COALESCE($12, external_workspace_id),
             integration_type = COALESCE($13, integration_type),
             provider_id = COALESCE($14, provider_id),
             opencode_config = COALESCE($15, opencode_config),
             primary_model_provider_key = CASE WHEN $16 THEN $17 ELSE primary_model_provider_key END,
             primary_model_id = CASE WHEN $18 THEN $19 ELSE primary_model_id END,
             small_model_provider_key = CASE WHEN $20 THEN $21 ELSE small_model_provider_key END,
             small_model_id = CASE WHEN $22 THEN $23 ELSE small_model_id END
             WHERE id = $1
             RETURNING id, team_id, external_project_id, name, external_workspace_id, integration_type as "integration_type!: _", enabled, pickup_column, target_column,
             progress_column, blocked_column, max_turns, prompt_template, repo_url, agents_md, opencode_config, primary_model_provider_key, primary_model_id,
             small_model_provider_key, small_model_id, created_at, provider_id as "provider_id?""#,
            id,
            params.name,
            params.pickup_column,
            params.target_column,
            params.progress_column,
            params.blocked_column,
            params.max_turns,
            params.prompt_template,
            params.repo_url,
            params.agents_md,
            params.enabled,
            params.external_workspace_id,
            params.integration_type as _,
            params.provider_id,
            params.opencode_config,
            params.primary_model_provider_key.is_some(),
            params.primary_model_provider_key.flatten(),
            params.primary_model_id.is_some(),
            params.primary_model_id.flatten(),
            params.small_model_provider_key.is_some(),
            params.small_model_provider_key.flatten(),
            params.small_model_id.is_some(),
            params.small_model_id.flatten(),
        )
        .fetch_optional(db)
        .await?
        .ok_or(ProjectConfigsError::NotFound)
    }

    pub async fn delete<'c, Q>(&self, db: Q, id: Uuid) -> Result<(), ProjectConfigsError>
    where
        Q: Queryer<'c>,
    {
        let rows = sqlx::query!("DELETE FROM project_configs WHERE id = $1", id)
            .execute(db)
            .await?
            .rows_affected();

        if rows == 0 {
            return Err(ProjectConfigsError::NotFound);
        }

        Ok(())
    }

    pub async fn count_enabled<'c, Q>(
        &self,
        db: Q,
        team_id: Uuid,
    ) -> Result<i64, ProjectConfigsError>
    where
        Q: Queryer<'c>,
    {
        let count = sqlx::query_scalar!(
            "SELECT COUNT(*) as count FROM project_configs WHERE enabled = true AND team_id = $1",
            team_id,
        )
        .fetch_one(db)
        .await?
        .unwrap_or(0);

        Ok(count)
    }
}
