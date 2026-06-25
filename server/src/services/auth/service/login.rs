use crate::models::auth::errors::AuthError;
use crate::models::auth::model::LoginRequest;
use crate::models::workers::model::ACCESS_TOKEN_TTL_MINUTES;
use crate::services::auth::service::AuthService;

impl AuthService {
    pub async fn login(&self, req: LoginRequest) -> Result<String, AuthError> {
        let user = self.users.find_or_create_user(&req.email).await?;

        let token = vulcanum_shared::crypto::generate_alphanumeric_string(32);
        self.token_store
            .insert(&token, &user.id, ACCESS_TOKEN_TTL_MINUTES);

        log::info!(
            "Magic link for {}: http://localhost:8080/api/v1/auth/verify?token={}",
            user.email,
            token
        );

        Ok(token)
    }
}
