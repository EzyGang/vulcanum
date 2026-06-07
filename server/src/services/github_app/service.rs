use octocrab::models::InstallationId;
use octocrab::Octocrab;

use base64::Engine;

use crate::config::AppConfig;
use crate::services::github_app::errors::GithubAppError;
use crate::services::github_app::model::GithubInstallation;
use crate::services::github_app::repository::GithubAppRepository;

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

    fn app_octocrab(&self) -> Result<Octocrab, GithubAppError> {
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

    pub async fn save_state_nonce(&self, state: &str) -> Result<(), GithubAppError> {
        let mut conn = self
            .redis_client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| GithubAppError::Redis(e.to_string()))?;
        let key = format!("vulcanum:github_state:{state}");
        redis::cmd("SETEX")
            .arg(&key)
            .arg(600u64)
            .arg("1")
            .query_async::<()>(&mut conn)
            .await
            .map_err(|e| GithubAppError::Redis(e.to_string()))?;
        Ok(())
    }

    pub async fn verify_and_consume_state_nonce(
        &self,
        state: &str,
    ) -> Result<bool, GithubAppError> {
        let mut conn = self
            .redis_client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| GithubAppError::Redis(e.to_string()))?;
        let key = format!("vulcanum:github_state:{state}");

        let script = redis::Script::new(
            r#"
            local v = redis.call("GET", KEYS[1])
            if v then
                redis.call("DEL", KEYS[1])
                return 1
            end
            return 0
        "#,
        );
        let existed: i64 = script
            .key(&key)
            .invoke_async(&mut conn)
            .await
            .map_err(|e| GithubAppError::Redis(e.to_string()))?;

        Ok(existed == 1)
    }

    pub async fn create_installation(
        &self,
        github_installation_id: i64,
    ) -> Result<GithubInstallation, GithubAppError> {
        let octo = self.app_octocrab()?;
        let installation = octo
            .apps()
            .installation(InstallationId(github_installation_id as u64))
            .await
            .map_err(|e| GithubAppError::Api(format!("get_installation from GitHub: {e}")))?;

        self.upsert_installation(github_installation_id, installation.account.login)
            .await
    }

    pub async fn delete_installation(&self, id: i64) -> Result<(), GithubAppError> {
        self.repo.delete_installation(&self.db, id).await
    }

    pub async fn get_installation(&self) -> Result<Option<GithubInstallation>, GithubAppError> {
        if let Some(inst) = self.repo.get_installation(&self.db).await? {
            return Ok(Some(inst));
        }
        match self.sync_installation_from_github().await {
            Ok(inst) => {
                tracing::info!(
                    github_installation_id = inst.github_installation_id,
                    account_login = inst.account_login,
                    "discovered installation from GitHub API"
                );
                Ok(Some(inst))
            }
            Err(GithubAppError::NoInstallation) => {
                tracing::warn!("no installations found for this GitHub App");
                Ok(None)
            }
            Err(GithubAppError::NotConfigured) => {
                tracing::warn!(
                    "github app not configured — set GITHUB_APP_ID, GITHUB_APP_PRIVATE_KEY, GITHUB_APP_SLUG"
                );
                Ok(None)
            }
            Err(e) => {
                tracing::error!(error = %e, "failed to sync installation from GitHub");
                Err(e)
            }
        }
    }

    pub async fn sync_installation_from_github(
        &self,
    ) -> Result<GithubInstallation, GithubAppError> {
        let octo = self.app_octocrab()?;
        let page = octo
            .apps()
            .installations()
            .send()
            .await
            .map_err(|e| GithubAppError::Api(format!("list installations: {e}")))?;

        let first = page
            .items
            .into_iter()
            .next()
            .ok_or(GithubAppError::NoInstallation)?;

        let gh_installation_id: i64 = first.id.0 as i64;

        self.upsert_installation(gh_installation_id, first.account.login)
            .await
    }

    async fn upsert_installation(
        &self,
        github_installation_id: i64,
        account_login: String,
    ) -> Result<GithubInstallation, GithubAppError> {
        self.repo
            .insert_installation(&self.db, github_installation_id, &account_login)
            .await
    }

    pub async fn list_repos(&self) -> Result<Vec<RepoInfo>, GithubAppError> {
        let installation = self
            .repo
            .get_installation(&self.db)
            .await?
            .ok_or(GithubAppError::NoInstallation)?;

        let octo = self.app_octocrab()?;
        let installation_client = octo
            .installation(InstallationId(installation.github_installation_id as u64))
            .map_err(|e| GithubAppError::Api(format!("installation client: {e}")))?;

        let repos = installation_client
            .current()
            .list_repos_for_authenticated_user()
            .send()
            .await
            .map_err(|e| GithubAppError::Api(format!("list_repos: {e}")))?;

        let infos = repos
            .items
            .into_iter()
            .map(|r| RepoInfo {
                owner: r.owner.map(|o| o.login).unwrap_or_default(),
                name: r.name,
                full_name: r.full_name.unwrap_or_default(),
            })
            .collect();

        Ok(infos)
    }

    pub async fn generate_installation_token(
        &self,
        repo_url: &str,
    ) -> Result<InstallationToken, GithubAppError> {
        let installation = self
            .repo
            .get_installation(&self.db)
            .await?
            .ok_or(GithubAppError::NoInstallation)?;

        let (_owner, repo_name) = parse_github_repo(repo_url)?;

        let octo = self.app_octocrab()?;
        let route = format!(
            "/app/installations/{}/access_tokens",
            installation.github_installation_id
        );

        let body = serde_json::json!({
            "repositories": [repo_name],
            "permissions": {
                "contents": "write",
                "pull_requests": "write"
            }
        });

        let response: octocrab::models::InstallationToken = octo
            .post(&route, Some(&body))
            .await
            .map_err(|e| GithubAppError::Api(format!("token mint failed: {e}")))?;

        let expires_at = response
            .expires_at
            .as_ref()
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|dt: chrono::DateTime<chrono::FixedOffset>| dt.with_timezone(&chrono::Utc))
            .unwrap_or_else(chrono::Utc::now);

        Ok(InstallationToken {
            token: response.token,
            expires_at,
        })
    }
}

fn parse_github_repo(url: &str) -> Result<(String, String), GithubAppError> {
    url.strip_prefix("https://github.com/")
        .or_else(|| url.strip_prefix("http://github.com/"))
        .and_then(|rest| rest.rsplit_once('/'))
        .ok_or_else(|| GithubAppError::InvalidRepoUrl(url.to_string()))
        .map(|(owner, repo)| {
            let repo = repo.strip_suffix(".git").unwrap_or(repo);
            (owner.to_string(), repo.to_string())
        })
}
