use chrono::{Duration, Utc};

use crate::models::workers::errors::WorkersError;
use crate::models::workers::model;
use crate::services::workers::service::token::{build_jwt, generate_random_token, hash_token};
use crate::services::workers::service::WorkersService;
use vulcanum_shared::api_types::{RefreshRequest, RefreshResponse};

impl WorkersService {
    pub async fn refresh(&self, req: RefreshRequest) -> Result<RefreshResponse, WorkersError> {
        let hash = hash_token(&req.refresh_token);

        let new_refresh_token = generate_random_token();
        let new_hash = hash_token(&new_refresh_token);
        let new_expires_at = Utc::now() + Duration::days(model::REFRESH_TOKEN_TTL_DAYS);

        let updated = match self
            .repo
            .rotate_refresh_token(&self.db, &hash, &new_hash, new_expires_at)
            .await
        {
            Ok(worker) => worker,
            Err(WorkersError::InvalidRefreshToken) => {
                let worker = self
                    .repo
                    .find_by_refresh_token_hash(&self.db, &hash)
                    .await?;
                if Utc::now() > worker.refresh_expires_at {
                    return Err(WorkersError::RefreshTokenExpired);
                }

                return Err(WorkersError::InvalidRefreshToken);
            }
            Err(err) => return Err(err),
        };

        let (access_token, expires_at) = build_jwt(updated.id, &self.jwt_secret)?;

        Ok(RefreshResponse {
            access_token,
            refresh_token: new_refresh_token,
            expires_at,
        })
    }
}
