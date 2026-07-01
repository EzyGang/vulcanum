use actix_web::body::to_bytes;
use actix_web::http::StatusCode;
use actix_web::ResponseError;

use crate::errors::AppError;
use crate::models::providers::errors::IntegrationError;
use crate::models::task_board::errors::TaskBoardError;
use crate::services::providers::kaneo::errors::KaneoError;

#[actix_web::test]
async fn task_board_provider_client_error_is_not_internal() {
    let err: AppError = TaskBoardError::Integration(IntegrationError::Kaneo(KaneoError::Api(
        "400 Bad Request: Label is not associated with a task".to_owned(),
    )))
    .into();

    let response = err.error_response();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = to_bytes(response.into_body()).await.expect("body reads");
    let payload: serde_json::Value = serde_json::from_slice(&body).expect("body is JSON");
    assert_eq!(payload["error"], "Label is not associated with a task");
}

#[actix_web::test]
async fn task_board_provider_server_error_stays_internal() {
    let err: AppError = TaskBoardError::Integration(IntegrationError::Kaneo(KaneoError::Api(
        "500 Internal Server Error: Failed to delete label".to_owned(),
    )))
    .into();

    assert_eq!(
        err.error_response().status(),
        StatusCode::INTERNAL_SERVER_ERROR
    );
}
