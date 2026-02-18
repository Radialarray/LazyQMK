//! Shared test fixtures for E2E CLI tests.
#![allow(dead_code)] // Some fixtures reserved for future tests

use chrono::{TimeZone, Utc};
use lazyqmk::config::{BuildConfig, Config, PathConfig, UiConfig};
use lazyqmk::models::{
    Category, IdleEffectSettings, KeyDefinition, KeyGeometry, KeyboardGeometry, Layer, Layout,
    LayoutMetadata, Position, RgbBrightness, RgbColor, RgbMatrixEffect, RgbOverlayRippleSettings,
    RgbSaturation, TapDanceAction, TapHoldSettings, UncoloredKeyBehavior, VisualLayoutMapping,
};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// Creates a basic test layout with simple keycodes.
///
/// # Arguments
/// * `rows` - Number of rows (matrix coordinates)
/// * `cols` - Number of columns (matrix coordinates)
///
/// # Returns
/// A `Layout` with deterministic metadata and two layers (Base, Function).
#[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
pub fn test_layout_basic(rows: usize, cols: usize) -> Layout {
    // Use deterministic timestamps
    let created = Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();
    let modified = Utc.with_ymd_and_hms(2025, 1, 2, 0, 0, 0).unwrap();

    let metadata = LayoutMetadata {
        name: "Test Layout".to_string(),
        description: "E2E test layout".to_string(),
        author: "Test Suite".to_string(),
        created,
        modified,
        tags: vec!["test".to_string(), "e2e".to_string()],
        is_template: false,
        version: "1.0".to_string(),
        layout_variant: Some("LAYOUT_test".to_string()),
        keyboard: Some("test_keyboard".to_string()),
        keymap_name: Some("test_keymap".to_string()),
        output_format: Some("uf2".to_string()),
    };

    // Layer 0: Base layer with simple keycodes
    let mut base_keys = Vec::new();
    for row in 0..rows {
        for col in 0..cols {
            let key_num = row * cols + col;
            base_keys.push(KeyDefinition {
                position: Position {
                    row: row as u8,
                    col: col as u8,
                },
                keycode: format!("KC_{}", key_num),
                label: Some(key_num.to_string()),
                color_override: None,
                category_id: None,
                combo_participant: false,
                description: None,
            });
        }
    }

    let layer0 = Layer {
        number: 0,
        name: "Base".to_string(),
        id: "00000000-0000-0000-0000-000000000000".to_string(), // Deterministic UUID
        default_color: RgbColor::new(255, 255, 255),
        category_id: None,
        keys: base_keys,
        layer_colors_enabled: true,
    };

    // Layer 1: Function layer with some transparent keys
    let mut func_keys = Vec::new();
    for row in 0..rows {
        for col in 0..cols {
            let keycode = if row == 0 && col == 0 {
                "KC_TRNS".to_string()
            } else {
                format!("KC_F{}", row * cols + col)
            };
            func_keys.push(KeyDefinition {
                position: Position {
                    row: row as u8,
                    col: col as u8,
                },
                keycode,
                label: None,
                color_override: None,
                category_id: None,
                combo_participant: false,
                description: None,
            });
        }
    }

    let layer1 = Layer {
        number: 1,
        name: "Function".to_string(),
        id: "11111111-1111-1111-1111-111111111111".to_string(), // Deterministic UUID
        default_color: RgbColor::new(100, 100, 255),
        category_id: None,
        keys: func_keys,
        layer_colors_enabled: true,
    };

    Layout {
        metadata,
        layers: vec![layer0, layer1],
        categories: vec![],
        rgb_enabled: true,
        rgb_brightness: RgbBrightness::default(),
        rgb_saturation: RgbSaturation::default(),
        rgb_matrix_default_speed: 127,
        rgb_timeout_ms: 0,
        uncolored_key_behavior: UncoloredKeyBehavior::default(),
        idle_effect_settings: IdleEffectSettings::default(),
        rgb_overlay_ripple: RgbOverlayRippleSettings::default(),
        tap_hold_settings: TapHoldSettings::default(),
        tap_dances: vec![],
    }
}

/// Creates a test layout with tap dance definitions.
pub fn test_layout_with_tap_dances() -> Layout {
    let mut layout = test_layout_basic(2, 3);

    // Add 2-way tap dance
    let td_esc_caps = TapDanceAction {
        name: "esc_caps".to_string(),
        single_tap: "KC_ESC".to_string(),
        double_tap: Some("KC_CAPS".to_string()),
        hold: None,
    };

    // Add 3-way tap dance
    let td_shift = TapDanceAction {
        name: "shift_ctrl".to_string(),
        single_tap: "KC_LSFT".to_string(),
        double_tap: Some("KC_CAPS".to_string()),
        hold: Some("KC_LCTL".to_string()),
    };

    layout.tap_dances = vec![td_esc_caps, td_shift];

    // Use tap dances in the layout
    layout.layers[0].keys[0].keycode = "TD(esc_caps)".to_string();
    layout.layers[0].keys[1].keycode = "TD(shift_ctrl)".to_string();

    layout
}

