//! Validation helpers for filename and keyboard path.
//!
//! Extracted from src/web/mod.rs as part of LazyQMK-2rf6.2.

use super::error::ApiError;

/// Validates a filename to prevent path traversal attacks.
///
/// Returns the sanitized filename or an error if the filename is invalid.
pub(crate) fn validate_filename(filename: &str) -> Result<&str, ApiError> {
    // Reject empty filenames
    if filename.is_empty() {
        return Err(ApiError::new("Filename cannot be empty"));
    }

    // Reject path traversal attempts
    if filename.contains("..") || filename.contains('/') || filename.contains('\\') {
        return Err(ApiError::new(
            "Invalid filename: path traversal not allowed",
        ));
    }

    // Reject absolute paths
    if filename.starts_with('/') || filename.starts_with('\\') {
        return Err(ApiError::new(
            "Invalid filename: absolute paths not allowed",
        ));
    }

    // Reject hidden files
    if filename.starts_with('.') {
        return Err(ApiError::new("Invalid filename: hidden files not allowed"));
    }

    Ok(filename)
}

/// Appends or changes the filename extension to `.json`.
pub(crate) fn with_json_ext(filename: &str) -> String {
    let path = std::path::Path::new(filename);
    if path
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("json"))
    {
        filename.to_string()
    } else {
        format!("{filename}.json")
    }
}

/// Validates a keyboard path to prevent path traversal attacks.
pub(crate) fn validate_keyboard_path(keyboard: &str) -> Result<(), ApiError> {
    if keyboard.is_empty() {
        return Err(ApiError::new("Keyboard path cannot be empty"));
    }

    // Reject path traversal attempts
    if keyboard.contains("..") {
        return Err(ApiError::new(
            "Invalid keyboard path: path traversal not allowed",
        ));
    }

    // Reject absolute paths
    if keyboard.starts_with('/') || keyboard.starts_with('\\') {
        return Err(ApiError::new(
            "Invalid keyboard path: absolute paths not allowed",
        ));
    }

    Ok(())
}