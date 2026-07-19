//! Keyboard visual renderer for layout export.
//!
//! Generates ASCII/Unicode keyboard diagrams using box-drawing characters.
//! Supports split keyboards, tap-hold keys, and color reference indicators.
//!
//! Module layout:
//! - [`rendering`] — grid building, split detection, box-drawing rendering.
//! - [`formatting`] — keycode → display text (`KC_A` → `A`, `LCTL_T(KC_X)` → multi-line).
//! - [`color_ref`] — color reference map builder + key color lookup.

use crate::models::{keyboard_geometry::KeyboardGeometry, layout::Layout};
use anyhow::{Context, Result};

mod color_ref;
mod formatting;
mod rendering;

use rendering::{build_key_grid, render_grid};

/// A single key positioned in the rendering grid.
pub struct GridKey {
    /// Display text for the key (formatted keycode)
    pub label: String,
    /// Optional color reference number (1-based)
    pub color_ref: Option<usize>,
    /// Row in the grid (0-based)
    pub row: usize,
    /// Column in the grid (0-based)
    pub col: usize,
    /// Width in grid units (1 = standard key)
    pub width: usize,
    /// Height in grid units (1 = standard key)
    pub height: usize,
}

/// Collection of keys arranged in a 2D grid.
pub struct KeyGrid {
    /// All keys in the grid
    pub keys: Vec<GridKey>,
    /// Maximum row index used
    pub max_row: usize,
    /// Maximum column index used
    pub max_col: usize,
    /// Gap between split halves (column index where right half starts)
    pub split_gap: Option<usize>,
}

/// Renders a single layer as an ASCII/Unicode keyboard diagram.
pub fn render_layer_diagram(
    layout: &Layout,
    layer_idx: usize,
    geometry: &KeyboardGeometry,
) -> Result<String> {
    let layer = layout
        .layers
        .get(layer_idx)
        .context("Layer index out of bounds")?;

    let mut output = String::new();

    // Add layer header
    use std::fmt::Write;
    writeln!(output, "Layer {}: {}", layer_idx, layer.name).unwrap();

    // Build key grid with positioning
    let key_grid = build_key_grid(layout, layer_idx, geometry)?;

    // Render the grid to ASCII/Unicode
    let diagram = render_grid(&key_grid);
    output.push_str(&diagram);

    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::formatting::format_keycode;
    use crate::keycode_db::format::{format_modifier, strip_kc_prefix};
    use super::rendering::detect_split_gap;
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
}
