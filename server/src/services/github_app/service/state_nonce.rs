use crate::models::github_app::errors::GithubAppError;
use crate::services::github_app::service::{GithubAppManager, GithubInstallState};

impl GithubAppManager {
    pub async fn save_state_nonce(
        &self,
        state: &str,
        install_state: &GithubInstallState,
    ) -> Result<(), GithubAppError> {
        let mut conn = self
            .redis_client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| GithubAppError::Redis(e.to_string()))?;
        let key = format!("vulcanum:github_state:{state}");
        let value = serde_json::to_string(install_state)
            .map_err(|e| GithubAppError::Api(format!("serialize install state: {e}")))?;
        redis::cmd("SETEX")
            .arg(&key)
            .arg(600u64)
            .arg(value)
            .query_async::<()>(&mut conn)
            .await
            .map_err(|e| GithubAppError::Redis(e.to_string()))?;
        Ok(())
    }

    pub async fn verify_and_consume_state_nonce(
        &self,
        state: &str,
    ) -> Result<Option<GithubInstallState>, GithubAppError> {
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
                return v
            end
            return nil
        "#,
        );
        let value: Option<String> = script
            .key(&key)
            .invoke_async(&mut conn)
            .await
            .map_err(|e| GithubAppError::Redis(e.to_string()))?;

        match value {
            Some(value) => serde_json::from_str(&value)
                .map(Some)
                .map_err(|e| GithubAppError::Api(format!("parse install state: {e}"))),
            None => Ok(None),
        }
    }
}
