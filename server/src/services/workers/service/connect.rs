use chrono::{Duration, Utc};
use serde_json::json;
use sha2::Digest;
use uuid::Uuid;

use crate::services::workers::errors::WorkersError;
use crate::services::workers::model;
use crate::services::workers::model::{ConnectRequest, ConnectResponse};
use crate::services::workers::service::WorkersService;

impl WorkersService {
    pub async fn connect(&self, req: ConnectRequest) -> Result<ConnectResponse, WorkersError> {
        let expiry = self
            .code_store
            .consume(&req.code)
            .await?
            .ok_or(WorkersError::CodeNotFound)?;

        if Utc::now() > expiry {
            return Err(WorkersError::CodeExpired);
        }

        let refresh_token = generate_random_token(model::TOKEN_LENGTH);
        let refresh_hash = hash_token(&refresh_token);
        let refresh_expires_at = Utc::now() + Duration::days(model::REFRESH_TOKEN_TTL_DAYS);
        let max_concurrent_jobs = req
            .max_concurrent_jobs
            .unwrap_or(model::DEFAULT_MAX_CONCURRENT_JOBS);

        let worker = self
            .repo
            .create(
                &self.db,
                &req.worker_name,
                &refresh_hash,
                refresh_expires_at,
                &json!({}),
                max_concurrent_jobs,
            )
            .await?;

        let (access_token, expires_at) = build_jwt(worker.id, &self.jwt_secret)?;

        tracing::info!(
            worker_id = worker.id.to_string().as_str(),
            worker_name = worker.name.as_str(),
            "worker '{}' registered with id {}",
            worker.name,
            worker.id,
        );

        Ok(ConnectResponse {
            access_token,
            refresh_token,
            expires_at,
            worker_id: worker.id,
            name: worker.name,
            max_concurrent_jobs: worker.max_concurrent_jobs,
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
        &serde_json::json!({"sub": worker_id.to_string(), "exp": exp.timestamp()}),
        &jsonwebtoken::EncodingKey::from_secret(secret.as_bytes()),
    )?;
    Ok((claims, exp))
}
