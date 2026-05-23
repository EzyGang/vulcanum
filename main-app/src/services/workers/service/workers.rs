use chrono::{DateTime, Duration, Utc};
use rand::Rng;
use serde_json::json;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::services::workers::errors::WorkersError;
use crate::services::workers::model::{self, WorkerResponse};
use crate::services::workers::model::{
    CodeResponse, ConnectRequest, ConnectResponse, RefreshRequest, RefreshResponse,
};
use crate::services::workers::service::WorkersService;

impl WorkersService {
    pub async fn generate_code(&self) -> CodeResponse {
        let mut codes = self.codes.write().await;
        Self::evict_expired(&mut codes);

        let code: String = rand::thread_rng()
            .sample_iter(&rand::distributions::Alphanumeric)
            .take(model::CODE_LENGTH)
            .map(char::from)
            .collect();

        let expires_at = Utc::now() + Duration::minutes(model::CODE_TTL_MINUTES);
        codes.insert(code.clone(), expires_at);

        CodeResponse { code, expires_at }
    }

    pub async fn connect(&self, req: ConnectRequest) -> Result<ConnectResponse, WorkersError> {
        let mut codes = self.codes.write().await;
        let expiry = codes.remove(&req.code).ok_or(WorkersError::CodeNotFound)?;

        if Utc::now() > expiry {
            return Err(WorkersError::CodeExpired);
        }

        let refresh_token = generate_random_token(model::TOKEN_LENGTH);
        let refresh_hash = hash_token(&refresh_token);
        let refresh_expires_at = Utc::now() + Duration::days(model::REFRESH_TOKEN_TTL_DAYS);

        let worker = self
            .repo
            .create(
                &self.db,
                &req.worker_name,
                &refresh_hash,
                refresh_expires_at,
                &json!({}),
            )
            .await?;

        let (access_token, expires_at) = build_jwt(worker.id, &self.jwt_secret)?;

        Ok(ConnectResponse {
            access_token,
            refresh_token,
            expires_at,
            worker_id: worker.id,
            name: worker.name,
        })
    }

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

    pub async fn list_all(&self) -> Result<Vec<WorkerResponse>, WorkersError> {
        let workers = self.repo.list_all(&self.db).await?;
        Ok(workers.into_iter().map(WorkerResponse::from).collect())
    }

    pub async fn delete_worker(&self, worker_id: uuid::Uuid) -> Result<(), WorkersError> {
        self.repo.delete(&self.db, worker_id).await
    }

    fn evict_expired(codes: &mut std::collections::HashMap<String, DateTime<Utc>>) {
        let now = Utc::now();
        codes.retain(|_k, v| *v > now);
    }
}

fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    hex::encode(hasher.finalize())
}

fn generate_random_token(length: usize) -> String {
    vulcanum_shared::crypto::generate_alphanumeric_string(length)
}

fn build_jwt(
    worker_id: Uuid,
    secret: &str,
) -> Result<(String, DateTime<Utc>), jsonwebtoken::errors::Error> {
    let exp = Utc::now() + Duration::minutes(model::ACCESS_TOKEN_TTL_MINUTES);
    let claims = jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &serde_json::json!({"sub": worker_id.to_string(), "exp": exp.timestamp()}),
        &jsonwebtoken::EncodingKey::from_secret(secret.as_bytes()),
    )?;
    Ok((claims, exp))
}
