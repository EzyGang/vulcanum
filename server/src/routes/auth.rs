use actix_web::{web, HttpResponse};
use vulcanum_shared::api_types::{AuthExchangeRequest, AuthModeResponse, InstanceLoginRequest};

use crate::app_state::AppState;
use crate::errors::AppError;
use crate::models::auth::model::{
    GithubCallbackQuery, GithubStartQuery, LoginRequest, LoginResponse, LogoutRequest,
    RefreshRequest, TeamPrincipal, VerifyQuery, VerifyResponse,
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

pub async fn github_callback(
    state: web::Data<AppState>,
    query: web::Query<GithubCallbackQuery>,
) -> Result<HttpResponse, AppError> {
    let result = state
        .auth
        .github_callback(&query.code, &query.state)
        .await?;
    let code = state.auth.create_user_callback_code(&result.token_pair)?;
    let location = append_code_to_return_path(&result.return_to, &code);

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
