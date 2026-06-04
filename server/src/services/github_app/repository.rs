use crate::queryer::Queryer;
use crate::services::github_app::errors::GithubAppError;
use crate::services::github_app::model::GithubInstallation;
use chrono::{DateTime, Utc};

#[derive(Clone)]
pub struct GithubAppRepository;

impl Default for GithubAppRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl GithubAppRepository {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn get_installation<'c, Q: Queryer<'c>>(
        &self,
        db: Q,
    ) -> Result<Option<GithubInstallation>, GithubAppError> {
        let row = sqlx::query_as!(
            GithubInstallation,
            r#"SELECT id, account_login, created_at as "created_at!: DateTime<Utc>" FROM github_installations LIMIT 1"#
        )
        .fetch_optional(db)
        .await
        .map_err(GithubAppError::Database)?;

        Ok(row)
    }

    pub async fn insert_installation<'c, Q: Queryer<'c>>(
        &self,
        db: Q,
        account_login: &str,
    ) -> Result<GithubInstallation, GithubAppError> {
        let row = sqlx::query_as!(
            GithubInstallation,
            r#"INSERT INTO github_installations (account_login) VALUES ($1) RETURNING id, account_login, created_at as "created_at!: DateTime<Utc>""#,
            account_login,
        )
        .fetch_one(db)
        .await
        .map_err(GithubAppError::Database)?;

        Ok(row)
    }

    pub async fn delete_installation<'c, Q: Queryer<'c>>(
        &self,
        db: Q,
        id: i64,
    ) -> Result<(), GithubAppError> {
        let rows = sqlx::query!("DELETE FROM github_installations WHERE id = $1", id)
            .execute(db)
            .await
            .map_err(GithubAppError::Database)?
            .rows_affected();

        if rows == 0 {
            return Err(GithubAppError::NoInstallation);
        }

        Ok(())
    }
}
