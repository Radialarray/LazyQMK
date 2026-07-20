//! API error type and conversion helpers.
//!
//! Extracted from src/web/mod.rs as part of LazyQMK-2rf6.2.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;

/// API error response.
#[derive(Debug, Serialize)]
pub struct ApiError {
    /// Error message.
    pub error: String,
    /// Optional additional details.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

impl ApiError {
    /// Creates a new API error with just an error message.
    pub(crate) fn new(error: impl Into<String>) -> Self {
        Self {
            error: error.into(),
            details: None,
        }
    }
}

/// HTTP-aware error wrapper. Carries a status code alongside the JSON body
/// and implements `IntoResponse` so handlers can simply return
/// `Result<Json<T>, AppError>`.
#[derive(Debug)]
pub struct AppError {
    /// HTTP status code for the error response.
    pub status: StatusCode,
    /// JSON-serializable error body.
    pub error: ApiError,
}

impl AppError {
    /// 400 Bad Request with the given message.
    pub fn bad_request(msg: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            error: ApiError::new(msg),
        }
    }

    /// 404 Not Found with the given message.
    pub fn not_found(msg: impl Into<String>) -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            error: ApiError::new(msg),
        }
    }

    /// 500 Internal Server Error with the given message.
    pub fn internal(msg: impl Into<String>) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            error: ApiError::new(msg),
        }
    }

    /// Builds an `AppError` with an explicit status code, message, and
    /// optional details string.
    pub fn with_details(
        status: StatusCode,
        msg: impl Into<String>,
        details: impl Into<Option<String>>,
    ) -> Self {
        Self {
            status,
            error: ApiError {
                error: msg.into(),
                details: details.into(),
            },
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (self.status, Json(self.error)).into_response()
    }
}

impl From<ApiError> for AppError {
    fn from(err: ApiError) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            error: err,
        }
    }
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        Self::with_details(
            StatusCode::INTERNAL_SERVER_ERROR,
            "I/O error",
            Some(err.to_string()),
        )
    }
}

impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        Self::with_details(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Internal error",
            Some(err.to_string()),
        )
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        Self::with_details(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Serialization error",
            Some(err.to_string()),
        )
    }
}