/// Creates a test layout with idle effect settings.
///
/// # Arguments
/// * `enabled` - Whether idle effect should be enabled
pub fn test_layout_with_idle_effect(enabled: bool) -> Layout {
    let mut layout = test_layout_basic(2, 3);

    layout.idle_effect_settings = IdleEffectSettings {
        enabled,
        idle_timeout_ms: 30_000,
        idle_effect_duration_ms: 120_000,
        idle_effect_mode: RgbMatrixEffect::Breathing,
    };

    // Also set RGB timeout to test precedence
    layout.rgb_timeout_ms = 90_000;

    layout
}

/// Creates a test layout with categories and category assignments.
pub fn test_layout_with_categories() -> Layout {
    let mut layout = test_layout_basic(2, 3);

    // Add categories
    let nav_category = Category::new("navigation", "Navigation", RgbColor::new(0, 255, 0))
        .expect("Should create category");
    let num_category = Category::new("numbers", "Numbers", RgbColor::new(255, 128, 0))
        .expect("Should create category");

    layout.categories = vec![nav_category, num_category];

    // Assign categories to some keys
    layout.layers[0].keys[0].category_id = Some("navigation".to_string());
    layout.layers[0].keys[1].category_id = Some("numbers".to_string());

    layout
}

/// Creates a test layout with layer references (LT, MO, etc).
pub fn test_layout_with_layer_refs() -> Layout {
    let mut layout = test_layout_basic(2, 3);

    // Use layer references with UUIDs
    let layer1_id = layout.layers[1].id.clone();
    layout.layers[0].keys[0].keycode = format!("LT(@{}, KC_SPC)", layer1_id);
    layout.layers[0].keys[1].keycode = format!("MO(@{})", layer1_id);
    layout.layers[0].keys[2].keycode = format!("TG(@{})", layer1_id);

    layout
}

/// Creates a test layout with invalid keycodes for validation testing.
pub fn test_layout_with_invalid_keycode() -> Layout {
    let mut layout = test_layout_basic(2, 3);
    layout.layers[0].keys[0].keycode = "INVALID_KEYCODE_XYZ".to_string();
    layout
}

/// Creates a basic keyboard geometry matching the test layout dimensions.
///
/// # Arguments
/// * `rows` - Number of rows
/// * `cols` - Number of columns
#[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
pub fn test_geometry_basic(rows: usize, cols: usize) -> KeyboardGeometry {
    let mut keys = Vec::new();

    for row in 0..rows {
        for col in 0..cols {
            let layout_idx = row * cols + col;
            let key_geo = KeyGeometry {
                matrix_position: (row as u8, col as u8),
                led_index: layout_idx as u8,
                layout_index: layout_idx as u8,
                visual_x: col as f32,
                visual_y: row as f32,
                width: 1.0,
                height: 1.0,
                rotation: 0.0,
            };
            keys.push(key_geo);
        }
    }

    KeyboardGeometry {
        keyboard_name: "test_keyboard".to_string(),
        layout_name: "LAYOUT_test".to_string(),
        matrix_rows: rows as u8,
        matrix_cols: cols as u8,
        keys,
        encoder_count: 0,
    }
}

/// Creates a basic visual layout mapping with bidirectional transforms.
///
/// # Arguments
/// * `rows` - Number of rows
/// * `cols` - Number of columns
#[allow(clippy::cast_possible_truncation)]
pub fn test_mapping_basic(rows: usize, cols: usize) -> VisualLayoutMapping {
    let mut led_to_matrix = Vec::new();
    let mut matrix_to_led = HashMap::new();
    let mut layout_to_matrix = Vec::new();
    let mut matrix_to_layout = HashMap::new();
    let mut matrix_to_visual = HashMap::new();
    let mut visual_to_matrix = HashMap::new();

    for row in 0..rows {
        for col in 0..cols {
            let pos = Position {
                row: row as u8,
                col: col as u8,
            };
            let matrix_pos = (row as u8, col as u8);
            let led_idx = (row * cols + col) as u8;
            let layout_idx = (row * cols + col) as u8;

            led_to_matrix.push(matrix_pos);
            layout_to_matrix.push(matrix_pos);

            matrix_to_led.insert(matrix_pos, led_idx);
            matrix_to_layout.insert(matrix_pos, layout_idx);
            matrix_to_visual.insert(matrix_pos, pos);
            visual_to_matrix.insert(pos, matrix_pos);
        }
    }

    VisualLayoutMapping {
        led_to_matrix,
        matrix_to_led,
        layout_to_matrix,
        matrix_to_layout,
        matrix_to_visual,
        visual_to_matrix,
        max_col: (cols - 1) as u8,
    }
}

