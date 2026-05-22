use crate::services::auth::errors::AuthError;
use crate::services::auth::service::AuthService;

pub const INSTANCE_TOKEN_TTL_MINUTES: i64 = 15;

impl AuthService {
    pub fn instance_login(&self, password: &str) -> Result<String, AuthError> {
        if password != self.instance_password {
            return Err(AuthError::InvalidPassword);
        }

        let token = vulcanum_shared::crypto::generate_alphanumeric_string(32);

        self.token_store
            .insert(&token, "instance", INSTANCE_TOKEN_TTL_MINUTES);

        Ok(token)
    }
}
