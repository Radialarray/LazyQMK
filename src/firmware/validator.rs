//! Firmware validation before generation.
//!
//! This module performs pre-generation validation to ensure the layout
//! can be successfully compiled into QMK firmware.

use crate::keycode_db::KeycodeDb;
use crate::models::keyboard_geometry::KeyboardGeometry;
use crate::models::layout::Layout;
use crate::models::visual_layout_mapping::VisualLayoutMapping;
use anyhow::{Context, Result};
use std::collections::HashSet;

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
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    /// Returns true if there are no errors (warnings are allowed).
    pub fn is_valid(&self) -> bool {
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
    pub kind: ValidationErrorKind,
    pub layer: Option<usize>,
    pub row: Option<u8>,
    pub col: Option<u8>,
    pub message: String,
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
    pub fn with_layer(mut self, layer: usize) -> Self {
        self.layer = Some(layer);
        self
    }

    /// Sets the position context.
    pub fn with_position(mut self, row: u8, col: u8) -> Self {
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
            write!(f, "\n    → {}", suggestion)?;
        }

        Ok(())
    }
}

/// Types of validation errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationErrorKind {
    InvalidKeycode,
    MissingPosition,
    DuplicatePosition,
    MatrixOutOfBounds,
    EmptyLayer,
    MismatchedKeyCount,
}

impl std::fmt::Display for ValidationErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationErrorKind::InvalidKeycode => write!(f, "Invalid Keycode"),
            ValidationErrorKind::MissingPosition => write!(f, "Missing Position"),
            ValidationErrorKind::DuplicatePosition => write!(f, "Duplicate Position"),
            ValidationErrorKind::MatrixOutOfBounds => write!(f, "Matrix Out of Bounds"),
            ValidationErrorKind::EmptyLayer => write!(f, "Empty Layer"),
            ValidationErrorKind::MismatchedKeyCount => write!(f, "Mismatched Key Count"),
        }
    }
}

/// Validation warning (non-blocking).
#[derive(Debug, Clone)]
pub struct ValidationWarning {
    pub message: String,
}

impl ValidationWarning {
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

/// Firmware validator.
pub struct FirmwareValidator<'a> {
    layout: &'a Layout,
    geometry: &'a KeyboardGeometry,
    mapping: &'a VisualLayoutMapping,
    keycode_db: &'a KeycodeDb,
}

impl<'a> FirmwareValidator<'a> {
    /// Creates a new firmware validator.
    pub fn new(
        layout: &'a Layout,
        geometry: &'a KeyboardGeometry,
        mapping: &'a VisualLayoutMapping,
        keycode_db: &'a KeycodeDb,
    ) -> Self {
        Self {
            layout,
            geometry,
            mapping,
            keycode_db,
        }
    }

    /// Validates the layout for firmware generation.
    ///
    /// Checks:
    /// - All keycodes are valid
    /// - All positions map to matrix coordinates
    /// - Matrix coordinates are within keyboard bounds
    /// - All required positions are present
    /// - No duplicate positions per layer
    pub fn validate(&self) -> Result<ValidationReport> {
        let mut report = ValidationReport::new();

        // Validate layout structure first
        if let Err(e) = self.layout.validate() {
            report.add_error(
                ValidationError::new(
                    ValidationErrorKind::EmptyLayer,
                    format!("Layout validation failed: {}", e),
                )
                .with_suggestion("Check that all layers have keys and no gaps in layer numbers"),
            );
            return Ok(report);
        }

        // Validate each layer
        for (layer_idx, layer) in self.layout.layers.iter().enumerate() {
            self.validate_layer(&mut report, layer_idx, layer);
        }

        // Check matrix coverage
        self.validate_matrix_coverage(&mut report);

        Ok(report)
    }

