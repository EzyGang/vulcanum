use uuid::Uuid;

use crate::queryer::Queryer;
use crate::services::project_configs::errors::ProjectConfigsError;
use crate::services::project_configs::model::{CreateProjectConfigRequest, ProjectConfig};
use crate::services::project_configs::repository::{
    map_sqlx_error, ProjectConfigsRepository, UpdateProjectConfigParams,
};

const PROJECT_CONFIG_FIELDS: &str = "id, team_id, external_project_id, name, external_workspace_id, integration_type, enabled, pickup_column, target_column, progress_column, blocked_column, max_turns, prompt_template, repo_url, agents_md, opencode_config, primary_model_provider_key, primary_model_id, small_model_provider_key, small_model_id, created_at, provider_id";

impl ProjectConfigsRepository {
    pub async fn list_all<'c, Q: Queryer<'c>>(
        &self,
        db: Q,
        team_id: Uuid,
    ) -> Result<Vec<ProjectConfig>, ProjectConfigsError> {
        sqlx::query_as::<_, ProjectConfig>(&format!(
            "SELECT {PROJECT_CONFIG_FIELDS} FROM project_configs WHERE team_id = $1 ORDER BY created_at DESC"
        ))
        .bind(team_id)
        .fetch_all(db)
        .await
        .map_err(ProjectConfigsError::from)
    }

    pub async fn find_by_id<'c, Q: Queryer<'c>>(
        &self,
        db: Q,
        id: Uuid,
    ) -> Result<ProjectConfig, ProjectConfigsError> {
        sqlx::query_as::<_, ProjectConfig>(&format!(
            "SELECT {PROJECT_CONFIG_FIELDS} FROM project_configs WHERE id = $1"
        ))
        .bind(id)
        .fetch_optional(db)
        .await?
        .ok_or(ProjectConfigsError::NotFound)
    }

    pub async fn list_enabled<'c, Q: Queryer<'c>>(
        &self,
        db: Q,
    ) -> Result<Vec<ProjectConfig>, ProjectConfigsError> {
        sqlx::query_as::<_, ProjectConfig>(&format!(
            "SELECT {PROJECT_CONFIG_FIELDS} FROM project_configs WHERE enabled = true ORDER BY created_at DESC"
        ))
        .fetch_all(db)
        .await
        .map_err(ProjectConfigsError::from)
    }

    pub async fn create<'c, Q: Queryer<'c>>(
        &self,
        db: Q,
        team_id: Uuid,
        params: &CreateProjectConfigRequest,
    ) -> Result<ProjectConfig, ProjectConfigsError> {
        let id = Uuid::new_v4();

        sqlx::query_as::<_, ProjectConfig>(
            r#"INSERT INTO project_configs (id, team_id, external_project_id, name, external_workspace_id, integration_type, enabled, pickup_column, target_column,
             progress_column, blocked_column, max_turns, prompt_template, repo_url, agents_md, opencode_config, provider_id, primary_model_provider_key, primary_model_id, small_model_provider_key, small_model_id)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21)
             RETURNING id, team_id, external_project_id, name, external_workspace_id, integration_type, enabled, pickup_column, target_column,
             progress_column, blocked_column, max_turns, prompt_template, repo_url, agents_md, opencode_config, primary_model_provider_key, primary_model_id, small_model_provider_key, small_model_id, created_at, provider_id"#,
        )
        .bind(id)
        .bind(team_id)
        .bind(&params.external_project_id)
        .bind(&params.name)
        .bind(&params.external_workspace_id)
        .bind(params.integration_type)
        .bind(params.enabled)
        .bind(&params.pickup_column)
        .bind(&params.target_column)
        .bind(&params.progress_column)
        .bind(&params.blocked_column)
        .bind(params.max_turns)
        .bind(&params.prompt_template)
        .bind(&params.repo_url)
        .bind(&params.agents_md)
        .bind(&params.opencode_config)
        .bind(params.provider_id)
        .bind(params.primary_model_provider_key.as_deref())
        .bind(params.primary_model_id.as_deref())
        .bind(params.small_model_provider_key.as_deref())
        .bind(params.small_model_id.as_deref())
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
        sqlx::query_as::<_, ProjectConfig>(
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
             primary_model_provider_key = COALESCE($16, primary_model_provider_key),
             primary_model_id = COALESCE($17, primary_model_id),
             small_model_provider_key = COALESCE($18, small_model_provider_key),
             small_model_id = COALESCE($19, small_model_id)
             WHERE id = $1
             RETURNING id, team_id, external_project_id, name, external_workspace_id, integration_type, enabled, pickup_column, target_column,
             progress_column, blocked_column, max_turns, prompt_template, repo_url, agents_md, opencode_config, primary_model_provider_key, primary_model_id, small_model_provider_key, small_model_id, created_at, provider_id"#,
        )
        .bind(id)
        .bind(params.name)
        .bind(params.pickup_column)
        .bind(params.target_column)
        .bind(params.progress_column)
        .bind(params.blocked_column)
        .bind(params.max_turns)
        .bind(params.prompt_template)
        .bind(params.repo_url)
        .bind(params.agents_md)
        .bind(params.enabled)
        .bind(params.external_workspace_id)
        .bind(params.integration_type)
        .bind(params.provider_id)
        .bind(params.opencode_config)
        .bind(params.primary_model_provider_key)
        .bind(params.primary_model_id)
        .bind(params.small_model_provider_key)
        .bind(params.small_model_id)
        .fetch_optional(db)
        .await?
        .ok_or(ProjectConfigsError::NotFound)
    }

    pub async fn delete<'c, Q: Queryer<'c>>(
        &self,
        db: Q,
        id: Uuid,
    ) -> Result<(), ProjectConfigsError> {
        let rows = sqlx::query("DELETE FROM project_configs WHERE id = $1")
            .bind(id)
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
        team_id: Uuid,
    ) -> Result<i64, ProjectConfigsError> {
        sqlx::query_scalar(
            "SELECT COUNT(*) as count FROM project_configs WHERE enabled = true AND team_id = $1",
        )
        .bind(team_id)
        .fetch_one(db)
        .await
        .map_err(ProjectConfigsError::from)
    }
}
