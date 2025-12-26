use crate::domain::DomainError;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;

#[derive(Debug)]
pub enum AppError {
    Domain(DomainError),
    Validation(String),
    BadRequest(String),
    Unauthorized(String),
    Forbidden(String),
    NotFound(String),
    Conflict(String),
    Internal(String),
}

#[derive(Serialize)]
struct ErrorBody {
    message: String,
}

impl From<DomainError> for AppError {
    fn from(value: DomainError) -> Self {
        match value {
            DomainError::ValidationError(message) => AppError::Validation(message),
            DomainError::NotFound(message) => AppError::NotFound(message),
            DomainError::Unauthorized(message) => AppError::Unauthorized(message),
            DomainError::Forbidden(message) => AppError::Forbidden(message),
            DomainError::Conflict(message) => AppError::Conflict(message),
            DomainError::Internal(message) => AppError::Internal(message),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::Validation(message) => (StatusCode::BAD_REQUEST, message),
            AppError::BadRequest(message) => (StatusCode::BAD_REQUEST, message),
            AppError::Unauthorized(message) => (StatusCode::UNAUTHORIZED, message),
            AppError::Forbidden(message) => (StatusCode::FORBIDDEN, message),
            AppError::NotFound(message) => (StatusCode::NOT_FOUND, message),
            AppError::Conflict(message) => (StatusCode::CONFLICT, message),
            AppError::Internal(message) => (StatusCode::INTERNAL_SERVER_ERROR, message),
            AppError::Domain(domain) => (StatusCode::INTERNAL_SERVER_ERROR, domain.to_string()),
        };

        let body = axum::Json(ErrorBody { message });
        (status, body).into_response()
    }
}