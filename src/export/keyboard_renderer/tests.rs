//! Tests for export::keyboard_renderer.
//!
//! Auto-extracted from src/export/keyboard_renderer/mod.rs.
use super::formatting::format_keycode;
use super::rendering::detect_split_gap;
use super::*;
use crate::keycode_db::format::{format_modifier, strip_kc_prefix};
use crate::models::{
    keyboard_geometry::KeyGeometry,
    layer::{KeyDefinition, Layer, Position},
    RgbColor,
};

fn create_test_geometry() -> KeyboardGeometry {
    let mut geom = KeyboardGeometry::new("test", "LAYOUT", 4, 12);

    // Create a simple 3x3 grid layout
    // Use exact integer positions so visual coordinates match
    for row in 0u8..3 {
        for col in 0u8..3 {
            let key = KeyGeometry::new(
                (row, col),
                row * 3 + col,
                f32::from(col), // Use exact integers instead of 1.25 spacing
                f32::from(row),
            );
            geom.add_key(key);
        }
    }

    geom
}

fn create_test_layout() -> Layout {
    let mut layout = Layout::new("Test Layout").unwrap();

    let mut layer = Layer::new(0, "Base", RgbColor::new(255, 255, 255)).unwrap();

    // Add 3x3 grid of keys
    for row in 0..3 {
        for col in 0..3 {
            let keycode = format!("KC_{}", (b'A' + (row * 3 + col)) as char);
            layer.add_key(KeyDefinition::new(Position::new(row, col), &keycode));
        }
    }

    layout.add_layer(layer).unwrap();
    layout
}

#[test]
fn test_format_keycode_simple() {
    assert_eq!(format_keycode("KC_A"), "A");
    assert_eq!(format_keycode("KC_SPACE"), "SPACE");
    assert_eq!(format_keycode("KC_ENTER"), "ENTER");
}

#[test]
fn test_format_keycode_layer_tap() {
    assert_eq!(format_keycode("LT(1, KC_A)"), "L1 / A");
    assert_eq!(format_keycode("LT(@abc-123, KC_SPC)"), "Labc-123 / SPC");
}

#[test]
fn test_format_keycode_mod_tap() {
    assert_eq!(format_keycode("LCTL_T(KC_A)"), "CTL / A");
    assert_eq!(format_keycode("LSFT_T(KC_SPC)"), "SFT / SPC");
}

#[test]
fn test_format_keycode_layer_mod() {
    assert_eq!(format_keycode("LM(1, MOD_LCTL)"), "L1+C");
}

#[test]
fn test_format_keycode_momentary_layer() {
    assert_eq!(format_keycode("MO(1)"), "▼L1");
    assert_eq!(format_keycode("MO(@abc-123)"), "▼Labc-123");
}

#[test]
fn test_format_keycode_tap_dance() {
    assert_eq!(format_keycode("TD(quote_dance)"), "TD:quote_dance");
}

#[test]
fn test_format_modifier() {
    assert_eq!(format_modifier("MOD_LCTL"), "C");
    assert_eq!(format_modifier("MOD_LSFT"), "S");
    assert_eq!(format_modifier("MOD_LCTL | MOD_LSFT"), "CS");
    assert_eq!(
        format_modifier("MOD_LCTL | MOD_LSFT | MOD_LALT | MOD_LGUI"),
        "CSAG"
    );
}

#[test]
fn test_strip_kc_prefix() {
    assert_eq!(strip_kc_prefix("KC_A"), "A");
    assert_eq!(strip_kc_prefix("KC_SPACE"), "SPACE");
    assert_eq!(strip_kc_prefix("MO(1)"), "MO(1)");
}

#[test]
fn test_build_key_grid() {
    let layout = create_test_layout();
    let geometry = create_test_geometry();

    let grid = build_key_grid(&layout, 0, &geometry).unwrap();

    assert_eq!(grid.keys.len(), 9); // 3x3 grid
    assert!(grid.max_row >= 2);
    assert!(grid.max_col >= 2);
}

#[test]
fn test_render_layer_diagram() {
    let layout = create_test_layout();
    let geometry = create_test_geometry();

    let diagram = render_layer_diagram(&layout, 0, &geometry).unwrap();

    // Basic checks
    assert!(diagram.contains("Layer 0: Base"));
    assert!(diagram.contains('┌')); // Contains box drawing chars
    assert!(diagram.contains('│'));
    assert!(diagram.contains('─'));
}

#[test]
fn test_detect_split_gap_no_split() {
    let keys = vec![
        GridKey {
            label: "A".to_string(),
            color_ref: None,
            row: 0,
            col: 0,
            width: 1,
            height: 1,
        },
        GridKey {
            label: "B".to_string(),
            color_ref: None,
            row: 0,
            col: 1,
            width: 1,
            height: 1,
        },
        GridKey {
            label: "C".to_string(),
            color_ref: None,
            row: 0,
            col: 2,
            width: 1,
            height: 1,
        },
    ];

    assert_eq!(detect_split_gap(&keys, 2), None);
}

#[test]
fn test_detect_split_gap_with_split() {
    let keys = vec![
        GridKey {
            label: "A".to_string(),
            color_ref: None,
            row: 0,
            col: 0,
            width: 1,
            height: 1,
        },
        GridKey {
            label: "B".to_string(),
            color_ref: None,
            row: 0,
            col: 1,
            width: 1,
            height: 1,
        },
        // Gap at columns 2, 3, 4
        GridKey {
            label: "C".to_string(),
            color_ref: None,
            row: 0,
            col: 5,
            width: 1,
            height: 1,
        },
    ];

    assert_eq!(detect_split_gap(&keys, 5), Some(2));
}
