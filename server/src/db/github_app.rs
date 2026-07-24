use crate::db::queryer::Queryer;
use crate::models::github_app::errors::GithubAppError;
use crate::models::github_app::model::GithubInstallation;

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
            r#"SELECT id, team_id, github_installation_id, account_login, installed_by_user_id,
                      review_identity_user_id, review_identity_login,
                      created_at as "created_at!: chrono::DateTime<chrono::Utc>"
               FROM github_installations WHERE team_id = $1 ORDER BY created_at DESC LIMIT 1"#,
            team_id,
        )
        .fetch_optional(db)
        .await
        .map_err(GithubAppError::Database)?;

        Ok(row)
    }
    pub async fn find_team_id_by_github_installation<'c, Q>(
        &self,
        db: Q,
        github_installation_id: i64,
    ) -> Result<Option<Uuid>, GithubAppError>
    where
        Q: Queryer<'c>,
    {
        sqlx::query_scalar!(
            "SELECT team_id FROM github_installations WHERE github_installation_id = $1",
            github_installation_id,
        )
        .fetch_optional(db)
        .await
        .map_err(GithubAppError::Database)
    }

    pub async fn insert_installation<'c, Q: Queryer<'c>>(
        &self,
        db: Q,
        team_id: Uuid,
        installed_by_user_id: Option<&str>,
        github_installation_id: i64,
        account_login: &str,
    ) -> Result<GithubInstallation, GithubAppError> {
        sqlx::query_as!(
            GithubInstallation,
            r#"INSERT INTO github_installations (team_id, installed_by_user_id, github_installation_id, account_login)
               VALUES ($1, $2, $3, $4)
               ON CONFLICT (github_installation_id) DO UPDATE SET
                   installed_by_user_id = EXCLUDED.installed_by_user_id,
                   account_login = EXCLUDED.account_login,
                   created_at = NOW()
               WHERE github_installations.team_id = EXCLUDED.team_id
               RETURNING id, team_id, github_installation_id, account_login, installed_by_user_id,
                         review_identity_user_id, review_identity_login,
                         created_at as "created_at!: chrono::DateTime<chrono::Utc>""#,
            team_id,
            installed_by_user_id,
            github_installation_id,
            account_login,
        )
        .fetch_optional(db)
        .await
        .map_err(GithubAppError::Database)?
        .ok_or(GithubAppError::InstallationAlreadyLinked)
    }

    pub async fn link_review_identity<'c, Q>(
        &self,
        db: Q,
        team_id: Uuid,
        user_id: &str,
        login: &str,
    ) -> Result<GithubInstallation, GithubAppError>
    where
        Q: Queryer<'c>,
    {
        sqlx::query_as!(
            GithubInstallation,
            r#"UPDATE github_installations
               SET review_identity_user_id = $2, review_identity_login = $3
               WHERE id = (
                   SELECT id FROM github_installations
                   WHERE team_id = $1
                   ORDER BY created_at DESC
                   LIMIT 1
               )
               RETURNING id, team_id, github_installation_id, account_login, installed_by_user_id,
                         review_identity_user_id, review_identity_login,
                         created_at as "created_at!: chrono::DateTime<chrono::Utc>""#,
            team_id,
            user_id,
            login,
        )
        .fetch_optional(db)
        .await
        .map_err(GithubAppError::Database)?
        .ok_or(GithubAppError::NoInstallation)
    }

    pub async fn is_linked_review_identity<'c, Q>(
        &self,
        db: Q,
        github_installation_id: i64,
        user_id: &str,
    ) -> Result<bool, GithubAppError>
    where
        Q: Queryer<'c>,
    {
        sqlx::query_scalar!(
            r#"SELECT EXISTS(
                SELECT 1 FROM github_installations
                WHERE github_installation_id = $1
                  AND review_identity_user_id = $2
            )"#,
            github_installation_id,
            user_id,
        )
        .fetch_one(db)
        .await
        .map(|value| value.unwrap_or(false))
        .map_err(GithubAppError::Database)
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

#[cfg(test)]
mod repository_tests;
