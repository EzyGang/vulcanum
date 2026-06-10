use actix_web::{dev::Payload, Error, FromRequest, HttpRequest};
use uuid::Uuid;

use crate::app_state::AppState;
use crate::errors::AppError;

/// Authenticated principal that may be either a worker or an instance admin.
/// Used for endpoints that both workers (their own jobs) and admins (any job)
/// need to read from.
#[derive(Clone)]
pub enum WorkerOrInstanceAuth {
    Worker {
        worker_id: Uuid,
    },
    User {
        user_id: String,
        team_id: Option<Uuid>,
    },
    Instance,
}

impl FromRequest for WorkerOrInstanceAuth {
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
            None => return std::future::ready(Err(AppError::AuthHeaderMissing.into())),
        };

        let state = match req.app_data::<actix_web::web::Data<AppState>>() {
            Some(s) => s,
            None => return std::future::ready(Err(AppError::Internal.into())),
        };

        let decoding = jsonwebtoken::DecodingKey::from_secret(state.jwt_secret.as_bytes());
        let validation = jsonwebtoken::Validation::default();

        match jsonwebtoken::decode::<serde_json::Value>(token, &decoding, &validation) {
            Ok(data) => {
                if let Some(sub) = data.claims.get("sub").and_then(|s| s.as_str()) {
                    let token_type = data.claims.get("typ").and_then(|s| s.as_str());
                    if sub == "instance" && matches!(token_type, None | Some("instance")) {
                        return std::future::ready(Ok(Self::Instance));
                    }
                    if token_type == Some("user") {
                        let team_id = req
                            .headers()
                            .get("X-Team-Id")
                            .and_then(|v| v.to_str().ok())
                            .and_then(|v| Uuid::parse_str(v).ok());
                        return std::future::ready(Ok(Self::User {
                            user_id: sub.to_owned(),
                            team_id,
                        }));
                    }
                    if matches!(token_type, None | Some("worker")) {
                        if let Ok(worker_id) = Uuid::parse_str(sub) {
                            return std::future::ready(Ok(Self::Worker { worker_id }));
                        }
                    }
                }
                std::future::ready(Err(AppError::InvalidToken.into()))
            }
            Err(_) => std::future::ready(Err(AppError::InvalidToken.into())),
        }
    }
}
