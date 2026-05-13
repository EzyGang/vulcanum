use sqlx::PgPool;

use crate::errors::AppError;
use crate::models::user::User;

pub async fn find_or_create_user(pool: &PgPool, email: &str) -> Result<User, AppError> {
    let existing = sqlx::query_as::<_, User>(
        "SELECT id, email, created_at, last_login_at FROM users WHERE email = $1",
    )
    .bind(email)
    .fetch_optional(pool)
    .await?;

    match existing {
        Some(user) => Ok(user),
        None => {
            let id = uuid::Uuid::new_v4().to_string();
            sqlx::query("INSERT INTO users (id, email) VALUES ($1, $2)")
                .bind(&id)
                .bind(email)
                .execute(pool)
                .await?;

            let user = sqlx::query_as::<_, User>(
                "SELECT id, email, created_at, last_login_at FROM users WHERE id = $1",
            )
            .bind(&id)
            .fetch_one(pool)
            .await?;

            Ok(user)
        }
    }
}

pub async fn find_user_by_id(pool: &PgPool, user_id: &str) -> Result<User, AppError> {
    sqlx::query_as::<_, User>(
        "SELECT id, email, created_at, last_login_at FROM users WHERE id = $1",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::UserNotFound)
}

pub async fn update_last_login(pool: &PgPool, user_id: &str) -> Result<(), AppError> {
    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query("UPDATE users SET last_login_at = $1 WHERE id = $2")
        .bind(&now)
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(())
}
