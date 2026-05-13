use rand::Rng;

use crate::services::auth::errors::AuthError;
use crate::services::auth::model::LoginRequest;
use crate::services::auth::service::AuthService;

const TOKEN_TTL_MINUTES: i64 = 15;

impl AuthService {
    pub async fn login(&self, req: LoginRequest) -> Result<String, AuthError> {
        let user = self.users.find_or_create_user(&req.email).await?;

        let token = generate_token();
        self.token_store.insert(&token, &user.id, TOKEN_TTL_MINUTES);

        log::info!(
            "Magic link for {}: http://localhost:8080/api/v1/auth/verify?token={}",
            user.email,
            token
        );

        Ok(token)
    }
}

fn generate_token() -> String {
    rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(32)
        .map(char::from)
        .collect()
}
