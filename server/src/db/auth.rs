use chrono::{DateTime, Utc};
use sqlx::Row;
use uuid::Uuid;

use crate::models::auth::errors::AuthError;

#[derive(Clone, Default)]
pub struct AuthRepository;

impl AuthRepository {
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    pub async fn create_refresh_token(
        &self,
        db: &sqlx::PgPool,
        user_id: &str,
        token_hash: &str,
        expires_at: DateTime<Utc>,
    ) -> Result<(), AuthError> {
        sqlx::query(
            r#"INSERT INTO user_refresh_tokens (id, user_id, token_hash, expires_at)
             VALUES ($1, $2, $3, $4)"#,
        )
        .bind(Uuid::new_v4())
        .bind(user_id)
        .bind(token_hash)
        .bind(expires_at)
        .execute(db)
        .await?;

        Ok(())
    }

    pub async fn create_instance_refresh_token(
        &self,
        db: &sqlx::PgPool,
        token_hash: &str,
        password_fingerprint: &str,
        expires_at: DateTime<Utc>,
    ) -> Result<(), AuthError> {
        sqlx::query(
            r#"INSERT INTO instance_refresh_tokens (
                id, token_hash, password_fingerprint, expires_at
            )
            VALUES ($1, $2, $3, $4)"#,
        )
        .bind(Uuid::new_v4())
        .bind(token_hash)
        .bind(password_fingerprint)
        .bind(expires_at)
        .execute(db)
        .await?;

        Ok(())
    }

    pub async fn revoke_mismatched_instance_refresh_tokens(
        &self,
        db: &sqlx::PgPool,
        password_fingerprint: &str,
    ) -> Result<(), AuthError> {
        sqlx::query(
            r#"UPDATE instance_refresh_tokens
            SET revoked_at = NOW()
            WHERE password_fingerprint <> $1
              AND revoked_at IS NULL"#,
        )
        .bind(password_fingerprint)
        .execute(db)
        .await?;

        Ok(())
    }

    pub async fn rotate_instance_refresh_token(
        &self,
        db: &sqlx::PgPool,
        token_hash: &str,
        new_token_hash: &str,
        password_fingerprint: &str,
        new_expires_at: DateTime<Utc>,
    ) -> Result<(), AuthError> {
        let now = Utc::now();
        let result = sqlx::query(
            r#"UPDATE instance_refresh_tokens
            SET token_hash = $1,
                expires_at = $2,
                last_used_at = $3
            WHERE token_hash = $4
              AND password_fingerprint = $5
              AND revoked_at IS NULL
              AND expires_at > $3"#,
        )
        .bind(new_token_hash)
        .bind(new_expires_at)
        .bind(now)
        .bind(token_hash)
        .bind(password_fingerprint)
        .execute(db)
        .await?;

        if result.rows_affected() != 1 {
            return Err(AuthError::InvalidRefreshToken);
        }

        Ok(())
    }

    pub async fn revoke_instance_refresh_token(
        &self,
        db: &sqlx::PgPool,
        token_hash: &str,
    ) -> Result<(), AuthError> {
        sqlx::query(
            r#"UPDATE instance_refresh_tokens
            SET revoked_at = NOW()
            WHERE token_hash = $1
              AND revoked_at IS NULL"#,
        )
        .bind(token_hash)
        .execute(db)
        .await?;

        Ok(())
    }

    pub async fn rotate_refresh_token(
        &self,
        db: &sqlx::PgPool,
        token_hash: &str,
        new_token_hash: &str,
        new_expires_at: DateTime<Utc>,
    ) -> Result<String, AuthError> {
        let now = Utc::now();
        let row = sqlx::query(
            r#"UPDATE user_refresh_tokens
             SET token_hash = $1,
                 expires_at = $2,
                 last_used_at = $3
             WHERE token_hash = $4
               AND revoked_at IS NULL
               AND expires_at > $3
             RETURNING user_id"#,
        )
        .bind(new_token_hash)
        .bind(new_expires_at)
        .bind(now)
        .bind(token_hash)
        .fetch_optional(db)
        .await?
        .ok_or(AuthError::InvalidRefreshToken)?;

        row.try_get("user_id").map_err(AuthError::from)
    }

    pub async fn revoke_refresh_token(
        &self,
        db: &sqlx::PgPool,
        token_hash: &str,
    ) -> Result<(), AuthError> {
        sqlx::query(
            r#"UPDATE user_refresh_tokens
             SET revoked_at = NOW()
             WHERE token_hash = $1
               AND revoked_at IS NULL"#,
        )
        .bind(token_hash)
        .execute(db)
        .await?;

        Ok(())
    }
}
