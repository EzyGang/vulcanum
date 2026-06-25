use crate::models::auth::errors::AuthError;
use crate::models::auth::model::VerifyQuery;
use crate::models::users::model::User;
use crate::services::auth::service::AuthService;

impl AuthService {
    pub async fn verify(&self, query: VerifyQuery) -> Result<User, AuthError> {
        let user_id = self
            .token_store
            .consume(&query.token)
            .ok_or(AuthError::InvalidToken)?;

        let user = self.users.find_user_by_id(&user_id).await?;
        self.users.update_last_login(&user.id).await?;

        Ok(user)
    }
}
