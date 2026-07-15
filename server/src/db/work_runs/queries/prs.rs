use uuid::Uuid;

use crate::db::queryer::Queryer;
use crate::db::work_runs::WorkRunsRepository;
use crate::models::work_runs::errors::WorkRunsError;
use crate::models::work_runs::model::{TaskPr, TaskPrTarget};

pub struct InsertReviewResultParams<'a> {
    pub work_run_id: Uuid,
    pub pr_url: &'a str,
    pub repo_full_name: &'a str,
    pub review_url: Option<&'a str>,
    pub review_body: Option<&'a str>,
    pub review_already_exists: bool,
}

pub struct UpsertTaskPrParams<'a> {
    pub project_config_id: Uuid,
    pub external_task_ref: &'a str,
    pub pr_url: &'a str,
    pub repo_full_name: &'a str,
    pub pr_number: i64,
    pub source_work_run_id: Uuid,
}

impl WorkRunsRepository {
    pub async fn list_pr_urls<'c, Q>(
        &self,
        db: Q,
        work_run_id: Uuid,
    ) -> Result<Vec<String>, WorkRunsError>
    where
        Q: Queryer<'c>,
    {
        let rows = sqlx::query!(
            r#"SELECT pr_url FROM work_run_prs
             WHERE work_run_id = $1 ORDER BY position ASC"#,
            work_run_id,
        )
        .fetch_all(db)
        .await
        .map_err(WorkRunsError::from)?;

        Ok(rows.into_iter().map(|row| row.pr_url).collect())
    }

    pub async fn list_task_prs_for_refs<'c, Q>(
        &self,
        db: Q,
        project_config_id: Uuid,
        external_task_refs: &[String],
    ) -> Result<Vec<TaskPr>, WorkRunsError>
    where
        Q: Queryer<'c>,
    {
        if external_task_refs.is_empty() {
            return Ok(Vec::new());
        }

        sqlx::query_as!(
            TaskPr,
            r#"SELECT id, project_config_id, external_task_ref, pr_url, repo_full_name, pr_number,
             source_work_run_id, created_at as "created_at!: chrono::DateTime<chrono::Utc>",
             updated_at as "updated_at!: chrono::DateTime<chrono::Utc>"
             FROM task_prs
             WHERE project_config_id = $1 AND external_task_ref = ANY($2)
             ORDER BY external_task_ref, created_at"#,
            project_config_id,
            external_task_refs,
        )
        .fetch_all(db)
        .await
        .map_err(WorkRunsError::from)
    }

    pub async fn list_task_pr_targets_for_pull_request<'c, Q>(
        &self,
        db: Q,
        installation_id: i64,
        repo_full_name: &str,
        pr_number: i64,
    ) -> Result<Vec<TaskPrTarget>, WorkRunsError>
    where
        Q: Queryer<'c>,
    {
        sqlx::query_as!(
            TaskPrTarget,
            r#"SELECT DISTINCT tp.project_config_id, tp.external_task_ref
             FROM task_prs tp
             INNER JOIN project_configs pc ON pc.id = tp.project_config_id
             INNER JOIN github_installations gi ON gi.team_id = pc.team_id
             WHERE gi.github_installation_id = $1
               AND LOWER(tp.repo_full_name) = LOWER($2)
               AND tp.pr_number = $3
             ORDER BY tp.project_config_id, tp.external_task_ref"#,
            installation_id,
            repo_full_name,
            pr_number,
        )
        .fetch_all(db)
        .await
        .map_err(WorkRunsError::from)
    }

    pub async fn upsert_task_pr<'c, Q>(
        &self,
        db: Q,
        params: UpsertTaskPrParams<'_>,
    ) -> Result<TaskPr, WorkRunsError>
    where
        Q: Queryer<'c>,
    {
        sqlx::query_as!(
            TaskPr,
            r#"INSERT INTO task_prs (project_config_id, external_task_ref, pr_url, repo_full_name, pr_number, source_work_run_id)
             VALUES ($1, $2, $3, $4, $5, $6)
             ON CONFLICT (project_config_id, external_task_ref, pr_url) DO UPDATE SET
                 repo_full_name = EXCLUDED.repo_full_name,
                 pr_number = EXCLUDED.pr_number,
                 source_work_run_id = EXCLUDED.source_work_run_id
             RETURNING id, project_config_id, external_task_ref, pr_url, repo_full_name, pr_number,
              source_work_run_id, created_at as "created_at!: chrono::DateTime<chrono::Utc>", updated_at as "updated_at!: chrono::DateTime<chrono::Utc>""#,
            params.project_config_id,
            params.external_task_ref,
            params.pr_url,
            params.repo_full_name,
            params.pr_number,
            params.source_work_run_id,
        )
        .fetch_one(db)
        .await
        .map_err(WorkRunsError::from)
    }

    pub async fn insert_review_result<'c, Q>(
        &self,
        db: Q,
        params: InsertReviewResultParams<'_>,
    ) -> Result<(), WorkRunsError>
    where
        Q: Queryer<'c>,
    {
        sqlx::query!(
            r#"INSERT INTO work_run_reviews (work_run_id, pr_url, repo_full_name, review_url, review_body, review_already_exists)
             VALUES ($1, $2, $3, $4, $5, $6)
             ON CONFLICT (work_run_id, pr_url) DO UPDATE SET
                 review_url = EXCLUDED.review_url,
                 review_body = EXCLUDED.review_body,
                 review_already_exists = EXCLUDED.review_already_exists"#,
            params.work_run_id,
            params.pr_url,
            params.repo_full_name,
            params.review_url,
            params.review_body,
            params.review_already_exists,
        )
        .execute(db)
        .await
        .map_err(WorkRunsError::from)?;

        Ok(())
    }
}
