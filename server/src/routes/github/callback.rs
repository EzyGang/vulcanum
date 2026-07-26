use super::{AppError, AppState, HttpResponse};

pub(super) async fn oauth_callback(
    state: &AppState,
    code: &str,
    state_nonce: &str,
    installation_id: Option<i64>,
) -> Result<HttpResponse, AppError> {
    match state
        .github
        .verify_and_consume_state_nonce(state_nonce)
        .await?
    {
        Some(install_state) => {
            let location = state
                .auth
                .complete_github_installation_authorization(
                    &state.github,
                    install_state,
                    code,
                    installation_id,
                )
                .await?;
            Ok(HttpResponse::Found()
                .append_header(("Location", location))
                .finish())
        }
        None => crate::routes::auth::complete_github_callback(state, code, state_nonce).await,
    }
}

pub(super) fn invalid_callback_response() -> HttpResponse {
    HttpResponse::BadRequest()
        .content_type("text/plain; charset=utf-8")
        .body(
            "Invalid GitHub callback. Configure the GitHub App Callback URL and \
             GITHUB_OAUTH_REDIRECT_URL as /api/v1/github/callback.",
        )
}
