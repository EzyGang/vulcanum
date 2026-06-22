use chrono::{Duration, Utc};

use crate::services::workers::errors::WorkersError;
use crate::services::workers::model;
use crate::services::workers::service::token::{build_jwt, generate_random_token, hash_token};
use crate::services::workers::service::WorkersService;
use vulcanum_shared::api_types::{RefreshRequest, RefreshResponse};

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

        let new_refresh_token = generate_random_token();
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
