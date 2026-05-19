use crate::queryer::Queryer;
use crate::services::users::errors::UsersError;
use crate::services::users::model::User;
use crate::services::users::repository::UsersRepository;

impl UsersRepository {
    pub async fn find_or_create_user(
        &self,
        pool: &sqlx::PgPool,
        email: &str,
    ) -> Result<User, UsersError> {
        let mut tx = pool.begin().await?;

        let existing = sqlx::query_as!(
            User,
            "SELECT id, email, created_at, last_login_at FROM users WHERE email = $1",
            email,
        )
        .fetch_optional(&mut *tx)
        .await?;

        let user = match existing {
            Some(user) => user,
            None => {
                let id = uuid::Uuid::new_v4().to_string();
                sqlx::query!("INSERT INTO users (id, email) VALUES ($1, $2)", &id, email)
                    .execute(&mut *tx)
                    .await?;

                sqlx::query_as!(
                    User,
                    "SELECT id, email, created_at, last_login_at FROM users WHERE id = $1",
                    &id,
                )
                .fetch_one(&mut *tx)
                .await?
            }
        };

        tx.commit().await?;
        Ok(user)
    }

    pub async fn find_user_by_id<'c, Q: Queryer<'c>>(
        &self,
        db: Q,
        user_id: &str,
    ) -> Result<User, UsersError> {
        sqlx::query_as!(
            User,
            "SELECT id, email, created_at, last_login_at FROM users WHERE id = $1",
            user_id,
        )
        .fetch_optional(db)
        .await?
        .ok_or(UsersError::UserNotFound)
    }

    pub async fn update_last_login<'c, Q: Queryer<'c>>(
        &self,
        db: Q,
        user_id: &str,
    ) -> Result<(), UsersError> {
        let now = chrono::Utc::now().to_rfc3339();
        sqlx::query!(
            "UPDATE users SET last_login_at = $1 WHERE id = $2",
            &now,
            user_id,
        )
        .execute(db)
        .await?;
        Ok(())
    }
}
