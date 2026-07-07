use actix_web::{dev::Payload, Error, FromRequest, HttpRequest};
use uuid::Uuid;

use crate::errors::AppError;
use crate::routes::auth_utils::decode_jwt;

pub struct WorkerAuth {
    pub worker_id: Uuid,
}

impl FromRequest for WorkerAuth {
    type Error = Error;
    type Future = std::future::Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let claims = match decode_jwt::<serde_json::Value>(req, AppError::InvalidToken) {
            Ok(claims) => claims,
            Err(err) => return std::future::ready(Err(err.into())),
        };

        let sub = match claims
            .get("sub")
            .and_then(|s| s.as_str())
            .and_then(|s| Uuid::parse_str(s).ok())
        {
            Some(id) => id,
            None => {
                return std::future::ready(Err(AppError::InvalidToken.into()));
            }
        };

        let token_type = claims.get("typ").and_then(|s| s.as_str());
        if !matches!(token_type, None | Some("worker")) {
            return std::future::ready(Err(AppError::InvalidToken.into()));
        }

        std::future::ready(Ok(WorkerAuth { worker_id: sub }))
    }
}
