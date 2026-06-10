use crate::queryer::Queryer;
use crate::services::github_app::errors::GithubAppError;
use crate::services::github_app::model::GithubInstallation;

use uuid::Uuid;

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
        team_id: Uuid,
    ) -> Result<Option<GithubInstallation>, GithubAppError> {
        let row = sqlx::query_as!(
            GithubInstallation,
            r#"SELECT id, team_id, github_installation_id, account_login, installed_by_user_id, created_at as "created_at!: chrono::DateTime<chrono::Utc>"
             FROM github_installations WHERE team_id = $1 ORDER BY created_at DESC LIMIT 1"#,
            team_id,
        )
        .fetch_optional(db)
        .await
        .map_err(GithubAppError::Database)?;

        Ok(row)
    }

    pub async fn insert_installation<'c, Q: Queryer<'c>>(
        &self,
        db: Q,
        team_id: Uuid,
        installed_by_user_id: Option<&str>,
        github_installation_id: i64,
        account_login: &str,
    ) -> Result<GithubInstallation, GithubAppError> {
        let row = sqlx::query_as!(
            GithubInstallation,
            r#"INSERT INTO github_installations (team_id, installed_by_user_id, github_installation_id, account_login)
               VALUES ($1, $2, $3, $4)
               ON CONFLICT (github_installation_id) DO UPDATE SET
                   team_id = EXCLUDED.team_id,
                   installed_by_user_id = EXCLUDED.installed_by_user_id,
                   github_installation_id = EXCLUDED.github_installation_id,
                   account_login = EXCLUDED.account_login,
                   created_at = NOW()
               RETURNING id, team_id, github_installation_id, account_login, installed_by_user_id, created_at as "created_at!: chrono::DateTime<chrono::Utc>""#,
            team_id,
            installed_by_user_id,
            github_installation_id,
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
        team_id: Uuid,
    ) -> Result<(), GithubAppError> {
        let rows = sqlx::query!(
            "DELETE FROM github_installations WHERE id = $1 AND team_id = $2",
            id,
            team_id,
        )
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
