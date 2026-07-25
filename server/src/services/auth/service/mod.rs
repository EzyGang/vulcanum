use std::time::Duration;

use sqlx::PgPool;

use crate::db::auth::AuthRepository;
use crate::db::github_app::GithubAppRepository;
use crate::models::auth::errors::AuthError;
use crate::models::auth::model::{IdentityInfo, MeResponse, TeamInfo, UserInfo};
use crate::services::auth::token_store::TokenStore;
use crate::services::teams::service::TeamsService;
use crate::services::users::service::UsersService;

pub mod github_oauth;
pub mod instance_login;
pub mod login;
pub mod refresh;
pub mod verify;

const GITHUB_OAUTH_HTTP_TIMEOUT: Duration = Duration::from_secs(15);

#[derive(Clone)]
pub struct AuthService {
    repo: AuthRepository,
    github_repo: GithubAppRepository,
    db: PgPool,
    users: UsersService,
    teams: TeamsService,
    token_store: TokenStore,
    instance_password: String,
    jwt_secret: String,
    is_single_user: bool,
    github_oauth_client_id: Option<String>,
    github_oauth_client_secret: Option<String>,
    github_oauth_redirect_url: Option<String>,
    github_oauth_http: reqwest::Client,
}

impl AuthService {
    pub fn new(
        repo: AuthRepository,
        github_repo: GithubAppRepository,
        db: PgPool,
        users: UsersService,
        teams: TeamsService,
        cfg: &crate::config::AppConfig,
    ) -> Result<Self, AuthError> {
        let github_oauth_http = reqwest::Client::builder()
            .timeout(GITHUB_OAUTH_HTTP_TIMEOUT)
            .build()
            .map_err(|e| AuthError::GithubOAuth(format!("building http client: {e}")))?;
        Ok(Self {
            repo,
            github_repo,
            db,
            users,
            teams,
            token_store: TokenStore::new(),
            instance_password: cfg.instance_password.clone(),
            jwt_secret: cfg.jwt_secret.clone(),
            is_single_user: cfg.is_single_user,
            github_oauth_client_id: cfg.github_oauth_client_id.clone(),
            github_oauth_client_secret: cfg.github_oauth_client_secret.clone(),
            github_oauth_redirect_url: cfg.github_oauth_redirect_url.clone(),
            github_oauth_http,
        })
    }

    pub async fn me(&self, user_id: &str) -> Result<MeResponse, AuthError> {
        let user = self.users.find_user_by_id(user_id).await?;
        let teams = self
            .teams
            .list_for_user(user_id)
            .await?
            .into_iter()
            .map(TeamInfo::from)
            .collect();
        let identities = self
            .teams
            .list_identities_for_user(user_id)
            .await?
            .into_iter()
            .map(IdentityInfo::from)
            .collect();

        Ok(MeResponse {
            user: UserInfo::from(user),
            teams,
            identities,
        })
    }
}
