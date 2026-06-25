use uuid::Uuid;

use crate::queryer::Queryer;
use crate::services::project_configs::errors::ProjectConfigsError;
use crate::services::project_configs::model::{CreateProjectConfigRequest, ProjectConfig};
use crate::services::project_configs::repository::{
    map_sqlx_error, ProjectConfigsRepository, UpdateProjectConfigParams,
};
use crate::util::github::{github_repo_url, GITHUB_REPO_URL_PREFIX};

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
            r#"SELECT pc.id, pc.team_id, pc.external_project_id, pc.name, pc.external_workspace_id, pc.integration_type as "integration_type!: _", pc.enabled, pc.pickup_column, pc.target_column,
             pc.progress_column, pc.max_turns, pc.prompt_template, pc.repo_url,
             COALESCE(array_agg(pcr.repo_full_name ORDER BY pcr.position) FILTER (WHERE pcr.id IS NOT NULL), ARRAY[]::TEXT[]) as "repo_full_names!",
             COALESCE(array_agg(pcr.repo_url ORDER BY pcr.position) FILTER (WHERE pcr.id IS NOT NULL), ARRAY[]::TEXT[]) as "repo_urls!",
             pc.agents_md, pc.primary_model_provider_key, pc.primary_model_id,
             pc.small_model_provider_key, pc.small_model_id,
             pc.review_enabled, pc.review_pickup_column, pc.review_max_turns, pc.review_prompt_template, pc.max_in_progress_tasks,
             pc.created_at, pc.provider_id as "provider_id?"
             FROM project_configs pc LEFT JOIN project_config_repos pcr ON pcr.project_config_id = pc.id
             WHERE pc.team_id = $1
             GROUP BY pc.id
             ORDER BY pc.created_at DESC"#,
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
            r#"SELECT pc.id, pc.team_id, pc.external_project_id, pc.name, pc.external_workspace_id, pc.integration_type as "integration_type!: _", pc.enabled, pc.pickup_column, pc.target_column,
             pc.progress_column, pc.max_turns, pc.prompt_template, pc.repo_url,
             COALESCE(array_agg(pcr.repo_full_name ORDER BY pcr.position) FILTER (WHERE pcr.id IS NOT NULL), ARRAY[]::TEXT[]) as "repo_full_names!",
             COALESCE(array_agg(pcr.repo_url ORDER BY pcr.position) FILTER (WHERE pcr.id IS NOT NULL), ARRAY[]::TEXT[]) as "repo_urls!",
              pc.agents_md, pc.primary_model_provider_key, pc.primary_model_id,
              pc.small_model_provider_key, pc.small_model_id,
              pc.review_enabled, pc.review_pickup_column, pc.review_max_turns, pc.review_prompt_template, pc.max_in_progress_tasks,
              pc.created_at, pc.provider_id as "provider_id?"
             FROM project_configs pc LEFT JOIN project_config_repos pcr ON pcr.project_config_id = pc.id
             WHERE pc.id = $1
             GROUP BY pc.id"#,
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
            r#"SELECT pc.id, pc.team_id, pc.external_project_id, pc.name, pc.external_workspace_id, pc.integration_type as "integration_type!: _", pc.enabled, pc.pickup_column, pc.target_column,
             pc.progress_column, pc.max_turns, pc.prompt_template, pc.repo_url,
             COALESCE(array_agg(pcr.repo_full_name ORDER BY pcr.position) FILTER (WHERE pcr.id IS NOT NULL), ARRAY[]::TEXT[]) as "repo_full_names!",
             COALESCE(array_agg(pcr.repo_url ORDER BY pcr.position) FILTER (WHERE pcr.id IS NOT NULL), ARRAY[]::TEXT[]) as "repo_urls!",
              pc.agents_md, pc.primary_model_provider_key, pc.primary_model_id,
              pc.small_model_provider_key, pc.small_model_id,
              pc.review_enabled, pc.review_pickup_column, pc.review_max_turns, pc.review_prompt_template, pc.max_in_progress_tasks,
              pc.created_at, pc.provider_id as "provider_id?"
             FROM project_configs pc LEFT JOIN project_config_repos pcr ON pcr.project_config_id = pc.id
             WHERE pc.enabled = true
             GROUP BY pc.id
             ORDER BY pc.created_at DESC"#,
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
        let repo_url = first_repo_url(&params.repo_full_names);

        sqlx::query_as!(
            ProjectConfig,
            r#"INSERT INTO project_configs (id, team_id, external_project_id, name, external_workspace_id, integration_type, enabled, pickup_column, target_column,
             progress_column, max_turns, prompt_template, repo_url, agents_md, provider_id, primary_model_provider_key, primary_model_id,
             small_model_provider_key, small_model_id, review_enabled, review_pickup_column, review_max_turns, review_prompt_template, max_in_progress_tasks)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23, $24)
              RETURNING id, team_id, external_project_id, name, external_workspace_id, integration_type as "integration_type!: _", enabled, pickup_column, target_column,
              progress_column, max_turns, prompt_template, repo_url, ARRAY[]::TEXT[] as "repo_full_names!", ARRAY[]::TEXT[] as "repo_urls!", agents_md, primary_model_provider_key, primary_model_id,
              small_model_provider_key, small_model_id, review_enabled, review_pickup_column, review_max_turns, review_prompt_template, max_in_progress_tasks,
              created_at, provider_id as "provider_id?""#,
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
            params.max_turns,
            params.prompt_template.as_deref(),
            repo_url,
            params.agents_md.as_deref(),
            params.provider_id,
            params.primary_model_provider_key.as_deref(),
            params.primary_model_id.as_deref(),
            params.small_model_provider_key.as_deref(),
            params.small_model_id.as_deref(),
            params.review_enabled,
            params.review_pickup_column.as_deref(),
            params.review_max_turns,
            params.review_prompt_template.as_deref(),
            params.max_in_progress_tasks,
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
             max_turns = COALESCE($6, max_turns),
             prompt_template = CASE WHEN $7 THEN $8 ELSE prompt_template END,
             repo_url = COALESCE($9, repo_url),
             agents_md = CASE WHEN $10 THEN $11 ELSE agents_md END,
             enabled = COALESCE($12, enabled),
             external_workspace_id = COALESCE($13, external_workspace_id),
             integration_type = COALESCE($14, integration_type),
             provider_id = COALESCE($15, provider_id),
             primary_model_provider_key = CASE WHEN $16 THEN $17 ELSE primary_model_provider_key END,
             primary_model_id = CASE WHEN $18 THEN $19 ELSE primary_model_id END,
              small_model_provider_key = CASE WHEN $20 THEN $21 ELSE small_model_provider_key END,
              small_model_id = CASE WHEN $22 THEN $23 ELSE small_model_id END,
              review_enabled = CASE WHEN $24 THEN $25 ELSE review_enabled END,
              review_pickup_column = CASE WHEN $26 THEN $27 ELSE review_pickup_column END,
              review_max_turns = CASE WHEN $28 THEN $29 ELSE review_max_turns END,
              review_prompt_template = CASE WHEN $30 THEN $31 ELSE review_prompt_template END,
              max_in_progress_tasks = CASE WHEN $32 THEN $33 ELSE max_in_progress_tasks END
               WHERE id = $1
               RETURNING id, team_id, external_project_id, name, external_workspace_id, integration_type as "integration_type!: _", enabled, pickup_column, target_column,
               progress_column, max_turns, prompt_template, repo_url, ARRAY[]::TEXT[] as "repo_full_names!", ARRAY[]::TEXT[] as "repo_urls!", agents_md, primary_model_provider_key, primary_model_id,
              small_model_provider_key, small_model_id, review_enabled, review_pickup_column, review_max_turns, review_prompt_template, max_in_progress_tasks,
              created_at, provider_id as "provider_id?""#,
            id,
            params.name,
            params.pickup_column,
            params.target_column,
            params.progress_column,
            params.max_turns,
            params.prompt_template.is_some(),
            params.prompt_template.flatten(),
            params.repo_url,
            params.agents_md.is_some(),
            params.agents_md.flatten(),
            params.enabled,
            params.external_workspace_id,
            params.integration_type as _,
            params.provider_id,
            params.primary_model_provider_key.is_some(),
            params.primary_model_provider_key.flatten(),
            params.primary_model_id.is_some(),
            params.primary_model_id.flatten(),
            params.small_model_provider_key.is_some(),
            params.small_model_provider_key.flatten(),
            params.small_model_id.is_some(),
            params.small_model_id.flatten(),
            params.review_enabled.is_some(),
            params.review_enabled.flatten(),
            params.review_pickup_column.is_some(),
            params.review_pickup_column.flatten(),
            params.review_max_turns.is_some(),
            params.review_max_turns.flatten(),
            params.review_prompt_template.is_some(),
            params.review_prompt_template.flatten(),
            params.max_in_progress_tasks.is_some(),
            params.max_in_progress_tasks.flatten(),
        )
        .fetch_optional(db)
        .await?
        .ok_or(ProjectConfigsError::NotFound)
    }

    pub async fn replace_repos(
        &self,
        db: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        project_config_id: Uuid,
        repo_full_names: &[String],
    ) -> Result<(), ProjectConfigsError> {
        sqlx::query!(
            "DELETE FROM project_config_repos WHERE project_config_id = $1",
            project_config_id,
        )
        .execute(&mut **db)
        .await
        .map_err(ProjectConfigsError::from)?;

        sqlx::query!(
            r#"WITH repos AS (
                SELECT repo_full_name, MIN(ordinality) AS first_position
                FROM UNNEST($2::TEXT[]) WITH ORDINALITY AS repo(repo_full_name, ordinality)
                WHERE repo_full_name <> ''
                GROUP BY repo_full_name
            ), ordered_repos AS (
                SELECT repo_full_name, ROW_NUMBER() OVER (ORDER BY first_position) - 1 AS position
                FROM repos
            )
            INSERT INTO project_config_repos (project_config_id, repo_full_name, repo_url, position)
            SELECT $1, repo_full_name, $3::TEXT || repo_full_name, position::INT
            FROM ordered_repos"#,
            project_config_id,
            repo_full_names,
            GITHUB_REPO_URL_PREFIX,
        )
        .execute(&mut **db)
        .await
        .map_err(ProjectConfigsError::from)?;

        Ok(())
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
}

fn first_repo_url(repo_full_names: &[String]) -> String {
    repo_full_names
        .first()
        .map(|full_name| github_repo_url(full_name))
        .unwrap_or_default()
}