/// Creates a temporary config with QMK path set.
///
/// # Arguments
/// * `qmk_path` - Optional QMK firmware path (creates minimal structure if None)
pub fn temp_config_with_qmk(qmk_path: Option<PathBuf>) -> (Config, TempDir) {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let qmk_path = qmk_path.unwrap_or_else(|| {
        let path = temp_dir.path().join("qmk_firmware");
        fs::create_dir_all(&path).expect("Failed to create QMK dir");

        // Create minimal QMK structure with info.json
        let keyboard_dir = path.join("keyboards").join("test_keyboard");
        let keymaps_dir = keyboard_dir.join("keymaps").join("test_keymap");
        fs::create_dir_all(&keymaps_dir).expect("Failed to create keyboards dir");

        // Create minimal info.json for test_keyboard
        let info_json = serde_json::json!({
            "keyboard_name": "test_keyboard",
            "manufacturer": "Test",
            "maintainer": "test",
            "processor": "atmega32u4",
            "bootloader": "atmel-dfu",
            "usb": {
                "vid": "0xFEED",
                "pid": "0x0000",
                "device_version": "1.0.0"
            },
            "matrix_pins": {
                "cols": ["F0", "F1", "F2"],
                "rows": ["D0", "D1"]
            },
            "diode_direction": "COL2ROW",
            "layouts": {
                "LAYOUT_test": {
                    "layout": [
                        {"matrix": [0, 0], "x": 0, "y": 0},
                        {"matrix": [0, 1], "x": 1, "y": 0},
                        {"matrix": [0, 2], "x": 2, "y": 0},
                        {"matrix": [1, 0], "x": 0, "y": 1},
                        {"matrix": [1, 1], "x": 1, "y": 1},
                        {"matrix": [1, 2], "x": 2, "y": 1}
                    ]
                }
            }
        });

        fs::write(
            keyboard_dir.join("info.json"),
            serde_json::to_string_pretty(&info_json).unwrap(),
        )
        .expect("Failed to write info.json");

        // Create minimal Makefile (required for validation)
        fs::write(
            path.join("Makefile"),
            "# Minimal QMK Makefile for testing\n",
        )
        .expect("Failed to write Makefile");

        path
    });

    let config = Config {
        paths: PathConfig {
            qmk_firmware: Some(qmk_path),
        },
        build: BuildConfig {
            output_dir: temp_dir.path().to_path_buf(),
        },
        ui: UiConfig::default(),
    };

    (config, temp_dir)
}

/// Writes a layout to a markdown file for CLI testing.
///
/// # Arguments
/// * `layout` - The layout to serialize
/// * `path` - The file path to write to
pub fn write_layout_file(layout: &Layout, path: &Path) -> std::io::Result<()> {
    use lazyqmk::services::LayoutService;

    LayoutService::save(layout, path).map_err(|e| std::io::Error::other(e.to_string()))
}

/// Creates a layout file in a temp directory and returns the path.
pub fn create_temp_layout_file(layout: &Layout) -> (PathBuf, TempDir) {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let layout_path = temp_dir.path().join("test_layout.md");
    write_layout_file(layout, &layout_path).expect("Failed to write layout file");
    (layout_path, temp_dir)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fixture_basic_layout() {
        let layout = test_layout_basic(2, 3);
        assert_eq!(layout.layers.len(), 2);
        assert_eq!(layout.layers[0].keys.len(), 6);
        assert_eq!(layout.layers[0].name, "Base");
    }

    #[test]
    fn test_fixture_with_tap_dances() {
        let layout = test_layout_with_tap_dances();
        assert_eq!(layout.tap_dances.len(), 2);
        assert!(layout.layers[0].keys[0].keycode.contains("TD("));
    }

    #[test]
    fn test_fixture_with_idle_effect() {
        let layout = test_layout_with_idle_effect(true);
        assert!(layout.idle_effect_settings.enabled);
        assert_eq!(layout.idle_effect_settings.idle_timeout_ms, 30_000);
    }

    #[test]
    fn test_fixture_geometry_matches_layout() {
        let layout = test_layout_basic(2, 3);
        let geometry = test_geometry_basic(2, 3);
        assert_eq!(geometry.keys.len(), layout.layers[0].keys.len());
    }

    #[test]
    fn test_fixture_mapping() {
        let mapping = test_mapping_basic(2, 3);
        assert_eq!(mapping.led_to_matrix.len(), 6);
        assert_eq!(mapping.max_col, 2);
    }
}
