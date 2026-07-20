//! API error type and conversion helpers.
//!
//! Extracted from src/web/mod.rs as part of LazyQMK-2rf6.2.

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

    /// Creates a new API error with both message and details.
    pub(crate) fn with_details(error: impl Into<String>, details: impl Into<String>) -> Self {
        Self {
            error: error.into(),
            details: Some(details.into()),
        }
    }
}