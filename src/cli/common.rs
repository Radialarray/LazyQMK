//! Common types and utilities for CLI commands.

use serde::Serialize;
use std::fmt;

/// CLI result type with proper exit codes.
pub type CliResult<T> = Result<T, CliError>;

/// Standard CLI exit codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExitCode {
    /// Success (0)
    Success = 0,
    /// Validation or user error (1)
    ValidationError = 1,
    /// I/O or system error (2)
    IoError = 2,
}

/// CLI error with exit code.
#[derive(Debug)]
pub struct CliError {
    /// Error message
    pub message: String,
    /// Exit code
    pub exit_code: ExitCode,
}

impl CliError {
    /// Creates a validation error (exit code 1).
    #[must_use]
    pub fn validation(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            exit_code: ExitCode::ValidationError,
        }
    }

    /// Creates an I/O error (exit code 2).
    #[must_use]
    pub fn io(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            exit_code: ExitCode::IoError,
        }
    }
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for CliError {}

impl From<anyhow::Error> for CliError {
    fn from(err: anyhow::Error) -> Self {
        // Convert anyhow errors to I/O errors by default
        Self::io(err.to_string())
    }
}

impl From<std::io::Error> for CliError {
    fn from(err: std::io::Error) -> Self {
        Self::io(err.to_string())
    }
}

/// JSON response for validation commands.
#[derive(Debug, Clone, Serialize)]
pub struct ValidationResponse {
    /// Whether validation passed (no errors)
    pub valid: bool,
    /// List of errors and warnings
    pub errors: Vec<ValidationMessage>,
    /// Validation check results
    pub checks: ValidationChecks,
}

/// Individual validation error or warning.
#[derive(Debug, Clone, Serialize)]
pub struct ValidationMessage {
    /// Severity: "error" or "warning"
    pub severity: String,
    /// Human-readable message
    pub message: String,
    /// Optional location context
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<ValidationLocation>,
}

/// Location context for a validation message.
#[derive(Debug, Clone, Serialize)]
pub struct ValidationLocation {
    /// Layer index
    pub layer: usize,
    /// Position in the layout
    pub position: ValidationPosition,
}

/// Position coordinates.
#[derive(Debug, Clone, Serialize)]
pub struct ValidationPosition {
    /// Row coordinate
    pub row: u8,
    /// Column coordinate
    pub col: u8,
}

/// Validation check results.
#[derive(Debug, Clone, Serialize)]
pub struct ValidationChecks {
    /// Keycode validation result
    pub keycodes: String,
    /// Position validation result
    pub positions: String,
    /// Layer reference validation result
    pub layer_refs: String,
    /// Tap dance validation result
    pub tap_dances: String,
}

impl ValidationChecks {
    /// Creates a new validation checks struct with all passed.
    #[must_use]
    pub fn all_passed() -> Self {
        Self {
            keycodes: "passed".to_string(),
            positions: "passed".to_string(),
            layer_refs: "passed".to_string(),
            tap_dances: "passed".to_string(),
        }
    }
}
