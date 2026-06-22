use chrono::{Duration, Utc};
use serde_json::json;

use crate::services::workers::errors::WorkersError;
use crate::services::workers::model;
use crate::services::workers::repository::queries::CreateWorkerParams;
use crate::services::workers::service::token::{build_jwt, generate_random_token, hash_token};
use crate::services::workers::service::WorkersService;
use vulcanum_shared::api_types::{ConnectRequest, ConnectResponse};

impl WorkersService {
    pub async fn connect(&self, req: ConnectRequest) -> Result<ConnectResponse, WorkersError> {
        let registration = self
            .code_store
            .consume(&req.code)
            .await?
            .ok_or(WorkersError::CodeNotFound)?;

        if Utc::now() > registration.expires_at {
            return Err(WorkersError::CodeExpired);
        }

        let refresh_token = generate_random_token();
        let refresh_hash = hash_token(&refresh_token);
        let refresh_expires_at = Utc::now() + Duration::days(model::REFRESH_TOKEN_TTL_DAYS);
        let max_concurrent_jobs = req
            .max_concurrent_jobs
            .unwrap_or(model::DEFAULT_MAX_CONCURRENT_JOBS);
        let capabilities = json!({});

        let worker = self
            .repo
            .create(
                &self.db,
                CreateWorkerParams {
                    team_id: registration.team_id,
                    name: &req.worker_name,
                    refresh_token_hash: &refresh_hash,
                    refresh_expires_at,
                    capabilities: &capabilities,
                    max_concurrent_jobs,
                },
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