    /// Validates a single layer.
    fn validate_layer(
        &self,
        report: &mut ValidationReport,
        layer_idx: usize,
        layer: &crate::models::layer::Layer,
    ) {
        if layer.keys.is_empty() {
            report.add_error(
                ValidationError::new(
                    ValidationErrorKind::EmptyLayer,
                    format!("Layer {} '{}' has no keys", layer_idx, layer.name),
                )
                .with_layer(layer_idx)
                .with_suggestion("Add keys to the layer or remove it"),
            );
            return;
        }

        // Check for expected key count
        let expected_count = self.mapping.key_count();
        if layer.keys.len() != expected_count {
            report.add_error(
                ValidationError::new(
                    ValidationErrorKind::MismatchedKeyCount,
                    format!(
                        "Layer {} has {} keys, expected {} for keyboard layout {}",
                        layer_idx,
                        layer.keys.len(),
                        expected_count,
                        self.geometry.layout_name
                    ),
                )
                .with_layer(layer_idx)
                .with_suggestion(format!(
                    "Add or remove keys to match the {} layout",
                    self.geometry.layout_name
                )),
            );
        }

        // Track seen positions to detect duplicates
        let mut seen_positions = HashSet::new();

        for key in &layer.keys {
            let pos = key.position;

            // Check for duplicate positions
            if !seen_positions.insert(pos) {
                report.add_error(
                    ValidationError::new(
                        ValidationErrorKind::DuplicatePosition,
                        format!("Position ({}, {}) appears multiple times", pos.row, pos.col),
                    )
                    .with_layer(layer_idx)
                    .with_position(pos.row, pos.col)
                    .with_suggestion("Remove duplicate key definitions"),
                );
            }

            // Validate keycode
            self.validate_keycode(report, layer_idx, pos.row, pos.col, &key.keycode);

            // Validate position mapping
            self.validate_position_mapping(report, layer_idx, pos.row, pos.col);
        }
    }

    /// Validates a single keycode.
    fn validate_keycode(
        &self,
        report: &mut ValidationReport,
        layer: usize,
        row: u8,
        col: u8,
        keycode: &str,
    ) {
        if !self.keycode_db.is_valid(keycode) {
            // Try to find similar keycodes for suggestion
            let suggestions = self.keycode_db.search(keycode);
            let suggestion_text = if !suggestions.is_empty() {
                let similar: Vec<&str> = suggestions
                    .iter()
                    .take(3)
                    .map(|k| k.code.as_str())
                    .collect();
                format!("Did you mean one of: {}", similar.join(", "))
            } else {
                "Check the keycode database for valid codes".to_string()
            };

            report.add_error(
                ValidationError::new(
                    ValidationErrorKind::InvalidKeycode,
                    format!("Invalid keycode '{}'", keycode),
                )
                .with_layer(layer)
                .with_position(row, col)
                .with_suggestion(suggestion_text),
            );
        }
    }

    /// Validates position mapping to matrix coordinates.
    fn validate_position_mapping(
        &self,
        report: &mut ValidationReport,
        layer: usize,
        row: u8,
        col: u8,
    ) {
        // Check if visual position maps to matrix
        if let Some(matrix_pos) = self.mapping.visual_to_matrix_pos(row, col) {
            // Verify matrix position is within keyboard bounds
            if matrix_pos.0 >= self.geometry.matrix_rows
                || matrix_pos.1 >= self.geometry.matrix_cols
            {
                report.add_error(
                    ValidationError::new(
                        ValidationErrorKind::MatrixOutOfBounds,
                        format!(
                            "Position ({}, {}) maps to matrix ({}, {}) which is out of bounds ({}×{})",
                            row, col, matrix_pos.0, matrix_pos.1,
                            self.geometry.matrix_rows, self.geometry.matrix_cols
                        ),
                    )
                    .with_layer(layer)
                    .with_position(row, col)
                    .with_suggestion("Check keyboard geometry configuration"),
                );
            }
        } else {
            report.add_error(
                ValidationError::new(
                    ValidationErrorKind::MissingPosition,
                    format!(
                        "Position ({}, {}) does not map to any matrix position",
                        row, col
                    ),
                )
                .with_layer(layer)
                .with_position(row, col)
                .with_suggestion("This position is not defined in the keyboard layout"),
            );
        }
    }

