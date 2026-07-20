//! Shared helper functions for firmware generation tests.
//!
//! Each topic module does `use super::helpers::*;` to bring these into scope.

use chrono::Utc;
use lazyqmk::config::{BuildConfig, Config, PathConfig, UiConfig};
use lazyqmk::models::{
    KeyDefinition, KeyGeometry, KeyboardGeometry, Layer, Layout, LayoutMetadata, Position,
    RgbColor, VisualLayoutMapping,
};
use std::collections::HashMap;

// Re-exported for topic modules via `use super::helpers::*;`.
pub(super) use std::fs;
pub(super) use tempfile::TempDir;

/// Creates a minimal test layout for firmware generation.
pub(super) fn create_test_layout() -> Layout {
    let metadata = LayoutMetadata {
        name: "Test Layout".to_string(),
        description: "Integration test layout".to_string(),
        author: "Test Suite".to_string(),
        created: Utc::now(),
        modified: Utc::now(),
        tags: vec!["test".to_string()],
        is_template: false,
        version: "1.0.0".to_string(),
        layout_variant: Some("LAYOUT_test".to_string()),
        keyboard: Some("test_kb".to_string()),
        keymap_name: Some("test_keymap".to_string()),
        output_format: Some("uf2".to_string()),
    };

    // Create a simple 2x3 layout (6 keys)
    let mut keys = Vec::new();
    for row in 0..2 {
        for col in 0..3 {
            keys.push(KeyDefinition {
                position: Position { row, col },
                keycode: format!("KC_{}", (row * 3 + col)),
                label: None,
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
        id: uuid::Uuid::new_v4().to_string(),
        default_color: RgbColor::new(255, 255, 255),
        category_id: None,
        keys: keys.clone(),
        layer_colors_enabled: true,
    };

    // Second layer with some transparent keys
    let mut layer1_keys = keys.clone();
    layer1_keys[0].keycode = "KC_TRNS".to_string();
    layer1_keys[1].keycode = "KC_TRNS".to_string();

    let layer1 = Layer {
        number: 1,
        name: "Function".to_string(),
        id: uuid::Uuid::new_v4().to_string(),
        default_color: RgbColor::new(100, 100, 255),
        category_id: None,
        keys: layer1_keys,
        layer_colors_enabled: true,
    };

    Layout {
        metadata,
        layers: vec![layer0, layer1],
        categories: vec![],
        uncolored_key_behavior: lazyqmk::models::UncoloredKeyBehavior::default(),
        idle_effect_settings: lazyqmk::models::IdleEffectSettings::default(),
        rgb_overlay_ripple: lazyqmk::models::RgbOverlayRippleSettings::default(),
        palette_fx: lazyqmk::models::PaletteFxSettings::default(),
        tap_hold_settings: lazyqmk::models::TapHoldSettings::default(),
        rgb_enabled: true,
        rgb_brightness: lazyqmk::models::RgbBrightness::default(),
        rgb_saturation: lazyqmk::models::RgbSaturation::default(),
        rgb_matrix_default_speed: 127,
        rgb_timeout_ms: 0,
        tap_dances: vec![],
        combo_settings: lazyqmk::models::ComboSettings::default(),
    }
}

/// Creates a test keyboard geometry matching the test layout.
pub(super) fn create_test_geometry() -> KeyboardGeometry {
    let mut keys = Vec::new();

    // Map positions to matrix coordinates and LED indices
    for row in 0..2 {
        for col in 0..3 {
            let layout_idx = row * 3 + col;
            let key_geo = KeyGeometry {
                matrix_position: (row, col),
                led_index: layout_idx,
                layout_index: layout_idx,
                visual_x: f32::from(col) * 2.0,
                visual_y: f32::from(row) * 2.0,
                width: 1.0,
                height: 1.0,
                rotation: 0.0,
            };
            keys.push(key_geo);
        }
    }

    KeyboardGeometry {
        keyboard_name: "test_kb".to_string(),
        layout_name: "LAYOUT_test".to_string(),
        matrix_rows: 2,
        matrix_cols: 3,
        keys,
        encoder_count: 0,
    }
}

/// Creates a test visual layout mapping.
pub(super) fn create_test_mapping() -> VisualLayoutMapping {
    let mut led_to_matrix = Vec::new();
    let mut matrix_to_led = HashMap::new();
    let mut layout_to_matrix = Vec::new();
    let mut matrix_to_layout = HashMap::new();
    let mut matrix_to_visual = HashMap::new();
    let mut visual_to_matrix = HashMap::new();

    for row in 0..2 {
        for col in 0..3 {
            let pos = Position { row, col };
            let matrix_pos = (row, col);
            let led_idx = row * 3 + col;
            let layout_idx = row * 3 + col;

            // led_to_matrix is a Vec indexed by LED index
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
        max_col: 2, // 3 columns means max_col = 2
    }
}

/// Creates a test config with temporary directory.
pub(super) fn create_test_config(temp_dir: &TempDir) -> Config {
    let qmk_path = temp_dir.path().join("qmk_firmware");
    fs::create_dir_all(&qmk_path).unwrap();

    // Create minimal QMK directory structure
    let keyboards_dir = qmk_path
        .join("keyboards")
        .join("test_kb")
        .join("keymaps")
        .join("test_keymap");
    fs::create_dir_all(&keyboards_dir).unwrap();

    Config {
        paths: PathConfig {
            qmk_firmware: Some(qmk_path),
        },
        build: BuildConfig {
            output_dir: temp_dir.path().to_path_buf(),
        },
        ui: UiConfig::default(),
    }
}
