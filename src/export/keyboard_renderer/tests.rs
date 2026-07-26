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

    let grid = build_key_grid(&layout, 0, &geometry, std::collections::HashMap::new()).unwrap();

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
            visual_pos: (0, 0),
            row: 0,
            col: 0,
            width: 1,
            height: 1,
        },
        GridKey {
            label: "B".to_string(),
            color_ref: None,
            visual_pos: (0, 1),
            row: 0,
            col: 1,
            width: 1,
            height: 1,
        },
        GridKey {
            label: "C".to_string(),
            color_ref: None,
            visual_pos: (0, 2),
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
            visual_pos: (0, 0),
            row: 0,
            col: 0,
            width: 1,
            height: 1,
        },
        GridKey {
            label: "B".to_string(),
            color_ref: None,
            visual_pos: (0, 1),
            row: 0,
            col: 1,
            width: 1,
            height: 1,
        },
        // Gap at columns 2, 3, 4
        GridKey {
            label: "C".to_string(),
            color_ref: None,
            visual_pos: (0, 5),
            row: 0,
            col: 5,
            width: 1,
            height: 1,
        },
    ];

    assert_eq!(detect_split_gap(&keys, 5), Some(2));
}

#[test]
fn test_render_layer_diagram_with_marker_places_marker_in_top_right_slot() {
    let layout = create_test_layout();
    let geometry = create_test_geometry();

    // Use a marker char that is unlikely to appear in any keycode label.
    let mut markers = std::collections::HashMap::new();
    markers.insert((1, 1), '★');

    let diagram =
        render_layer_diagram_with_markers(&layout, 0, &geometry, markers, None).unwrap();

    // The marker is drawn on the top border line of the key box, as the last
    // `─` before `┐` of that specific box. Find a top border line containing
    // the marker (skipping any content lines that might contain the same
    // char) and verify the chars immediately around it are `─` (left) and
    // `┐` (right).
    let lines: Vec<&str> = diagram.lines().collect();
    let target_line = lines
        .iter()
        .find(|line| line.starts_with('┌') && line.contains('★'))
        .expect("expected marker in a top border");

    let chars: Vec<char> = target_line.chars().collect();
    let marker_pos = chars
        .iter()
        .position(|c| *c == '★')
        .expect("marker must be present");

    assert!(marker_pos > 0, "marker must not be the first char");
    assert!(
        marker_pos + 1 < chars.len(),
        "marker must not be the last char"
    );
    assert_eq!(chars[marker_pos - 1], '─', "char before marker must be `─`");
    assert_eq!(chars[marker_pos + 1], '┐', "char after marker must be `┐`");
    assert!(target_line.ends_with('┐'));
}

#[test]
fn test_render_layer_diagram_with_marker_legend_appended() {
    let layout = create_test_layout();
    let geometry = create_test_geometry();

    let markers = std::collections::HashMap::new();
    let diagram = render_layer_diagram_with_markers(
        &layout,
        0,
        &geometry,
        markers,
        Some("B: bootloader combo"),
    )
    .unwrap();

    assert!(diagram.ends_with("B: bootloader combo\n"));
}

#[test]
fn test_render_layer_diagram_with_missing_marker_position_ignored() {
    let layout = create_test_layout();
    let geometry = create_test_geometry();

    // Marker pointing at a position that doesn't exist in the layout.
    let mut markers = std::collections::HashMap::new();
    markers.insert((99, 99), 'X');

    let diagram =
        render_layer_diagram_with_markers(&layout, 0, &geometry, markers, None).unwrap();

    // Marker char must not appear anywhere — the position doesn't map to a key.
    assert!(!diagram.contains('X'));
}
