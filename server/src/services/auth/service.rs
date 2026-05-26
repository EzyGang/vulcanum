use crate::services::auth::token_store::TokenStore;
use crate::services::users::service::UsersService;

pub mod instance_login;
pub mod login;
pub mod verify;

#[derive(Clone)]
pub struct AuthService {
    pub users: UsersService,
    pub token_store: TokenStore,
    pub instance_password: String,
    pub jwt_secret: String,
}

impl AuthService {
    pub fn new(users: UsersService, instance_password: String, jwt_secret: String) -> Self {
        Self {
            users,
            token_store: TokenStore::new(),
            instance_password,
            jwt_secret,
        }
    }
}
