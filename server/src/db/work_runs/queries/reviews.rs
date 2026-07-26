use uuid::Uuid;

use crate::db::queryer::Queryer;
use crate::db::work_runs::WorkRunsRepository;
use crate::models::work_runs::errors::WorkRunsError;

pub struct InsertReviewResultParams<'a> {
    pub work_run_id: Uuid,
    pub pr_url: &'a str,
    pub repo_full_name: &'a str,
    pub review_url: Option<&'a str>,
    pub review_body: Option<&'a str>,
    pub review_already_exists: bool,
}

impl WorkRunsRepository {
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
