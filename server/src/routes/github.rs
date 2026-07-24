use actix_web::{web, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::app_state::AppState;
use crate::errors::AppError;
use crate::models::auth::model::TeamPrincipal;
use crate::services::github_app::service::webhooks::GithubWebhookError;
use crate::services::github_app::service::GithubInstallState;

pub async fn webhook(
    state: web::Data<AppState>,
    request: HttpRequest,
    body: web::Bytes,
) -> Result<HttpResponse, AppError> {
    let signature = match request
        .headers()
        .get("X-Hub-Signature-256")
        .and_then(|value| value.to_str().ok())
    {
        Some(signature) => signature,
        None => return Ok(HttpResponse::Unauthorized().finish()),
    };
    let event = match request
        .headers()
        .get("X-GitHub-Event")
        .and_then(|value| value.to_str().ok())
    {
        Some(event) => event,
        None => return Ok(HttpResponse::BadRequest().finish()),
    };
    let delivery = match request
        .headers()
        .get("X-GitHub-Delivery")
        .and_then(|value| value.to_str().ok())
    {
        Some(delivery) => delivery,
        None => return Ok(HttpResponse::BadRequest().finish()),
    };

    match state
        .github_webhooks
        .handle(signature, event, delivery, &body)
        .await
    {
        Ok(_) => Ok(HttpResponse::Accepted().finish()),
        Err(GithubWebhookError::InvalidSignature) => Ok(HttpResponse::Unauthorized().finish()),
        Err(GithubWebhookError::InvalidPayload(_)) => Ok(HttpResponse::BadRequest().finish()),
        Err(GithubWebhookError::NotConfigured | GithubWebhookError::MissingAppSlug) => {
            tracing::error!("GitHub webhook received without required GitHub App configuration");
            Ok(HttpResponse::ServiceUnavailable().finish())
        }
        Err(GithubWebhookError::Persistence(e)) => {
            tracing::error!(error = %e, "GitHub webhook delivery persistence failed");
            Err(AppError::Internal)
        }
    }
}

pub async fn auth_redirect(
    state: web::Data<AppState>,
    auth: TeamPrincipal,
) -> Result<HttpResponse, AppError> {
    let url = build_install_url_for_principal(&state, auth).await?;

    Ok(HttpResponse::Found()
        .append_header(("Location", url))
        .finish())
}

#[derive(Serialize)]
pub struct AuthUrlResponse {
    pub url: String,
}

pub async fn auth_url(
    state: web::Data<AppState>,
    auth: TeamPrincipal,
) -> Result<HttpResponse, AppError> {
    let url = build_install_url_for_principal(&state, auth).await?;

    Ok(HttpResponse::Ok().json(AuthUrlResponse { url }))
}

#[derive(Debug, Deserialize)]
pub struct CallbackQuery {
    pub code: Option<String>,
    pub installation_id: Option<i64>,
    pub setup_action: Option<String>,
    pub state: Option<String>,
}

#[derive(Debug, Eq, PartialEq)]
pub(super) enum CallbackKind<'a> {
    OAuth {
        code: &'a str,
        state_nonce: &'a str,
    },
    Installation {
        installation_id: i64,
        setup_action: &'a str,
        state_nonce: &'a str,
    },
}

#[must_use]
pub(super) fn classify_callback(query: &CallbackQuery) -> Option<CallbackKind<'_>> {
    match (
        query.code.as_deref(),
        query.installation_id,
        query.setup_action.as_deref(),
        query.state.as_deref(),
    ) {
        (Some(code), _, _, Some(state_nonce)) => Some(CallbackKind::OAuth { code, state_nonce }),
        (None, Some(installation_id), Some(setup_action), Some(state_nonce)) => {
            Some(CallbackKind::Installation {
                installation_id,
                setup_action,
                state_nonce,
            })
        }
        _ => None,
    }
}