    /// Validates that all matrix positions are covered.
    fn validate_matrix_coverage(&self, report: &mut ValidationReport) {
        // This is a warning, not an error, because some keyboards
        // may have unused matrix positions
        let first_layer_positions: HashSet<_> = self.layout.layers[0]
            .keys
            .iter()
            .map(|k| k.position)
            .collect();

        let expected_positions = self.mapping.key_count();
        let actual_positions = first_layer_positions.len();

        if actual_positions < expected_positions {
            report.add_warning(ValidationWarning::new(format!(
                "Only {} of {} positions are defined in the layout. Some keys may be missing.",
                actual_positions, expected_positions
            )));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::category::Category;
    use crate::models::keyboard_geometry::KeyGeometry;
    use crate::models::layer::{KeyDefinition, Layer, Position};
    use crate::models::RgbColor;

    fn create_test_setup() -> (Layout, KeyboardGeometry, VisualLayoutMapping, KeycodeDb) {
        let mut layout = Layout::new("Test").unwrap();
        let mut layer = Layer::new(0, "Base", RgbColor::new(255, 255, 255)).unwrap();
        layer.add_key(KeyDefinition::new(Position::new(0, 0), "KC_A"));
        layer.add_key(KeyDefinition::new(Position::new(0, 1), "KC_B"));
        layout.add_layer(layer).unwrap();

        let mut geometry = KeyboardGeometry::new("test", "LAYOUT", 2, 2);
        geometry.add_key(KeyGeometry::new((0, 0), 0, 0.0, 0.0));
        geometry.add_key(KeyGeometry::new((0, 1), 1, 1.0, 0.0));

        let mapping = VisualLayoutMapping::build(&geometry);
        let keycode_db = KeycodeDb::load().unwrap();

        (layout, geometry, mapping, keycode_db)
    }

    #[test]
    fn test_valid_layout() {
        let (layout, geometry, mapping, keycode_db) = create_test_setup();
        let validator = FirmwareValidator::new(&layout, &geometry, &mapping, &keycode_db);
        let report = validator.validate().unwrap();

        assert!(report.is_valid(), "Layout should be valid");
        assert!(report.errors.is_empty());
    }

    #[test]
    fn test_invalid_keycode() {
        let (mut layout, geometry, mapping, keycode_db) = create_test_setup();
        let mut layer = Layer::new(1, "Invalid", RgbColor::new(255, 0, 0)).unwrap();
        layer.add_key(KeyDefinition::new(Position::new(0, 0), "INVALID_KEY"));
        layer.add_key(KeyDefinition::new(Position::new(0, 1), "KC_B"));
        layout.add_layer(layer).unwrap();

        let validator = FirmwareValidator::new(&layout, &geometry, &mapping, &keycode_db);
        let report = validator.validate().unwrap();

        assert!(!report.is_valid());
        assert_eq!(report.errors.len(), 1);
        assert_eq!(report.errors[0].kind, ValidationErrorKind::InvalidKeycode);
    }

    #[test]
    fn test_duplicate_position() {
        let (mut layout, geometry, mapping, keycode_db) = create_test_setup();
        let mut layer = Layer::new(1, "Duplicate", RgbColor::new(255, 0, 0)).unwrap();
        layer.add_key(KeyDefinition::new(Position::new(0, 0), "KC_A"));
        layer.add_key(KeyDefinition::new(Position::new(0, 0), "KC_B")); // Duplicate
        layout.add_layer(layer).unwrap();

        let validator = FirmwareValidator::new(&layout, &geometry, &mapping, &keycode_db);
        let report = validator.validate().unwrap();

        assert!(!report.is_valid());
        assert!(report
            .errors
            .iter()
            .any(|e| e.kind == ValidationErrorKind::DuplicatePosition));
    }

    #[test]
    fn test_empty_layer() {
        let (mut layout, geometry, mapping, keycode_db) = create_test_setup();
        let layer = Layer::new(1, "Empty", RgbColor::new(255, 0, 0)).unwrap();
        layout.add_layer(layer).unwrap();

        let validator = FirmwareValidator::new(&layout, &geometry, &mapping, &keycode_db);
        let report = validator.validate().unwrap();

        assert!(!report.is_valid());
        assert!(report
            .errors
            .iter()
            .any(|e| e.kind == ValidationErrorKind::EmptyLayer));
    }

    #[test]
    fn test_validation_report_format() {
        let mut report = ValidationReport::new();
        report.add_error(
            ValidationError::new(ValidationErrorKind::InvalidKeycode, "Test error")
                .with_layer(0)
                .with_position(0, 0)
                .with_suggestion("Fix the keycode"),
        );
        report.add_warning(ValidationWarning::new("Test warning"));

        let message = report.format_message();
        assert!(message.contains("1 validation errors"));
        assert!(message.contains("1 warnings"));
        assert!(message.contains("Test error"));
        assert!(message.contains("Test warning"));
    }
}
