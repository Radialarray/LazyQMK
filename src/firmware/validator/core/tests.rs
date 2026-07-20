//! Tests for core.
//!
//! Auto-extracted from core.rs.

use super::*;

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
