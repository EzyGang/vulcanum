use sqlx::PgConnection;
use uuid::Uuid;

use crate::db::work_runs::WorkRunsRepository;
use crate::models::work_runs::errors::WorkRunsError;

impl WorkRunsRepository {
    pub async fn lock_github_review_ticket(
        &self,
        db: &mut PgConnection,
        project_config_id: Uuid,
        repo_full_name: &str,
        pr_number: i64,
    ) -> Result<(), WorkRunsError> {
        let lock_key = format!("{project_config_id}:{repo_full_name}#{pr_number}");
        sqlx::query!(
            "SELECT pg_advisory_xact_lock(hashtextextended($1, 0))",
            lock_key,
        )
        .execute(db)
        .await?;

        Ok(())
    }

    pub async fn find_github_review_ticket(
        &self,
        db: &mut PgConnection,
        project_config_id: Uuid,
        repo_full_name: &str,
        pr_number: i64,
    ) -> Result<Option<String>, WorkRunsError> {
        let external_task_ref = sqlx::query_scalar!(
            r#"SELECT external_task_ref
               FROM github_review_tickets
               WHERE project_config_id = $1 AND repo_full_name = $2 AND pr_number = $3"#,
            project_config_id,
            repo_full_name,
            pr_number,
        )
        .fetch_optional(db)
        .await?;

        Ok(external_task_ref)
    }

    pub async fn insert_github_review_ticket(
        &self,
        db: &mut PgConnection,
        project_config_id: Uuid,
        repo_full_name: &str,
        pr_number: i64,
        external_task_ref: &str,
    ) -> Result<(), WorkRunsError> {
        sqlx::query!(
            r#"INSERT INTO github_review_tickets
               (project_config_id, repo_full_name, pr_number, external_task_ref)
               VALUES ($1, $2, $3, $4)"#,
            project_config_id,
            repo_full_name,
            pr_number,
            external_task_ref,
        )
        .execute(db)
        .await?;

        Ok(())
    }
}
