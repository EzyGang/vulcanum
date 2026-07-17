use crate::db::workers::queries::CreateWorkerParams;
use crate::models::workers::errors::WorkersError;
use crate::models::workers::model;
use crate::services::workers::service::token::{build_jwt, generate_random_token, hash_token};
use crate::services::workers::service::WorkersService;
use chrono::{Duration, Utc};
use vulcanum_shared::api::wire::{ConnectRequest, ConnectResponse};

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
        let max_concurrent_jobs = validate_max_concurrent_jobs(req.max_concurrent_jobs)?;
        let capabilities = serde_json::to_value(&req.capabilities).map_err(|e| {
            WorkersError::RegistrationFailed(format!("invalid worker capabilities: {e}"))
        })?;

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

fn validate_max_concurrent_jobs(value: Option<i32>) -> Result<i32, WorkersError> {
    let value = value.unwrap_or(model::DEFAULT_MAX_CONCURRENT_JOBS);
    if value < 1 {
        return Err(WorkersError::RegistrationFailed(
            "max_concurrent_jobs must be at least 1".to_owned(),
        ));
    }
    if value > vulcanum_shared::constants::MAX_WORKER_CAPACITY {
        return Err(WorkersError::RegistrationFailed(format!(
            "max_concurrent_jobs must be at most {}",
            vulcanum_shared::constants::MAX_WORKER_CAPACITY
        )));
    }

    Ok(value)
}
