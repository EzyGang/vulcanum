use actix_web::{dev::Payload, Error, FromRequest, HttpRequest};

use crate::app_state::AppState;
use crate::errors::AppError;

pub struct InstanceAuth {
    pub token: String,
}

impl FromRequest for InstanceAuth {
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
            None => {
                return std::future::ready(Err(AppError::AuthHeaderMissing.into()));
            }
        };

        let state = match req.app_data::<actix_web::web::Data<AppState>>() {
            Some(s) => s,
            None => return std::future::ready(Err(AppError::Internal.into())),
        };

        if state.auth.token_store.validate(token) {
            std::future::ready(Ok(InstanceAuth {
                token: token.to_owned(),
            }))
        } else {
            std::future::ready(Err(AppError::InvalidToken.into()))
        }
    }
}
