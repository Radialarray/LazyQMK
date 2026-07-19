//! `FirmwareValidator` — runs the actual validation checks.

use crate::keycode_db::KeycodeDb;
use crate::models::keyboard_geometry::KeyboardGeometry;
use crate::models::layout::Layout;
use crate::models::visual_layout_mapping::VisualLayoutMapping;
use anyhow::Result;
use std::collections::HashSet;

use super::report::{ValidationError, ValidationErrorKind, ValidationReport, ValidationWarning};

/// Firmware validator.
pub struct FirmwareValidator<'a> {
    layout: &'a Layout,
    geometry: &'a KeyboardGeometry,
    mapping: &'a VisualLayoutMapping,
    keycode_db: &'a KeycodeDb,
}

impl<'a> FirmwareValidator<'a> {
    /// Creates a new firmware validator.
    #[must_use]
    pub const fn new(
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
    #[allow(clippy::unnecessary_wraps)]
    pub fn validate(&self) -> Result<ValidationReport> {
        let mut report = ValidationReport::new();

        // Validate layout structure first
        if let Err(e) = self.layout.validate() {
            let error_msg = e.to_string();
            // Determine error kind based on the error message
            let kind = if error_msg.contains("Duplicate position") {
                ValidationErrorKind::DuplicatePosition
            } else if error_msg.contains("at least one layer") {
                ValidationErrorKind::EmptyLayer
            } else if error_msg.contains("must have the same number of keys")
                && error_msg.contains("has 0,")
            {
                // Empty layer will show as "has 0, expected N"
                ValidationErrorKind::EmptyLayer
            } else {
                // Default to MismatchedKeyCount for other structural issues
                ValidationErrorKind::MismatchedKeyCount
            };

            report.add_error(
                ValidationError::new(kind, format!("Layout validation failed: {e}"))
                    .with_suggestion(
                        "Check that all layers have keys and no gaps in layer numbers",
                    ),
            );
            return Ok(report);
        }

        // Validate each layer
        for (layer_idx, layer) in self.layout.layers.iter().enumerate() {
            self.validate_layer(&mut report, layer_idx, layer);
        }

        // Check matrix coverage
        self.validate_matrix_coverage(&mut report);

        // Check for orphaned tap dances
        self.validate_tap_dances(&mut report);

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

    /// Validates a single keycode at a **visual** position.
    ///
    /// `row` and `col` are visual-grid coordinates used only for error reporting;
    /// the validation itself is purely keycode-based.
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
            let suggestion_text = if suggestions.is_empty() {
                "Check the keycode database for valid codes".to_string()
            } else {
                let similar: Vec<&str> = suggestions
                    .iter()
                    .take(3)
                    .map(|k| k.code.as_str())
                    .collect();
                format!("Did you mean one of: {}", similar.join(", "))
            };

            report.add_error(
                ValidationError::new(
                    ValidationErrorKind::InvalidKeycode,
                    format!("Invalid keycode '{keycode}'"),
                )
                .with_layer(layer)
                .with_position(row, col)
                .with_suggestion(suggestion_text),
            );
        }
    }

    /// Validates that a **visual** position `(row, col)` maps to a valid matrix coordinate.
    ///
    /// `row` and `col` are visual-grid coordinates. Internally converts visual → matrix
    /// via `VisualLayoutMapping` and checks matrix bounds against `KeyboardGeometry`.
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
                    format!("Position ({row}, {col}) does not map to any matrix position"),
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
                "Only {actual_positions} of {expected_positions} positions are defined in the layout. Some keys may be missing."
            )));
        }
    }

    /// Validates tap dance definitions.
    fn validate_tap_dances(&self, report: &mut ValidationReport) {
        // Check for orphaned tap dances (defined but never used)
        let orphaned = self.layout.get_orphaned_tap_dances();
        for td_name in orphaned {
            report.add_warning(ValidationWarning::new(format!(
                "Tap dance '{}' is defined but never used in any layer",
                td_name
            )));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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

    #[test]
    fn test_orphaned_tap_dance_warning() {
        use crate::models::layout::TapDanceAction;

        let (mut layout, geometry, mapping, keycode_db) = create_test_setup();

        // Add a tap dance but don't use it anywhere
        let mut tap_dance = TapDanceAction::new("unused_td", "KC_A");
        tap_dance = tap_dance.with_double_tap("KC_B");
        layout.add_tap_dance(tap_dance).unwrap();

        let validator = FirmwareValidator::new(&layout, &geometry, &mapping, &keycode_db);
        let report = validator.validate().unwrap();

        // Should be valid (warnings don't block validation)
        assert!(report.is_valid());
        // Should have exactly one warning about the orphaned tap dance
        assert_eq!(report.warnings.len(), 1);
        assert!(report.warnings[0]
            .message
            .contains("Tap dance 'unused_td' is defined but never used"));
    }

    #[test]
    fn test_used_tap_dance_no_warning() {
        use crate::models::layout::TapDanceAction;

        let (mut layout, geometry, mapping, keycode_db) = create_test_setup();

        // Add a tap dance and use it
        let mut tap_dance = TapDanceAction::new("used_td", "KC_A");
        tap_dance = tap_dance.with_double_tap("KC_B");
        layout.add_tap_dance(tap_dance).unwrap();

        // Update a key to use the tap dance
        layout.layers[0].keys[0].keycode = "TD(used_td)".to_string();

        let validator = FirmwareValidator::new(&layout, &geometry, &mapping, &keycode_db);
        let report = validator.validate().unwrap();

        // Should be valid with no warnings about tap dance
        assert!(report.is_valid());
        // Should not have any tap dance warnings
        assert!(!report
            .warnings
            .iter()
            .any(|w| w.message.contains("Tap dance")));
    }
}
