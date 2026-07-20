//! Tests for geometry.
//!
//! Auto-extracted from geometry.rs.

use super::*;

use super::*;

#[test]
fn test_extract_base_keyboard() {
    // With standard variant names
    assert_eq!(
        extract_base_keyboard("keebart/corne_choc_pro/standard"),
        "keebart/corne_choc_pro"
    );

    assert_eq!(
        extract_base_keyboard("manufacturer/keyboard/mini"),
        "manufacturer/keyboard"
    );

    // With revision variants
    assert_eq!(
        extract_base_keyboard("manufacturer/keyboard/rev1"),
        "manufacturer/keyboard"
    );

    assert_eq!(
        extract_base_keyboard("manufacturer/keyboard/rev2"),
        "manufacturer/keyboard"
    );

    // With version variants
    assert_eq!(
        extract_base_keyboard("manufacturer/keyboard/v1"),
        "manufacturer/keyboard"
    );

    assert_eq!(
        extract_base_keyboard("manufacturer/keyboard/v2"),
        "manufacturer/keyboard"
    );

    // With other common variant names
    assert_eq!(
        extract_base_keyboard("manufacturer/keyboard/rgb"),
        "manufacturer/keyboard"
    );

    assert_eq!(
        extract_base_keyboard("manufacturer/keyboard/wireless"),
        "manufacturer/keyboard"
    );

    assert_eq!(
        extract_base_keyboard("manufacturer/keyboard/ansi"),
        "manufacturer/keyboard"
    );

    assert_eq!(
        extract_base_keyboard("manufacturer/keyboard/iso"),
        "manufacturer/keyboard"
    );

    assert_eq!(
        extract_base_keyboard("manufacturer/keyboard/hotswap"),
        "manufacturer/keyboard"
    );

    // Without variant subdirectory
    assert_eq!(
        extract_base_keyboard("keebart/corne_choc_pro"),
        "keebart/corne_choc_pro"
    );

    assert_eq!(extract_base_keyboard("crkbd"), "crkbd");

    // Edge case: single directory with variant name (not enough path components)
    assert_eq!(extract_base_keyboard("standard"), "standard");
    assert_eq!(extract_base_keyboard("rev1"), "rev1");

    // Edge case: non-variant subdirectory (should not be stripped)
    assert_eq!(
        extract_base_keyboard("manufacturer/keyboard/custom"),
        "manufacturer/keyboard/custom"
    );
}

#[test]
fn test_build_minimal_geometry() {
    let result = build_minimal_geometry();
    assert!(result.geometry.keys.is_empty());
    assert_eq!(result.geometry.keyboard_name, "unknown");
    assert_eq!(result.geometry.layout_name, "LAYOUT");
    assert_eq!(result.geometry.matrix_rows, 0);
    assert_eq!(result.geometry.matrix_cols, 0);
    assert_eq!(result.variant_path, "");
}
