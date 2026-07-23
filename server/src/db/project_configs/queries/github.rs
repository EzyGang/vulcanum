use crate::db::project_configs::ProjectConfigsRepository;
use crate::db::queryer::Queryer;
use crate::models::project_configs::errors::ProjectConfigsError;
use crate::models::project_configs::model::ProjectConfig;

impl ProjectConfigsRepository {
    pub async fn list_enabled_for_github_repo<'c, Q>(
        &self,
        db: Q,
        github_installation_id: i64,
        repo_full_name: &str,
    ) -> Result<Vec<ProjectConfig>, ProjectConfigsError>
    where
        Q: Queryer<'c>,
    {
        sqlx::query_as!(
            ProjectConfig,
            r#"SELECT pc.id, pc.team_id, pc.external_project_id, pc.name, pc.external_workspace_id,
             pc.integration_type as "integration_type!: _", pc.enabled, pc.pickup_column, pc.review_column,
             pc.done_column, pc.progress_column, pc.max_turns, pc.prompt_template, pc.repo_url,
             COALESCE(array_agg(all_repos.repo_full_name ORDER BY all_repos.position)
                 FILTER (WHERE all_repos.id IS NOT NULL), ARRAY[]::TEXT[]) as "repo_full_names!",
             COALESCE(array_agg(all_repos.repo_url ORDER BY all_repos.position)
                 FILTER (WHERE all_repos.id IS NOT NULL), ARRAY[]::TEXT[]) as "repo_urls!",
             pc.agents_md, pc.review_enabled, pc.review_max_turns, pc.review_prompt_template,
             pc.max_in_progress_tasks, pc.created_at, pc.provider_id as "provider_id?"
             FROM github_installations gi
             INNER JOIN project_configs pc ON pc.team_id = gi.team_id
             INNER JOIN project_config_repos matched_repo ON matched_repo.project_config_id = pc.id
             LEFT JOIN project_config_repos all_repos ON all_repos.project_config_id = pc.id
             WHERE gi.github_installation_id = $1
               AND pc.enabled = true
               AND LOWER(matched_repo.repo_full_name) = LOWER($2)
             GROUP BY pc.id
             ORDER BY pc.created_at ASC, pc.id ASC"#,
            github_installation_id,
            repo_full_name,
        )
        .fetch_all(db)
        .await
        .map_err(ProjectConfigsError::from)
    }
}
