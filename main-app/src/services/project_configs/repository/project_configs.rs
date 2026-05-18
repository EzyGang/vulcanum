use uuid::Uuid;

use crate::services::project_configs::errors::ProjectConfigsError;
use crate::services::project_configs::model::{CreateProjectConfigRequest, ProjectConfig};
use crate::services::project_configs::repository::{map_sqlx_error, ProjectConfigsRepository, Queryer};

const BASE_COLUMNS: &str = "id, kaneo_project_id, enabled, pickup_column, target_column, \
     progress_column, prompt_template, repo_url, created_at";

impl ProjectConfigsRepository {
    pub async fn list_all<'c, Q: Queryer<'c>>(
        &self,
        db: Q,
    ) -> Result<Vec<ProjectConfig>, ProjectConfigsError> {
        sqlx::query_as::<_, ProjectConfig>(&format!(
            "SELECT {} FROM project_configs ORDER BY created_at DESC",
            BASE_COLUMNS
        ))
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
            "SELECT {} FROM project_configs WHERE id = $1",
            BASE_COLUMNS
        ))
        .bind(id)
        .fetch_optional(db)
        .await?
        .ok_or(ProjectConfigsError::NotFound)
    }

    pub async fn find_by_kaneo_project_id<'c, Q: Queryer<'c>>(
        &self,
        db: Q,
        kaneo_project_id: &str,
    ) -> Result<Option<ProjectConfig>, ProjectConfigsError> {
        sqlx::query_as::<_, ProjectConfig>(&format!(
            "SELECT {} FROM project_configs WHERE kaneo_project_id = $1",
            BASE_COLUMNS
        ))
        .bind(kaneo_project_id)
        .fetch_optional(db)
        .await
        .map_err(ProjectConfigsError::from)
    }

    pub async fn list_enabled<'c, Q: Queryer<'c>>(
        &self,
        db: Q,
    ) -> Result<Vec<ProjectConfig>, ProjectConfigsError> {
        sqlx::query_as::<_, ProjectConfig>(&format!(
            "SELECT {} FROM project_configs WHERE enabled = true ORDER BY created_at DESC",
            BASE_COLUMNS
        ))
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

        sqlx::query_as::<_, ProjectConfig>(&format!(
            "INSERT INTO project_configs (id, kaneo_project_id, pickup_column, target_column, \
             progress_column, prompt_template, repo_url) \
             VALUES ($1, $2, $3, $4, $5, $6, $7) \
             RETURNING {}",
            BASE_COLUMNS
        ))
        .bind(id)
        .bind(&params.kaneo_project_id)
        .bind(&params.pickup_column)
        .bind(&params.target_column)
        .bind(&params.progress_column)
        .bind(&params.prompt_template)
        .bind(&params.repo_url)
        .fetch_one(db)
        .await
        .map_err(map_sqlx_error)
    }

    pub async fn update<'c, Q: Queryer<'c>>(
        &self,
        db: Q,
        id: Uuid,
        pickup_column: Option<&str>,
        target_column: Option<&str>,
        progress_column: Option<&str>,
        prompt_template: Option<&str>,
        repo_url: Option<&str>,
        enabled: Option<bool>,
    ) -> Result<ProjectConfig, ProjectConfigsError> {
        sqlx::query_as::<_, ProjectConfig>(&format!(
            "UPDATE project_configs SET \
             pickup_column = COALESCE($2, pickup_column), \
             target_column = COALESCE($3, target_column), \
             progress_column = COALESCE($4, progress_column), \
             prompt_template = COALESCE($5, prompt_template), \
             repo_url = COALESCE($6, repo_url), \
             enabled = COALESCE($7, enabled) \
             WHERE id = $1 \
             RETURNING {}",
            BASE_COLUMNS
        ))
        .bind(id)
        .bind(pickup_column)
        .bind(target_column)
        .bind(progress_column)
        .bind(prompt_template)
        .bind(repo_url)
        .bind(enabled)
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
}
