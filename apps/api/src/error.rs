//! A single error type for the whole HTTP layer.
//!
//! Every handler returns `Result<T, AppError>`. The `IntoResponse` impl maps
//! each variant to a status code + JSON body, so handlers never need to think
//! about response shaping for errors.

use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("{0}")]
    BadRequest(String),

    #[error("invalid email or password")]
    InvalidCredentials,

    #[error("email is already registered")]
    EmailTaken,

    #[error("authentication required")]
    Unauthorized,

    #[error("refresh token is invalid or expired")]
    InvalidRefreshToken,

    #[error("internal server error")]
    Internal(#[source] anyhow::Error),

    #[error("{0}")]
    NotFound(String),
}

impl AppError {
    fn status(&self) -> StatusCode {
        match self {
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            AppError::InvalidCredentials | AppError::Unauthorized => StatusCode::UNAUTHORIZED,
            AppError::InvalidRefreshToken => StatusCode::UNAUTHORIZED,
            AppError::EmailTaken => StatusCode::CONFLICT,
            AppError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        // Log internal errors with their full chain; never leak the detail to clients.
        if let AppError::Internal(err) = &self {
            tracing::error!(error = ?err, "internal server error");
        }

        let body = Json(json!({ "error": self.to_string() }));
        (self.status(), body).into_response()
    }
}

// Convenience conversions so handlers can use `?` on common errors.
impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        AppError::Internal(err.into())
    }
}

impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        AppError::Internal(err)
    }
}
