use crate::services::auth::token_store::TokenStore;
use crate::services::users::service::UsersService;

pub mod login;
pub mod verify;

#[derive(Clone)]
pub struct AuthService {
    pub users: UsersService,
    pub token_store: TokenStore,
}

impl AuthService {
    pub fn new(users: UsersService) -> Self {
        Self {
            users,
            token_store: TokenStore::new(),
        }
    }
}