pub async fn callback(
    state: web::Data<AppState>,
    query: web::Query<CallbackQuery>,
) -> Result<HttpResponse, AppError> {
    let (installation_id, setup_action, state_nonce) = match classify_callback(&query) {
        Some(CallbackKind::OAuth { code, state_nonce }) => {
            return oauth_callback(state.get_ref(), code, state_nonce).await;
        }
        Some(CallbackKind::Installation {
            installation_id,
            setup_action,
            state_nonce,
        }) => (installation_id, setup_action, state_nonce),
        None => return Ok(invalid_callback_response()),
    };

    if !matches!(setup_action, "install" | "update") {
        return Ok(HttpResponse::Found()
            .append_header(("Location", "/"))
            .finish());
    }

    let install_state = match state
        .github
        .verify_and_consume_state_nonce(state_nonce)
        .await
    {
        Ok(Some(install_state)) => install_state,
        Ok(None) => {
            tracing::warn!(state = %state_nonce, "github state nonce not found or expired");
            return Ok(HttpResponse::Found()
                .append_header(("Location", "/"))
                .finish());
        }
        Err(error) => {
            tracing::warn!(error = %error, state = %state_nonce, "github state verification failed");
            return Ok(HttpResponse::Found()
                .append_header(("Location", "/"))
                .finish());
        }
    };

    state
        .github
        .create_installation(
            install_state.team_id,
            install_state.user_id.as_deref(),
            installation_id,
        )
        .await?;

    Ok(HttpResponse::Found()
        .append_header(("Location", "/"))
        .finish())
}
async fn oauth_callback(
    state: &AppState,
    code: &str,
    state_nonce: &str,
) -> Result<HttpResponse, AppError> {
    match state
        .github
        .verify_and_consume_state_nonce(state_nonce)
        .await?
    {
        Some(install_state) => {
            let location = state
                .auth
                .complete_github_installation_authorization(&state.github, install_state, code)
                .await?;
            Ok(HttpResponse::Found()
                .append_header(("Location", location))
                .finish())
        }
        None => crate::routes::auth::complete_github_callback(state, code, state_nonce).await,
    }
}

fn invalid_callback_response() -> HttpResponse {
    HttpResponse::BadRequest()
        .content_type("text/plain; charset=utf-8")
        .body(
            "Invalid GitHub callback. Configure the GitHub App Callback URL and \
             GITHUB_OAUTH_REDIRECT_URL as /api/v1/github/callback.",
        )
}

pub async fn list_repos(
    state: web::Data<AppState>,
    auth: TeamPrincipal,
) -> Result<HttpResponse, AppError> {
    let team_id = state
        .teams
        .resolve_team(&auth, state.is_single_user)
        .await?;
    let repos = state.github.list_repos(team_id).await.map_err(|e| {
        tracing::warn!(error = %e, "list_repos failed");
        AppError::Internal
    })?;

    Ok(HttpResponse::Ok().json(repos))
}

pub async fn get_installation(
    state: web::Data<AppState>,
    auth: TeamPrincipal,
) -> Result<HttpResponse, AppError> {
    let team_id = state
        .teams
        .resolve_team(&auth, state.is_single_user)
        .await?;
    let inst = state
        .github
        .get_installation(team_id, state.is_single_user)
        .await
        .map_err(|e| {
            tracing::warn!(error = %e, "get_installation failed");
            AppError::Internal
        })?;

    Ok(HttpResponse::Ok().json(inst))
}

pub async fn delete_installation(
    state: web::Data<AppState>,
    path: web::Path<i64>,
    auth: TeamPrincipal,
) -> Result<HttpResponse, AppError> {
    let team_id = state
        .teams
        .resolve_team(&auth, state.is_single_user)
        .await?;
    state
        .github
        .delete_installation(path.into_inner(), team_id)
        .await
        .map_err(|e| {
            tracing::warn!(error = %e, "delete_installation failed");
            AppError::Internal
        })?;

    Ok(HttpResponse::NoContent().finish())
}

async fn build_install_url_for_principal(
    state: &web::Data<AppState>,
    auth: TeamPrincipal,
) -> Result<String, AppError> {
    let nonce = Uuid::new_v4().to_string();
    let team_id = state
        .teams
        .resolve_team(&auth, state.is_single_user)
        .await?;
    let user_id = match auth {
        TeamPrincipal::User { user_id, .. } => Some(user_id),
        TeamPrincipal::Instance { .. } => None,
    };
    state
        .github
        .save_state_nonce(&nonce, &GithubInstallState { user_id, team_id })
        .await
        .map_err(|e| {
            tracing::warn!(error = %e, "failed to save github oauth state");
            AppError::Internal
        })?;

    state.github.install_url(&nonce).await.map_err(|e| {
        tracing::warn!(error = %e, "failed to build install url");
        AppError::Internal
    })
}
