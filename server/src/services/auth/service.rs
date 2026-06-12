use sqlx::PgPool;

use crate::services::auth::repository::AuthRepository;
use crate::services::auth::token_store::TokenStore;
use crate::services::teams::service::TeamsService;
use crate::services::users::service::UsersService;

pub mod github_oauth;
pub mod instance_login;
pub mod login;
pub mod refresh;
pub mod verify;

#[derive(Clone)]
pub struct AuthService {
    pub repo: AuthRepository,
    pub db: PgPool,
    pub users: UsersService,
    pub teams: TeamsService,
    pub token_store: TokenStore,
    pub instance_password: String,
    pub jwt_secret: String,
    pub is_single_user: bool,
    pub github_oauth_client_id: Option<String>,
    pub github_oauth_client_secret: Option<String>,
    pub github_oauth_redirect_url: Option<String>,
}

impl AuthService {
    pub fn new(
        repo: AuthRepository,
        db: PgPool,
        users: UsersService,
        teams: TeamsService,
        instance_password: String,
        jwt_secret: String,
        cfg: &crate::config::AppConfig,
    ) -> Self {
        Self {
            repo,
            db,
            users,
            teams,
            token_store: TokenStore::new(),
            instance_password,
            jwt_secret,
            is_single_user: cfg.is_single_user,
            github_oauth_client_id: cfg.github_oauth_client_id.clone(),
            github_oauth_client_secret: cfg.github_oauth_client_secret.clone(),
            github_oauth_redirect_url: cfg.github_oauth_redirect_url.clone(),
        }
    }
}
