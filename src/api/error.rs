use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Invalid input")]
    BadRequest,

    #[error("Internal server error")]
    Internal,
}

#[derive(Clone, Copy, serde::Serialize)]
pub struct AppErrorResponse<'a> {
    message: &'a str,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = match self {
            AppError::BadRequest => StatusCode::BAD_REQUEST,
            AppError::Internal => StatusCode::INTERNAL_SERVER_ERROR,
        };

        (
            status,
            Json(AppErrorResponse {
                message: &self.to_string(),
            }),
        )
            .into_response()
    }
}
