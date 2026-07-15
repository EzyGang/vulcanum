use uuid::Uuid;

use crate::db::project_configs::{
    map_sqlx_error, ProjectConfigsRepository, UpdateProjectConfigParams,
};
use crate::db::queryer::Queryer;
use crate::models::project_configs::errors::ProjectConfigsError;
use crate::models::project_configs::model::{CreateProjectConfigRequest, ProjectConfig};
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
            r#"SELECT pc.id, pc.team_id, pc.external_project_id, pc.name, pc.external_workspace_id, pc.integration_type as "integration_type!: _", pc.enabled, pc.pickup_column, pc.review_column,
             pc.done_column, pc.progress_column, pc.max_turns, pc.prompt_template, pc.repo_url,
             COALESCE(array_agg(pcr.repo_full_name ORDER BY pcr.position) FILTER (WHERE pcr.id IS NOT NULL), ARRAY[]::TEXT[]) as "repo_full_names!",
             COALESCE(array_agg(pcr.repo_url ORDER BY pcr.position) FILTER (WHERE pcr.id IS NOT NULL), ARRAY[]::TEXT[]) as "repo_urls!",
             pc.agents_md, pc.review_enabled, pc.review_max_turns, pc.review_prompt_template, pc.max_in_progress_tasks,
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
            r#"SELECT pc.id, pc.team_id, pc.external_project_id, pc.name, pc.external_workspace_id, pc.integration_type as "integration_type!: _", pc.enabled, pc.pickup_column, pc.review_column,
             pc.done_column, pc.progress_column, pc.max_turns, pc.prompt_template, pc.repo_url,
             COALESCE(array_agg(pcr.repo_full_name ORDER BY pcr.position) FILTER (WHERE pcr.id IS NOT NULL), ARRAY[]::TEXT[]) as "repo_full_names!",
             COALESCE(array_agg(pcr.repo_url ORDER BY pcr.position) FILTER (WHERE pcr.id IS NOT NULL), ARRAY[]::TEXT[]) as "repo_urls!",
              pc.agents_md, pc.review_enabled, pc.review_max_turns, pc.review_prompt_template, pc.max_in_progress_tasks,
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

    pub async fn find_by_provider_project<'c, Q>(
        &self,
        db: Q,
        team_id: Uuid,
        provider_id: Uuid,
        external_project_id: &str,
    ) -> Result<Option<ProjectConfig>, ProjectConfigsError>
    where
        Q: Queryer<'c>,
    {
        sqlx::query_as!(
            ProjectConfig,
            r#"SELECT pc.id, pc.team_id, pc.external_project_id, pc.name, pc.external_workspace_id, pc.integration_type as "integration_type!: _", pc.enabled, pc.pickup_column, pc.review_column,
             pc.done_column, pc.progress_column, pc.max_turns, pc.prompt_template, pc.repo_url,
             COALESCE(array_agg(pcr.repo_full_name ORDER BY pcr.position) FILTER (WHERE pcr.id IS NOT NULL), ARRAY[]::TEXT[]) as "repo_full_names!",
             COALESCE(array_agg(pcr.repo_url ORDER BY pcr.position) FILTER (WHERE pcr.id IS NOT NULL), ARRAY[]::TEXT[]) as "repo_urls!",
              pc.agents_md, pc.review_enabled, pc.review_max_turns, pc.review_prompt_template, pc.max_in_progress_tasks,
              pc.created_at, pc.provider_id as "provider_id?"
             FROM project_configs pc LEFT JOIN project_config_repos pcr ON pcr.project_config_id = pc.id
             WHERE pc.team_id = $1 AND pc.provider_id = $2 AND pc.external_project_id = $3
             GROUP BY pc.id"#,
            team_id,
            provider_id,
            external_project_id,
        )
        .fetch_optional(db)
        .await
        .map_err(ProjectConfigsError::from)
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
            r#"SELECT pc.id, pc.team_id, pc.external_project_id, pc.name, pc.external_workspace_id, pc.integration_type as "integration_type!: _", pc.enabled, pc.pickup_column, pc.review_column,
             pc.done_column, pc.progress_column, pc.max_turns, pc.prompt_template, pc.repo_url,
             COALESCE(array_agg(pcr.repo_full_name ORDER BY pcr.position) FILTER (WHERE pcr.id IS NOT NULL), ARRAY[]::TEXT[]) as "repo_full_names!",
             COALESCE(array_agg(pcr.repo_url ORDER BY pcr.position) FILTER (WHERE pcr.id IS NOT NULL), ARRAY[]::TEXT[]) as "repo_urls!",
              pc.agents_md, pc.review_enabled, pc.review_max_turns, pc.review_prompt_template, pc.max_in_progress_tasks,
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
            r#"INSERT INTO project_configs (id, team_id, external_project_id, name, external_workspace_id, integration_type, enabled, pickup_column, review_column,
             done_column, progress_column, max_turns, prompt_template, repo_url, agents_md, provider_id, review_enabled, review_max_turns, review_prompt_template, max_in_progress_tasks)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20)
              RETURNING id, team_id, external_project_id, name, external_workspace_id, integration_type as "integration_type!: _", enabled, pickup_column, review_column,
              done_column, progress_column, max_turns, prompt_template, repo_url, ARRAY[]::TEXT[] as "repo_full_names!", ARRAY[]::TEXT[] as "repo_urls!", agents_md,
              review_enabled, review_max_turns, review_prompt_template, max_in_progress_tasks,
              created_at, provider_id as "provider_id?""#,
            id,
            team_id,
            params.external_project_id,
            params.name,
            params.external_workspace_id,
            params.integration_type as _,
            params.enabled,
            params.pickup_column,
            params.review_column,
            params.done_column,
            params.progress_column,
            params.max_turns,
            params.prompt_template.as_deref(),
            repo_url,
            params.agents_md.as_deref(),
            params.provider_id,
            params.review_enabled,
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
             review_column = COALESCE($4, review_column),
             done_column = COALESCE($5, done_column),
             progress_column = COALESCE($6, progress_column),
             max_turns = COALESCE($7, max_turns),
             prompt_template = CASE WHEN $8 THEN $9 ELSE prompt_template END,
             repo_url = COALESCE($10, repo_url),
             agents_md = CASE WHEN $11 THEN $12 ELSE agents_md END,
             enabled = COALESCE($13, enabled),
             external_workspace_id = COALESCE($14, external_workspace_id),
             integration_type = COALESCE($15, integration_type),
             provider_id = COALESCE($16, provider_id),
             review_enabled = CASE WHEN $17 THEN $18 ELSE review_enabled END,
             review_max_turns = CASE WHEN $19 THEN $20 ELSE review_max_turns END,
             review_prompt_template = CASE WHEN $21 THEN $22 ELSE review_prompt_template END,
             max_in_progress_tasks = CASE WHEN $23 THEN $24 ELSE max_in_progress_tasks END
              WHERE id = $1
              RETURNING id, team_id, external_project_id, name, external_workspace_id, integration_type as "integration_type!: _", enabled, pickup_column, review_column,
              done_column, progress_column, max_turns, prompt_template, repo_url, ARRAY[]::TEXT[] as "repo_full_names!", ARRAY[]::TEXT[] as "repo_urls!", agents_md,
              review_enabled, review_max_turns, review_prompt_template, max_in_progress_tasks,
              created_at, provider_id as "provider_id?""#,
            id,
            params.name,
            params.pickup_column,
            params.review_column,
            params.done_column,
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
            params.review_enabled.is_some(),
            params.review_enabled.flatten(),
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
