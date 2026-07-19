//! Validation report types: `ValidationReport`, `ValidationError`,
//! `ValidationWarning`, plus their `Display` impls.

// Allow format! appended to String - more readable for building messages
#![allow(clippy::format_push_string)]

/// Validation result with specific errors and warnings.
#[derive(Debug, Clone)]
pub struct ValidationReport {
    /// Critical errors that prevent firmware generation
    pub errors: Vec<ValidationError>,
    /// Non-critical warnings
    pub warnings: Vec<ValidationWarning>,
}

impl ValidationReport {
    /// Creates a new empty validation report.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    /// Returns true if there are no errors (warnings are allowed).
    #[must_use]
    pub const fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }

    /// Adds an error to the report.
    pub fn add_error(&mut self, error: ValidationError) {
        self.errors.push(error);
    }

    /// Adds a warning to the report.
    pub fn add_warning(&mut self, warning: ValidationWarning) {
        self.warnings.push(warning);
    }

    /// Formats the report as a user-friendly error message.
    #[must_use]
    pub fn format_message(&self) -> String {
        let mut message = String::new();

        if !self.errors.is_empty() {
            message.push_str(&format!("❌ {} validation errors:\n", self.errors.len()));
            for (idx, error) in self.errors.iter().enumerate() {
                message.push_str(&format!("  {}. {}\n", idx + 1, error));
            }
        }

        if !self.warnings.is_empty() {
            message.push_str(&format!("\n⚠️  {} warnings:\n", self.warnings.len()));
            for (idx, warning) in self.warnings.iter().enumerate() {
                message.push_str(&format!("  {}. {}\n", idx + 1, warning));
            }
        }

        message
    }
}

impl Default for ValidationReport {
    fn default() -> Self {
        Self::new()
    }
}

/// Validation error with context.
#[derive(Debug, Clone)]
pub struct ValidationError {
    /// Type of validation error
    pub kind: ValidationErrorKind,
    /// Layer index where error occurred
    pub layer: Option<usize>,
    /// Row in matrix where error occurred
    pub row: Option<u8>,
    /// Column in matrix where error occurred
    pub col: Option<u8>,
    /// Human-readable error message
    pub message: String,
    /// Optional suggestion for fixing the error
    pub suggestion: Option<String>,
}

impl ValidationError {
    /// Creates a new validation error.
    pub fn new(kind: ValidationErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            layer: None,
            row: None,
            col: None,
            message: message.into(),
            suggestion: None,
        }
    }

    /// Sets the layer context.
    #[must_use]
    pub const fn with_layer(mut self, layer: usize) -> Self {
        self.layer = Some(layer);
        self
    }

    /// Sets the **visual** position context for this error.
    ///
    /// `row` and `col` are visual-grid coordinates (matching `KeyDefinition.position`).
    #[must_use]
    pub const fn with_position(mut self, row: u8, col: u8) -> Self {
        self.row = Some(row);
        self.col = Some(col);
        self
    }

    /// Sets a suggestion for fixing the error.
    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestion = Some(suggestion.into());
        self
    }
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let (Some(layer), Some(row), Some(col)) = (self.layer, self.row, self.col) {
            write!(
                f,
                "[Layer {} ({}, {})] {}: {}",
                layer, row, col, self.kind, self.message
            )?;
        } else if let Some(layer) = self.layer {
            write!(f, "[Layer {}] {}: {}", layer, self.kind, self.message)?;
        } else {
            write!(f, "{}: {}", self.kind, self.message)?;
        }

        if let Some(suggestion) = &self.suggestion {
            write!(f, "\n    → {suggestion}")?;
        }

        Ok(())
    }
}

/// Types of validation errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationErrorKind {
    /// Keycode not recognized by QMK
    InvalidKeycode,
    /// Key definition missing matrix position
    MissingPosition,
    /// Multiple keys assigned to same matrix position
    DuplicatePosition,
    /// Matrix position exceeds keyboard geometry bounds
    MatrixOutOfBounds,
    /// Layer contains no key definitions
    EmptyLayer,
    /// Number of keys doesn't match keyboard geometry
    MismatchedKeyCount,
}

impl std::fmt::Display for ValidationErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidKeycode => write!(f, "Invalid Keycode"),
            Self::MissingPosition => write!(f, "Missing Position"),
            Self::DuplicatePosition => write!(f, "Duplicate Position"),
            Self::MatrixOutOfBounds => write!(f, "Matrix Out of Bounds"),
            Self::EmptyLayer => write!(f, "Empty Layer"),
            Self::MismatchedKeyCount => write!(f, "Mismatched Key Count"),
        }
    }
}

/// Validation warning (non-blocking).
#[derive(Debug, Clone)]
pub struct ValidationWarning {
    /// Warning message
    pub message: String,
}

impl ValidationWarning {
    /// Creates a new validation warning
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl std::fmt::Display for ValidationWarning {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}
