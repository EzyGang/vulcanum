use chrono::{Duration, Utc};
use rand::Rng;

use crate::services::workers::errors::WorkersError;
use crate::services::workers::model;
use crate::services::workers::model::CodeResponse;
use crate::services::workers::service::WorkersService;

impl WorkersService {
    pub async fn generate_code(&self) -> Result<CodeResponse, WorkersError> {
        let code: String = rand::thread_rng()
            .sample_iter(&rand::distributions::Alphanumeric)
            .take(model::CODE_LENGTH)
            .map(char::from)
            .collect();

        let expires_at = Utc::now() + Duration::minutes(model::CODE_TTL_MINUTES);
        self.code_store.save(&code, expires_at).await?;

        Ok(CodeResponse { code, expires_at })
    }
}
