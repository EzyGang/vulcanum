use crate::services::auth::service::AuthService;

impl AuthService {
    pub fn revoke(&self, token: &str) {
        self.token_store.consume(token);
    }
}
