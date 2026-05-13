use sqlx::PgPool;

use crate::services::users::errors::UsersError;
use crate::services::users::model::User;
use crate::services::users::repository::UsersRepository;

#[derive(Clone)]
pub struct UsersService {
    pub repo: UsersRepository,
    pub db: PgPool,
}

impl UsersService {
    pub fn new(repo: UsersRepository, db: PgPool) -> Self {
        Self { repo, db }
    }

    pub async fn find_or_create_user(&self, email: &str) -> Result<User, UsersError> {
        self.repo.find_or_create_user(&self.db, email).await
    }

    pub async fn find_user_by_id(&self, user_id: &str) -> Result<User, UsersError> {
        self.repo.find_user_by_id(&self.db, user_id).await
    }

    pub async fn update_last_login(&self, user_id: &str) -> Result<(), UsersError> {
        self.repo.update_last_login(&self.db, user_id).await
    }
}
