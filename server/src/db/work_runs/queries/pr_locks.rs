use sqlx::PgConnection;
use uuid::Uuid;

use crate::db::work_runs::WorkRunsRepository;
use crate::models::work_runs::errors::WorkRunsError;

impl WorkRunsRepository {
    pub async fn lock_task_pr_target(
        &self,
        db: &mut PgConnection,
        project_config_id: Uuid,
        repo_full_name: &str,
        pr_number: i64,
    ) -> Result<(), WorkRunsError> {
        sqlx::query(
            r#"SELECT pg_advisory_xact_lock(hashtextextended(
                   'github-task-pr:' || $1::TEXT || ':' || LOWER($2) || '#' || $3::TEXT,
                   0
               ))"#,
        )
        .bind(project_config_id)
        .bind(repo_full_name)
        .bind(pr_number)
        .execute(db)
        .await?;

        Ok(())
    }

    pub async fn lock_task_pr_completion(
        &self,
        db: &mut PgConnection,
        project_config_id: Uuid,
        task_ref: &str,
    ) -> Result<(), WorkRunsError> {
        sqlx::query(
            r#"SELECT pg_advisory_xact_lock(hashtextextended(
                   'github-task-completion:' || $1::TEXT || ':' || $2,
                   0
               ))"#,
        )
        .bind(project_config_id)
        .bind(task_ref)
        .execute(db)
        .await?;

        Ok(())
    }
}
