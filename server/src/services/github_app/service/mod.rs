pub(crate) mod pull_requests;
mod repos;
mod state_nonce;
pub(crate) mod webhooks;
#[cfg(test)]
mod webhooks_tests;

use base64::Engine;
use octocrab::models::{Installation, InstallationId};
use octocrab::Octocrab;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::config::AppConfig;
use crate::db::github_app::GithubAppRepository;
use crate::models::github_app::errors::GithubAppError;
use crate::models::github_app::model::GithubInstallation;

pub struct GithubAppManager {
    pub(crate) repo: GithubAppRepository,
    pub(crate) db: sqlx::PgPool,
    pub(crate) redis_client: redis::Client,
    pub(crate) app_id: Option<u64>,
    pub(crate) app_private_key: Option<String>,
    pub(crate) app_slug: Option<String>,
}

impl Clone for GithubAppManager {
    fn clone(&self) -> Self {
        Self {
            repo: self.repo.clone(),
            db: self.db.clone(),
            redis_client: self.redis_client.clone(),
            app_id: self.app_id,
            app_private_key: self.app_private_key.clone(),
            app_slug: self.app_slug.clone(),
        }
    }
}

#[derive(Debug, serde::Serialize)]
pub struct InstallationToken {
    pub token: String,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, serde::Serialize)]
pub struct RepoInfo {
    pub owner: String,
    pub name: String,
    pub full_name: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GithubInstallState {
    pub user_id: Option<String>,
    pub team_id: Uuid,
}

impl GithubAppManager {
    pub fn new(
        repo: GithubAppRepository,
        db: sqlx::PgPool,
        redis_url: &str,
        cfg: &AppConfig,
    ) -> Result<Self, GithubAppError> {
        let redis_client =
            redis::Client::open(redis_url).map_err(|e| GithubAppError::Redis(e.to_string()))?;
        Ok(Self {
            repo,
            db,
            redis_client,
            app_id: cfg.github_app_id,
            app_private_key: cfg.github_app_private_key.clone(),
            app_slug: cfg.github_app_slug.clone(),
        })
    }

    pub(super) fn app_octocrab(&self) -> Result<Octocrab, GithubAppError> {
        let app_id = self.app_id.ok_or(GithubAppError::NotConfigured)?;
        let key_b64 = self
            .app_private_key
            .as_ref()
            .ok_or(GithubAppError::NotConfigured)?;
        let key_pem = base64::engine::general_purpose::STANDARD
            .decode(key_b64)
            .map_err(|e| GithubAppError::Base64Decode(format!("{e}")))?;
        let key = jsonwebtoken::EncodingKey::from_rsa_pem(&key_pem)
            .map_err(|e| GithubAppError::Api(format!("invalid private key: {e}")))?;
        Octocrab::builder()
            .app(octocrab::models::AppId(app_id), key)
            .build()
            .map_err(|e| GithubAppError::Api(format!("octocrab build failed: {e}")))
    }

    pub async fn install_url(&self, state: &str) -> Result<String, GithubAppError> {
        let slug = self
            .app_slug
            .as_ref()
            .ok_or(GithubAppError::NotConfigured)?;
        let url = format!("https://github.com/apps/{slug}/installations/new?state={state}");
        Ok(url)
    }

    pub async fn create_installation(
        &self,
        team_id: Uuid,
        installed_by_user_id: Option<&str>,
        github_installation_id: i64,
    ) -> Result<GithubInstallation, GithubAppError> {
        let octo = self.app_octocrab()?;
        let installation = octo
            .apps()
            .installation(InstallationId(github_installation_id as u64))
            .await
            .map_err(|e| GithubAppError::Api(format!("get_installation from GitHub: {e}")))?;

        self.upsert_installation(
            team_id,
            installed_by_user_id,
            github_installation_id,
            installation.account.login,
        )
        .await
    }

    pub async fn delete_installation(&self, id: i64, team_id: Uuid) -> Result<(), GithubAppError> {
        self.repo.delete_installation(&self.db, id, team_id).await
    }

    pub async fn get_installation(
        &self,
        team_id: Uuid,
        discover_remote: bool,
    ) -> Result<Option<GithubInstallation>, GithubAppError> {
        if let Some(inst) = self.repo.get_installation(&self.db, team_id).await? {
            return Ok(Some(inst));
        }

        if !discover_remote || self.app_id.is_none() || self.app_private_key.is_none() {
            return Ok(None);
        }

        self.discover_single_installation(team_id).await
    }

    async fn discover_single_installation(
        &self,
        team_id: Uuid,
    ) -> Result<Option<GithubInstallation>, GithubAppError> {
        let octo = self.app_octocrab()?;
        let page = octo
            .apps()
            .installations()
            .per_page(2u8)
            .send()
            .await
            .map_err(|e| GithubAppError::Api(format!("list_installations from GitHub: {e}")))?;

        let mut installations = page.items.into_iter();
        let installation = match (installations.next(), installations.next()) {
            (Some(installation), None) => installation,
            (None, _) => return Ok(None),
            (Some(_), Some(_)) => {
                tracing::warn!(
                    "github app has multiple installations; refusing automatic recovery"
                );
                return Ok(None);
            }
        };

        self.upsert_remote_installation(team_id, installation)
            .await
            .map(Some)
    }

    async fn upsert_remote_installation(
        &self,
        team_id: Uuid,
        installation: Installation,
    ) -> Result<GithubInstallation, GithubAppError> {
        let github_installation_id = i64::try_from(installation.id.into_inner()).map_err(|e| {
            GithubAppError::Api(format!("installation id does not fit database type: {e}"))
        })?;

        self.upsert_installation(
            team_id,
            None,
            github_installation_id,
            installation.account.login,
        )
        .await
    }

    async fn upsert_installation(
        &self,
        team_id: Uuid,
        installed_by_user_id: Option<&str>,
        github_installation_id: i64,
        account_login: String,
    ) -> Result<GithubInstallation, GithubAppError> {
        self.repo
            .insert_installation(
                &self.db,
                team_id,
                installed_by_user_id,
                github_installation_id,
                &account_login,
            )
            .await
    }
}
