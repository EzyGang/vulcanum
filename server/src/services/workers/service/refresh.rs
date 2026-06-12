use chrono::{Duration, Utc};
use sha2::Digest;
use uuid::Uuid;

use crate::services::workers::errors::WorkersError;
use crate::services::workers::model;
use crate::services::workers::model::{RefreshRequest, RefreshResponse};
use crate::services::workers::service::WorkersService;

impl WorkersService {
    pub async fn refresh(&self, req: RefreshRequest) -> Result<RefreshResponse, WorkersError> {
        let hash = hash_token(&req.refresh_token);
        let worker = self
            .repo
            .find_by_refresh_token_hash(&self.db, &hash)
            .await?;

        if Utc::now() > worker.refresh_expires_at {
            return Err(WorkersError::RefreshTokenExpired);
        }

        let new_refresh_token = generate_random_token(model::TOKEN_LENGTH);
        let new_hash = hash_token(&new_refresh_token);
        let new_expires_at = Utc::now() + Duration::days(model::REFRESH_TOKEN_TTL_DAYS);

        let updated = self
            .repo
            .update_refresh_token(&self.db, worker.id, &new_hash, new_expires_at)
            .await?;

        let (access_token, expires_at) = build_jwt(updated.id, &self.jwt_secret)?;

        Ok(RefreshResponse {
            access_token,
            refresh_token: new_refresh_token,
            expires_at,
        })
    }
}

fn generate_random_token(length: usize) -> String {
    vulcanum_shared::crypto::generate_alphanumeric_string(length)
}

fn hash_token(token: &str) -> String {
    let mut hasher = sha2::Sha256::new();
    hasher.update(token.as_bytes());
    hex::encode(hasher.finalize())
}

fn build_jwt(
    worker_id: Uuid,
    secret: &str,
) -> Result<(String, chrono::DateTime<chrono::Utc>), jsonwebtoken::errors::Error> {
    let exp = Utc::now() + Duration::minutes(model::ACCESS_TOKEN_TTL_MINUTES);
    let claims = jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &serde_json::json!({"sub": worker_id.to_string(), "typ": "worker", "exp": exp.timestamp()}),
        &jsonwebtoken::EncodingKey::from_secret(secret.as_bytes()),
    )?;
    Ok((claims, exp))
}
