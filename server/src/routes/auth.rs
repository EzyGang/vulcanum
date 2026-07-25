use actix_web::{web, HttpResponse};
use serde::Serialize;
use vulcanum_shared::api::wire::{AuthExchangeRequest, AuthModeResponse, InstanceLoginRequest};

use crate::app_state::AppState;
use crate::errors::AppError;
use crate::models::auth::model::{
    GithubCallbackQuery, GithubCallbackResult, GithubStartQuery, LoginRequest, LoginResponse,
    LogoutRequest, RefreshRequest, TeamPrincipal, VerifyQuery, VerifyResponse,
};

pub async fn login(
    state: web::Data<AppState>,
    body: web::Json<LoginRequest>,
) -> Result<HttpResponse, AppError> {
    let _token = state.auth.login(body.into_inner()).await?;

    Ok(HttpResponse::Ok().json(LoginResponse {
        message: "If the email is registered, a magic link has been logged".to_owned(),
    }))
}

pub async fn verify(
    state: web::Data<AppState>,
    query: web::Query<VerifyQuery>,
) -> Result<HttpResponse, AppError> {
    let (user, token_pair) = state.auth.verify(query.into_inner()).await?;

    Ok(HttpResponse::Ok().json(VerifyResponse {
        message: "Logged in successfully".to_owned(),
        user: user.into(),
        token_pair,
    }))
}

pub async fn instance_login(
    state: web::Data<AppState>,
    body: web::Json<InstanceLoginRequest>,
) -> Result<HttpResponse, AppError> {
    let response = state.auth.instance_login(&body.password).await?;

    Ok(HttpResponse::Ok().json(response))
}

pub async fn mode(state: web::Data<AppState>) -> Result<HttpResponse, AppError> {
    Ok(HttpResponse::Ok().json(AuthModeResponse {
        is_single_user: state.is_single_user,
    }))
}

pub async fn github_start(
    state: web::Data<AppState>,
    query: web::Query<GithubStartQuery>,
) -> Result<HttpResponse, AppError> {
    let url = state
        .auth
        .github_authorize_url(query.return_to.as_deref())
        .await?;

    Ok(HttpResponse::Found()
        .append_header(("Location", url))
        .finish())
}

#[derive(Serialize)]
pub struct GithubLinkUrlResponse {
    pub url: String,
}

pub async fn github_link_url(
    state: web::Data<AppState>,
    auth: TeamPrincipal,
    query: web::Query<GithubStartQuery>,
) -> Result<HttpResponse, AppError> {
    let url = state
        .auth
        .github_link_authorize_url(&auth, query.return_to.as_deref())
        .await?;

    Ok(HttpResponse::Ok().json(GithubLinkUrlResponse { url }))
}

pub async fn github_callback(
    state: web::Data<AppState>,
    query: web::Query<GithubCallbackQuery>,
) -> Result<HttpResponse, AppError> {
    complete_github_callback(state.get_ref(), &query.code, &query.state).await
}

pub(crate) async fn complete_github_callback(
    state: &AppState,
    code: &str,
    oauth_state: &str,
) -> Result<HttpResponse, AppError> {
    let location = match state.auth.github_callback(code, oauth_state).await? {
        GithubCallbackResult::Login {
            token_pair,
            return_to,
        } => {
            let code = state.auth.create_user_callback_code(&token_pair)?;
            append_code_to_return_path(&return_to, &code)
        }
        GithubCallbackResult::IdentityLinked { return_to } => return_to,
    };

    Ok(HttpResponse::Found()
        .append_header(("Location", location))
        .finish())
}

pub(crate) fn append_code_to_return_path(return_to: &str, code: &str) -> String {
    let separator = match return_to.contains('?') {
        true => '&',
        false => '?',
    };
    format!("{return_to}{separator}code={code}")
}

pub async fn exchange(
    state: web::Data<AppState>,
    body: web::Json<AuthExchangeRequest>,
) -> Result<HttpResponse, AppError> {
    let response = state.auth.exchange_user_callback_code(&body.code)?;

    Ok(HttpResponse::Ok().json(response))
}

pub async fn refresh(
    state: web::Data<AppState>,
    body: web::Json<RefreshRequest>,
) -> Result<HttpResponse, AppError> {
    let response = state.auth.refresh_token(&body.refresh_token).await?;

    Ok(HttpResponse::Ok().json(response))
}

pub async fn me(state: web::Data<AppState>, auth: TeamPrincipal) -> Result<HttpResponse, AppError> {
    let user_id = match auth {
        TeamPrincipal::User { user_id, .. } => user_id,
        TeamPrincipal::Instance { .. } => return Err(AppError::Forbidden),
    };
    let response = state.auth.me(&user_id).await?;

    Ok(HttpResponse::Ok().json(response))
}

pub async fn logout(
    state: web::Data<AppState>,
    body: Option<web::Json<LogoutRequest>>,
) -> Result<HttpResponse, AppError> {
    if let Some(refresh_token) = body.and_then(|body| body.into_inner().refresh_token) {
        state.auth.revoke_refresh_token(&refresh_token).await?;
    }

    Ok(HttpResponse::NoContent().finish())
}
