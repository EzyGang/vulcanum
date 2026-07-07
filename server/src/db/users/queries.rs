use crate::db::queryer::Queryer;
use crate::db::users::UsersRepository;
use crate::models::users::errors::UsersError;
use crate::models::users::model::User;

impl UsersRepository {
    pub async fn find_or_create_user(
        &self,
        pool: &sqlx::PgPool,
        email: &str,
    ) -> Result<User, UsersError> {
        let id = uuid::Uuid::new_v4().to_string();
        let user = sqlx::query_as!(
            User,
            r#"INSERT INTO users (id, email)
               VALUES ($1, $2)
               ON CONFLICT (email) DO UPDATE SET email = EXCLUDED.email
               RETURNING id, email,
                created_at as "created_at!: chrono::DateTime<chrono::Utc>",
                last_login_at as "last_login_at?: chrono::DateTime<chrono::Utc>""#,
            &id,
            email,
        )
        .fetch_one(pool)
        .await?;
        Ok(user)
    }

    pub async fn find_user_by_id<'c, Q: Queryer<'c>>(
        &self,
        db: Q,
        user_id: &str,
    ) -> Result<User, UsersError> {
        sqlx::query_as!(
            User,
            r#"SELECT id, email,
               created_at as "created_at!: chrono::DateTime<chrono::Utc>",
               last_login_at as "last_login_at?: chrono::DateTime<chrono::Utc>"
               FROM users WHERE id = $1"#,
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
        sqlx::query!(
            "UPDATE users SET last_login_at = $1 WHERE id = $2",
            chrono::Utc::now(),
            user_id,
        )
        .execute(db)
        .await?;
        Ok(())
    }
}
