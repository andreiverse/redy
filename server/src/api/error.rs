use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Invalid input: {0}")]
    BadRequest(String),

    #[error("Internal server error: {0}")]
    Internal(String),

    #[error("Authentication failed: {0}")]
    Auth(String),

    #[error("Database error: {0}")]
    Database(#[from] sea_orm::DbErr),
}

#[derive(serde::Serialize)]
pub struct AppErrorResponse {
    message: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            AppError::Auth(msg) => (StatusCode::UNAUTHORIZED, msg),
            AppError::Database(_err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Database error".to_string(),
            ),
        };

        (status, Json(AppErrorResponse { message })).into_response()
    }
}
