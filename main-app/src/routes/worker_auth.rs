use actix_web::{dev::Payload, Error, FromRequest, HttpRequest};
use uuid::Uuid;

use crate::app_state::AppState;
use crate::errors::AppError;

pub struct WorkerAuth {
    pub worker_id: Uuid,
}

impl FromRequest for WorkerAuth {
    type Error = Error;
    type Future = std::future::Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let header = req
            .headers()
            .get("Authorization")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "));

        let token = match header {
            Some(t) => t,
            None => return std::future::ready(Err(AppError::InvalidToken.into())),
        };

        let state = match req.app_data::<actix_web::web::Data<AppState>>() {
            Some(s) => s,
            None => return std::future::ready(Err(AppError::Internal.into())),
        };

        let decoding = jsonwebtoken::DecodingKey::from_secret(state.jwt_secret.as_bytes());
        let validation = jsonwebtoken::Validation::default();

        let token_data =
            match jsonwebtoken::decode::<serde_json::Value>(token, &decoding, &validation) {
                Ok(t) => t,
                Err(_) => {
                    return std::future::ready(Err(AppError::InvalidToken.into()));
                }
            };

        let sub = match token_data
            .claims
            .get("sub")
            .and_then(|s| s.as_str())
            .and_then(|s| Uuid::parse_str(s).ok())
        {
            Some(id) => id,
            None => {
                return std::future::ready(Err(AppError::InvalidToken.into()));
            }
        };

        std::future::ready(Ok(WorkerAuth { worker_id: sub }))
    }
}
