//! Validation Tests
//!
//! Tests that the FirmwareValidator correctly validates layouts
//! for firmware generation.

use super::helpers::*;
use lazyqmk::firmware::FirmwareValidator;
use lazyqmk::keycode_db::KeycodeDb;

#[test]
fn test_validation_valid_layout() {
    // Arrange
    let layout = create_test_layout();
    let geometry = create_test_geometry();
    let mapping = create_test_mapping();
    let keycode_db = KeycodeDb::load().expect("Failed to load keycode database");

    // Act
    let validator = FirmwareValidator::new(&layout, &geometry, &mapping, &keycode_db);
    let report = validator.validate().expect("Validation failed");

    // Assert
    assert!(report.is_valid(), "Valid layout should pass validation");
    assert!(report.errors.is_empty(), "Should have no validation errors");
}

#[test]
fn test_validation_invalid_keycode() {
    // Arrange
    let mut layout = create_test_layout();
    layout.layers[0].keys[0].keycode = "INVALID_KEYCODE_XYZ".to_string();

    let geometry = create_test_geometry();
    let mapping = create_test_mapping();
    let keycode_db = KeycodeDb::load().expect("Failed to load keycode database");

    // Act
    let validator = FirmwareValidator::new(&layout, &geometry, &mapping, &keycode_db);
    let report = validator.validate().expect("Validation failed");

    // Assert
    assert!(
        !report.is_valid(),
        "Layout with invalid keycode should fail validation"
    );
    assert!(!report.errors.is_empty(), "Should have validation errors");
}

#[test]
fn test_validation_missing_position() {
    // Arrange
    let mut layout = create_test_layout();
    // Remove a key, creating a gap in positions
    layout.layers[0].keys.remove(2);

    let geometry = create_test_geometry();
    let mapping = create_test_mapping();
    let keycode_db = KeycodeDb::load().expect("Failed to load keycode database");

    // Act
    let validator = FirmwareValidator::new(&layout, &geometry, &mapping, &keycode_db);
    let report = validator.validate().expect("Validation failed");

    // Assert
    assert!(
        !report.is_valid(),
        "Layout with missing position should fail validation"
    );
}
