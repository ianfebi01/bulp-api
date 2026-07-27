use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

/// Application-level error type. Every handler returns `Result<_, AppError>`.
///
/// `IntoResponse` renders a consistent JSON error envelope:
/// `{ "success": false, "error": { "code": ..., "message": ... } }`.
/// Internal failures are logged server-side and never leaked to the client.
#[derive(Debug)]
pub enum AppError {
    /// Requested resource does not exist → 404.
    NotFound(String),
    /// Client sent invalid input → 400.
    BadRequest(String),
    /// Something failed on our side → 500. The detail is logged, never returned.
    Internal(String),
}

impl From<deadpool_postgres::PoolError> for AppError {
    fn from(e: deadpool_postgres::PoolError) -> Self {
        AppError::Internal(format!("pool error: {e}"))
    }
}

impl From<tokio_postgres::Error> for AppError {
    fn from(e: tokio_postgres::Error) -> Self {
        AppError::Internal(format!("db error: {e}"))
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code, message) = match self {
            AppError::NotFound(m) => (StatusCode::NOT_FOUND, "not_found", m),
            AppError::BadRequest(m) => (StatusCode::BAD_REQUEST, "bad_request", m),
            AppError::Internal(detail) => {
                // Log the real cause; return a generic message to the client.
                tracing::error!("internal error: {detail}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal_error",
                    "internal server error".to_string(),
                )
            }
        };

        let body = Json(json!({
            "success": false,
            "error": { "code": code, "message": message },
        }));

        (status, body).into_response()
    }
}
