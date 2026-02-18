//! Integration tests for firmware generation pipeline.
//!
//! Tests the complete flow:
//! 1. Validation of layouts before generation
//! 2. Generation of keymap.c and config.h files
//! 3. File writing with atomic operations
//! 4. Coordinate system transformations (visual -> matrix -> LED)

mod fixtures;

use chrono::Utc;
use lazyqmk::config::{BuildConfig, Config, PathConfig, UiConfig};
use lazyqmk::firmware::{FirmwareGenerator, FirmwareValidator};
use lazyqmk::keycode_db::KeycodeDb;
use lazyqmk::models::{
    Category, KeyDefinition, KeyGeometry, KeyboardGeometry, Layer, Layout, LayoutMetadata,
    Position, RgbColor, VisualLayoutMapping,
};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

// === Parameterized Keycode Tests ===

/// Test that LT() keycodes with layer references are resolved correctly
#[test]
fn test_lt_keycode_resolution() {
    let keycode_db = KeycodeDb::load().expect("Failed to load keycode database");
    // Create a layout with multiple layers
    let mut layout = create_test_layout();
    let layer1_id = layout.layers[1].id.clone();

    // Set a key to LT(@layer1_id, KC_SPC)
    layout.layers[0].keys[0].keycode = format!("LT(@{}, KC_SPC)", layer1_id);

    // The resolve_layer_keycode function should convert @uuid to index
    let resolved = layout.resolve_layer_keycode(&layout.layers[0].keys[0].keycode, &keycode_db);
    assert!(resolved.is_some(), "Should resolve LT keycode");
    assert_eq!(
        resolved.unwrap(),
        "LT(1, KC_SPC)",
        "Should resolve to layer index 1"
    );
}

/// Test that LM() keycodes with layer references are resolved correctly
#[test]
fn test_lm_keycode_resolution() {
    let keycode_db = KeycodeDb::load().expect("Failed to load keycode database");
    let mut layout = create_test_layout();
    let layer1_id = layout.layers[1].id.clone();

    // Set a key to LM(@layer1_id, MOD_LSFT)
    layout.layers[0].keys[0].keycode = format!("LM(@{}, MOD_LSFT)", layer1_id);

    let resolved = layout.resolve_layer_keycode(&layout.layers[0].keys[0].keycode, &keycode_db);
    assert!(resolved.is_some(), "Should resolve LM keycode");
    assert_eq!(
        resolved.unwrap(),
        "LM(1, MOD_LSFT)",
        "Should resolve to layer index 1"
    );
}

/// Test that simple layer keycodes still work (MO, TG, etc)
#[test]
fn test_simple_layer_keycode_resolution() {
    let keycode_db = KeycodeDb::load().expect("Failed to load keycode database");
    let mut layout = create_test_layout();
    let layer1_id = layout.layers[1].id.clone();

    layout.layers[0].keys[0].keycode = format!("MO(@{})", layer1_id);
    layout.layers[0].keys[1].keycode = format!("TG(@{})", layer1_id);

    let resolved_mo = layout.resolve_layer_keycode(&layout.layers[0].keys[0].keycode, &keycode_db);
    let resolved_tg = layout.resolve_layer_keycode(&layout.layers[0].keys[1].keycode, &keycode_db);

    assert_eq!(resolved_mo.unwrap(), "MO(1)");
    assert_eq!(resolved_tg.unwrap(), "TG(1)");
}

/// Test that invalid layer references return None
#[test]
fn test_invalid_layer_reference() {
    let keycode_db = KeycodeDb::load().expect("Failed to load keycode database");
    let layout = create_test_layout();

    // Reference a non-existent layer
    let keycode = "LT(@nonexistent-uuid, KC_A)";
    let resolved = layout.resolve_layer_keycode(keycode, &keycode_db);

    assert!(
        resolved.is_none(),
        "Invalid layer reference should return None"
    );
}

/// Test that MT() and SH_T() keycodes pass through (no layer reference)
#[test]
fn test_non_layer_keycodes_passthrough() {
    let keycode_db = KeycodeDb::load().expect("Failed to load keycode database");
    let layout = create_test_layout();

    // MT and SH_T don't have layer references, so resolve_layer_keycode returns None
    let mt_code = "MT(MOD_LCTL, KC_A)";
    let sht_code = "SH_T(KC_SPC)";

    let resolved_mt = layout.resolve_layer_keycode(mt_code, &keycode_db);
    let resolved_sht = layout.resolve_layer_keycode(sht_code, &keycode_db);

    // These are not layer keycodes, so they return None (handled by resolve_keycode fallback)
    assert!(resolved_mt.is_none(), "MT should not be a layer keycode");
    assert!(resolved_sht.is_none(), "SH_T should not be a layer keycode");
}

// === Idle Effect Tests ===

#[test]
fn test_idle_effect_enabled_in_keymap() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let mut layout = create_test_layout();

    // Enable idle effect
    layout.idle_effect_settings.enabled = true;
    layout.idle_effect_settings.idle_timeout_ms = 30_000;
    layout.idle_effect_settings.idle_effect_duration_ms = 120_000;
    layout.idle_effect_settings.idle_effect_mode = lazyqmk::models::RgbMatrixEffect::Breathing;

    let geometry = create_test_geometry();
    let mapping = create_test_mapping();
    let config = create_test_config(&temp_dir);
    let keycode_db = KeycodeDb::load().expect("Failed to load keycode database");

    let generator = FirmwareGenerator::new(&layout, &geometry, &mapping, &config, &keycode_db);
    let result = generator.generate();

    assert!(result.is_ok(), "Generation with idle effect should succeed");

    let (keymap_path, config_path) = result.unwrap();
    let keymap_content = fs::read_to_string(&keymap_path).expect("Should read keymap.c");
    let config_content = fs::read_to_string(&config_path).expect("Should read config.h");

    // Check keymap.c contains idle effect state machine
    assert!(
        keymap_content.contains("idle_state_t"),
        "Should have idle state enum"
    );
    assert!(
        keymap_content.contains("IDLE_STATE_ACTIVE"),
        "Should have ACTIVE state"
    );
    assert!(
        keymap_content.contains("IDLE_STATE_IDLE_EFFECT"),
        "Should have IDLE_EFFECT state"
    );
    assert!(
        keymap_content.contains("IDLE_STATE_OFF"),
        "Should have OFF state"
    );
    assert!(
        keymap_content.contains("matrix_scan_user"),
        "Should have matrix_scan_user"
    );
    assert!(
        keymap_content.contains("process_record_user"),
        "Should have process_record_user"
    );
    assert!(
        keymap_content.contains("keyboard_post_init_user"),
        "Should have keyboard_post_init_user"
    );

    // Check config.h contains idle effect defines
    assert!(
        config_content.contains("#define LQMK_IDLE_TIMEOUT_MS 30000"),
        "Should have timeout define"
    );
    assert!(
        config_content.contains("#define LQMK_IDLE_EFFECT_DURATION_MS 120000"),
        "Should have duration define"
    );
    assert!(
        config_content.contains("#define LQMK_IDLE_EFFECT_MODE RGB_MATRIX_BREATHING"),
        "Should have mode define"
    );
}

#[test]
fn test_idle_effect_disabled_no_code() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let mut layout = create_test_layout();

    // Disable idle effect
    layout.idle_effect_settings.enabled = false;

    let geometry = create_test_geometry();
    let mapping = create_test_mapping();
    let config = create_test_config(&temp_dir);
    let keycode_db = KeycodeDb::load().expect("Failed to load keycode database");

    let generator = FirmwareGenerator::new(&layout, &geometry, &mapping, &config, &keycode_db);
    let result = generator.generate();

    assert!(result.is_ok());

    let (keymap_path, config_path) = result.unwrap();
    let keymap_content = fs::read_to_string(&keymap_path).expect("Should read keymap.c");
    let config_content = fs::read_to_string(&config_path).expect("Should read config.h");

    // Should not contain idle effect code
    assert!(
        !keymap_content.contains("idle_state_t"),
        "Should not have idle state enum"
    );
    assert!(
        !config_content.contains("LQMK_IDLE_TIMEOUT_MS"),
        "Should not have idle defines"
    );
}

#[test]
fn test_idle_effect_no_rgb_matrix_timeout_conflict() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let mut layout = create_test_layout();

    // Enable idle effect AND set rgb_timeout_ms (which should be ignored)
    layout.idle_effect_settings.enabled = true;
    layout.idle_effect_settings.idle_timeout_ms = 60_000;
    layout.idle_effect_settings.idle_effect_duration_ms = 300_000;
    layout.rgb_timeout_ms = 120_000; // This should be ignored

    let geometry = create_test_geometry();
    let mapping = create_test_mapping();
    let config = create_test_config(&temp_dir);
    let keycode_db = KeycodeDb::load().expect("Failed to load keycode database");

    let generator = FirmwareGenerator::new(&layout, &geometry, &mapping, &config, &keycode_db);
    let result = generator.generate();

    assert!(result.is_ok());

    let (_, config_path) = result.unwrap();
    let config_content = fs::read_to_string(&config_path).expect("Should read config.h");

    // Should have idle effect defines
    assert!(
        config_content.contains("LQMK_IDLE_TIMEOUT_MS"),
        "Should have idle timeout"
    );

    // Should NOT have RGB_MATRIX_TIMEOUT when idle effect is enabled
    assert!(
        !config_content.contains("#define RGB_MATRIX_TIMEOUT"),
        "Should not emit RGB_MATRIX_TIMEOUT when idle effect is enabled"
    );
}

#[test]
fn test_rgb_matrix_timeout_when_idle_disabled() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let mut layout = create_test_layout();

    // Disable idle effect but set rgb_timeout_ms
    layout.idle_effect_settings.enabled = false;
    layout.rgb_timeout_ms = 90_000;

    let geometry = create_test_geometry();
    let mapping = create_test_mapping();
    let config = create_test_config(&temp_dir);
    let keycode_db = KeycodeDb::load().expect("Failed to load keycode database");

    let generator = FirmwareGenerator::new(&layout, &geometry, &mapping, &config, &keycode_db);
    let result = generator.generate();

    assert!(result.is_ok());

    let (_, config_path) = result.unwrap();
    let config_content = fs::read_to_string(&config_path).expect("Should read config.h");

    // Should have RGB_MATRIX_TIMEOUT when idle effect is disabled
    assert!(
        config_content.contains("#define RGB_MATRIX_TIMEOUT 90000"),
        "Should emit RGB_MATRIX_TIMEOUT when idle effect is disabled"
    );

    // Should NOT have idle effect defines
    assert!(
        !config_content.contains("LQMK_IDLE_TIMEOUT_MS"),
        "Should not have idle defines"
    );
}

#[test]
fn test_idle_effect_different_modes() {
    use lazyqmk::models::RgbMatrixEffect;

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let keycode_db = KeycodeDb::load().expect("Failed to load keycode database");

    let effects = vec![
        (RgbMatrixEffect::SolidColor, "RGB_MATRIX_SOLID_COLOR"),
        (RgbMatrixEffect::Breathing, "RGB_MATRIX_BREATHING"),
        (
            RgbMatrixEffect::RainbowMovingChevron,
            "RGB_MATRIX_RAINBOW_MOVING_CHEVRON",
        ),
        (RgbMatrixEffect::CycleAll, "RGB_MATRIX_CYCLE_ALL"),
        (
            RgbMatrixEffect::JellybeanRaindrops,
            "RGB_MATRIX_JELLYBEAN_RAINDROPS",
        ),
    ];

    for (effect, expected_mode) in effects {
        let mut layout = create_test_layout();
        layout.idle_effect_settings.enabled = true;
        layout.idle_effect_settings.idle_effect_mode = effect;

        let geometry = create_test_geometry();
        let mapping = create_test_mapping();
        let config = create_test_config(&temp_dir);

        let generator = FirmwareGenerator::new(&layout, &geometry, &mapping, &config, &keycode_db);
        let result = generator.generate();

        assert!(
            result.is_ok(),
            "Generation with {:?} should succeed",
            effect
        );

        let (_, config_path) = result.unwrap();
        let config_content = fs::read_to_string(&config_path).expect("Should read config.h");

        assert!(
            config_content.contains(&format!("#define LQMK_IDLE_EFFECT_MODE {}", expected_mode)),
            "Should have correct mode for {:?}: expected {}",
            effect,
            expected_mode
        );
    }
}

/// Test firmware generation with parameterized keycodes
#[test]
fn test_generation_with_parameterized_keycodes() {
    let keycode_db = KeycodeDb::load().expect("Failed to load keycode database");
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let mut layout = create_test_layout();
    let layer1_id = layout.layers[1].id.clone();

    // Add parameterized keycodes
    layout.layers[0].keys[0].keycode = format!("LT(@{}, KC_SPC)", layer1_id);
    layout.layers[0].keys[1].keycode = "MT(MOD_LCTL, KC_A)".to_string();
    layout.layers[0].keys[2].keycode = "SH_T(KC_ESC)".to_string();

    let geometry = create_test_geometry();
    let mapping = create_test_mapping();
    let config = create_test_config(&temp_dir);

    let generator = FirmwareGenerator::new(&layout, &geometry, &mapping, &config, &keycode_db);
    let result = generator.generate();

    assert!(
        result.is_ok(),
        "Generation with parameterized keycodes should succeed"
    );

    let (keymap_path, _) = result.unwrap();
    let content = fs::read_to_string(&keymap_path).expect("Should be able to read keymap.c");

    // LT with @uuid should be resolved to index
    assert!(
        content.contains("LT(1, KC_SPC)"),
        "LT should be resolved to layer index: {}",
        content
    );
    // MT and SH_T should pass through as-is
    assert!(
        content.contains("MT(MOD_LCTL, KC_A)"),
        "MT should be in output"
    );
    assert!(content.contains("SH_T(KC_ESC)"), "SH_T should be in output");
}

/// Creates a minimal test layout for firmware generation.
fn create_test_layout() -> Layout {
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
        tap_hold_settings: lazyqmk::models::TapHoldSettings::default(),
        rgb_enabled: true,
        rgb_brightness: lazyqmk::models::RgbBrightness::default(),
        rgb_saturation: lazyqmk::models::RgbSaturation::default(),
        rgb_timeout_ms: 0,
        tap_dances: vec![],
    }
}

/// Creates a test keyboard geometry matching the test layout.
fn create_test_geometry() -> KeyboardGeometry {
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
fn create_test_mapping() -> VisualLayoutMapping {
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
fn create_test_config(temp_dir: &TempDir) -> Config {
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

#[test]
fn test_generation_creates_files() {
    // Arrange
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let layout = create_test_layout();
    let geometry = create_test_geometry();
    let mapping = create_test_mapping();
    let config = create_test_config(&temp_dir);

    let keycode_db = KeycodeDb::load().expect("Failed to load keycode database");

    // Act
    let generator = FirmwareGenerator::new(&layout, &geometry, &mapping, &config, &keycode_db);
    let result = generator.generate();

    // Assert
    assert!(
        result.is_ok(),
        "Generation should succeed: {:?}",
        result.err()
    );

    let (keymap_path, _) = result.unwrap();
    assert!(
        PathBuf::from(&keymap_path).exists(),
        "keymap.c should be created"
    );
}

#[test]
fn test_generation_keymap_c_structure() {
    // Arrange
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let layout = create_test_layout();
    let geometry = create_test_geometry();
    let mapping = create_test_mapping();
    let config = create_test_config(&temp_dir);

    let keycode_db = KeycodeDb::load().expect("Failed to load keycode database");

    // Act
    let generator = FirmwareGenerator::new(&layout, &geometry, &mapping, &config, &keycode_db);
    let result = generator.generate();
    assert!(result.is_ok(), "Generation should succeed");

    let (keymap_path, _) = result.unwrap();
    let content = fs::read_to_string(&keymap_path).expect("Should be able to read keymap.c");

    // Assert - Check for expected C code structure
    assert!(
        content.contains("// Generated by lazyqmk"),
        "Should have generator comment"
    );
    assert!(
        content.contains("const uint16_t PROGMEM keymaps"),
        "Should have PROGMEM keymaps array"
    );
    assert!(
        content.contains("LAYOUT_test"),
        "Should use correct layout macro"
    );
    assert!(
        content.contains("KC_0"),
        "Should contain keycodes from layer 0"
    );
    assert!(
        content.contains("KC_TRNS"),
        "Should contain transparent keycodes from layer 1"
    );

    // Check for layer comments
    assert!(
        content.contains("// Layer 0: Base"),
        "Should have layer 0 comment"
    );
    assert!(
        content.contains("// Layer 1: Function"),
        "Should have layer 1 comment"
    );
}

#[test]
fn test_generation_led_ordering() {
    // Arrange
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let layout = create_test_layout();
    let geometry = create_test_geometry();
    let mapping = create_test_mapping();
    let config = create_test_config(&temp_dir);

    let keycode_db = KeycodeDb::load().expect("Failed to load keycode database");

    // Act
    let generator = FirmwareGenerator::new(&layout, &geometry, &mapping, &config, &keycode_db);
    let result = generator.generate();
    assert!(result.is_ok(), "Generation should succeed");

    let (keymap_path, _) = result.unwrap();
    let content = fs::read_to_string(&keymap_path).expect("Should be able to read keymap.c");

    // Assert - Keys should be ordered by LED index (0, 1, 2, 3, 4, 5)
    // Find the first LAYOUT_test macro in Layer 0
    let layer0_start = content
        .find("// Layer 0: Base")
        .expect("Should find Layer 0 comment");
    let layer0_layout = &content[layer0_start..];
    let layout_start = layer0_layout
        .find("LAYOUT_test(")
        .expect("Should find LAYOUT_test macro");
    let layout_section = &layer0_layout[layout_start..];

    // Keys should appear in LED order: KC_0, KC_1, KC_2, KC_3, KC_4, KC_5
    let kc0_pos = layout_section.find("KC_0").expect("Should find KC_0");
    let kc1_pos = layout_section.find("KC_1").expect("Should find KC_1");
    let kc2_pos = layout_section.find("KC_2").expect("Should find KC_2");
    let kc5_pos = layout_section.find("KC_5").expect("Should find KC_5");

    assert!(kc0_pos < kc1_pos, "KC_0 should appear before KC_1");
    assert!(kc1_pos < kc2_pos, "KC_1 should appear before KC_2");
    assert!(kc2_pos < kc5_pos, "KC_2 should appear before KC_5");
}

#[test]
fn test_generation_with_categories() {
    // Arrange
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let mut layout = create_test_layout();

    // Add a category
    let nav_category = Category::new("navigation", "Navigation", RgbColor::new(0, 255, 0))
        .expect("Should create category");

    layout.categories.push(nav_category);
    layout.layers[0].keys[0].category_id = Some("navigation".to_string());

    let geometry = create_test_geometry();
    let mapping = create_test_mapping();
    let config = create_test_config(&temp_dir);

    let keycode_db = KeycodeDb::load().expect("Failed to load keycode database");

    // Act
    let generator = FirmwareGenerator::new(&layout, &geometry, &mapping, &config, &keycode_db);
    let result = generator.generate();

    // Assert
    assert!(result.is_ok(), "Generation with categories should succeed");

    let (keymap_path, _) = result.unwrap();
    let content = fs::read_to_string(&keymap_path).expect("Should be able to read keymap.c");

    // Categories don't affect C code generation, but should not cause errors
    assert!(content.contains("KC_0"), "Should still contain keycodes");
}

#[test]
fn test_generation_atomic_write() {
    // Arrange
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let layout = create_test_layout();
    let geometry = create_test_geometry();
    let mapping = create_test_mapping();
    let config = create_test_config(&temp_dir);
    let keycode_db = KeycodeDb::load().expect("Failed to load keycode database");

    // Act - Generate twice to test atomic write (temp + rename)
    let generator = FirmwareGenerator::new(&layout, &geometry, &mapping, &config, &keycode_db);
    let result1 = generator.generate();
    assert!(result1.is_ok(), "First generation should succeed");

    let result2 = generator.generate();
    assert!(
        result2.is_ok(),
        "Second generation should succeed (overwrite)"
    );

    // Assert - Files should exist and be readable
    let (keymap_path, _) = result2.unwrap();
    let keymap_content =
        fs::read_to_string(&keymap_path).expect("Should read keymap.c after overwrite");

    assert!(!keymap_content.is_empty(), "keymap.c should not be empty");
}

#[test]
fn test_full_pipeline_validation_to_generation() {
    // Arrange
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let layout = create_test_layout();
    let geometry = create_test_geometry();
    let mapping = create_test_mapping();
    let config = create_test_config(&temp_dir);
    let keycode_db = KeycodeDb::load().expect("Failed to load keycode database");

    // Act - Full pipeline: Validate then Generate

    // Step 1: Validate
    let validator = FirmwareValidator::new(&layout, &geometry, &mapping, &keycode_db);
    let report = validator.validate().expect("Validation should complete");
    assert!(
        report.is_valid(),
        "Layout should be valid before generation"
    );

    // Step 2: Generate
    let generator = FirmwareGenerator::new(&layout, &geometry, &mapping, &config, &keycode_db);
    let result = generator.generate();

    // Assert
    assert!(
        result.is_ok(),
        "Generation should succeed after validation: {:?}",
        result.err()
    );

    let (keymap_path, _) = result.unwrap();
    assert!(
        PathBuf::from(&keymap_path).exists(),
        "keymap.c should exist"
    );

    // Verify file contents are non-empty and valid
    let keymap_content = fs::read_to_string(&keymap_path).unwrap();

    assert!(
        keymap_content.len() > 100,
        "keymap.c should have substantial content"
    );
}

// === Tap Dance Tests ===

#[test]
fn test_tap_dance_two_way_generation() {
    use lazyqmk::models::TapDanceAction;

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let mut layout = create_test_layout();

    // Add a 2-way tap dance
    let td = TapDanceAction::new("test", "KC_A".to_string()).with_double_tap("KC_B".to_string());
    layout.tap_dances.push(td);

    // Apply TD(test) to a key
    layout.layers[0].keys[0].keycode = "TD(test)".to_string();

    let geometry = create_test_geometry();
    let mapping = create_test_mapping();
    let config = create_test_config(&temp_dir);
    let keycode_db = KeycodeDb::load().expect("Failed to load keycode database");

    let generator = FirmwareGenerator::new(&layout, &geometry, &mapping, &config, &keycode_db);
    let result = generator.generate();

    assert!(
        result.is_ok(),
        "Generation with 2-way tap dance should succeed"
    );

    let (keymap_path, _) = result.unwrap();
    let content = fs::read_to_string(&keymap_path).expect("Should read keymap.c");

    // Check enum
    assert!(
        content.contains("enum tap_dance_ids"),
        "Should have tap dance enum"
    );
    assert!(content.contains("TD_TEST"), "Should have TD_TEST in enum");

    // Check actions array with 2-way macro
    assert!(
        content.contains("tap_dance_action_t tap_dance_actions"),
        "Should have actions array"
    );
    assert!(
        content.contains("ACTION_TAP_DANCE_DOUBLE(KC_A, KC_B)"),
        "Should use ACTION_TAP_DANCE_DOUBLE"
    );

    // Check keymap uses TD(TD_TEST)
    assert!(
        content.contains("TD(TD_TEST)"),
        "Should use TD(TD_TEST) in keymap"
    );
}

#[test]
fn test_tap_dance_three_way_generation() {
    use lazyqmk::models::TapDanceAction;

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let mut layout = create_test_layout();

    // Add a 3-way tap dance
    let td = TapDanceAction::new("triple", "KC_ESC".to_string())
        .with_double_tap("KC_CAPS".to_string())
        .with_hold("KC_LCTL".to_string());
    layout.tap_dances.push(td);

    // Apply TD(triple) to a key
    layout.layers[0].keys[1].keycode = "TD(triple)".to_string();

    let geometry = create_test_geometry();
    let mapping = create_test_mapping();
    let config = create_test_config(&temp_dir);
    let keycode_db = KeycodeDb::load().expect("Failed to load keycode database");

    let generator = FirmwareGenerator::new(&layout, &geometry, &mapping, &config, &keycode_db);
    let result = generator.generate();

    assert!(
        result.is_ok(),
        "Generation with 3-way tap dance should succeed"
    );

    let (keymap_path, _) = result.unwrap();
    let content = fs::read_to_string(&keymap_path).expect("Should read keymap.c");

    // Check enum
    assert!(
        content.contains("TD_TRIPLE"),
        "Should have TD_TRIPLE in enum"
    );

    // Check helper functions
    assert!(
        content.contains("void td_triple_finished"),
        "Should have finished function"
    );
    assert!(
        content.contains("void td_triple_reset"),
        "Should have reset function"
    );
    assert!(
        content.contains("register_code16(KC_ESC)"),
        "Should register single tap"
    );
    assert!(
        content.contains("register_code16(KC_CAPS)"),
        "Should register double tap"
    );
    assert!(
        content.contains("register_code16(KC_LCTL)"),
        "Should register hold"
    );
    assert!(
        content.contains("unregister_code16(KC_ESC)"),
        "Should unregister single tap"
    );
    assert!(
        content.contains("unregister_code16(KC_CAPS)"),
        "Should unregister double tap"
    );
    assert!(
        content.contains("unregister_code16(KC_LCTL)"),
        "Should unregister hold"
    );

    // Check actions array with 3-way macro
    assert!(
        content.contains("ACTION_TAP_DANCE_FN_ADVANCED(NULL, td_triple_finished, td_triple_reset)"),
        "Should use ACTION_TAP_DANCE_FN_ADVANCED"
    );

    // Check keymap uses TD(TD_TRIPLE)
    assert!(
        content.contains("TD(TD_TRIPLE)"),
        "Should use TD(TD_TRIPLE) in keymap"
    );
}

#[test]
fn test_tap_dance_multiple_stable_ordering() {
    use lazyqmk::models::TapDanceAction;

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let mut layout = create_test_layout();

    // Add multiple tap dances in non-alphabetical order
    let td_zebra =
        TapDanceAction::new("zebra", "KC_Z".to_string()).with_double_tap("KC_X".to_string());
    let td_alpha =
        TapDanceAction::new("alpha", "KC_A".to_string()).with_double_tap("KC_B".to_string());
    let td_beta =
        TapDanceAction::new("beta", "KC_C".to_string()).with_double_tap("KC_D".to_string());

    layout.tap_dances.push(td_zebra);
    layout.tap_dances.push(td_alpha);
    layout.tap_dances.push(td_beta);

    // Apply TD keycodes
    layout.layers[0].keys[0].keycode = "TD(zebra)".to_string();
    layout.layers[0].keys[1].keycode = "TD(alpha)".to_string();
    layout.layers[0].keys[2].keycode = "TD(beta)".to_string();

    let geometry = create_test_geometry();
    let mapping = create_test_mapping();
    let config = create_test_config(&temp_dir);
    let keycode_db = KeycodeDb::load().expect("Failed to load keycode database");

    let generator = FirmwareGenerator::new(&layout, &geometry, &mapping, &config, &keycode_db);
    let result = generator.generate();

    assert!(
        result.is_ok(),
        "Generation with multiple tap dances should succeed"
    );

    let (keymap_path, _) = result.unwrap();
    let content = fs::read_to_string(&keymap_path).expect("Should read keymap.c");

    // Verify alphabetical ordering in enum (alpha, beta, zebra)
    let td_alpha_pos = content.find("TD_ALPHA").expect("Should find TD_ALPHA");
    let td_beta_pos = content.find("TD_BETA").expect("Should find TD_BETA");
    let td_zebra_pos = content.find("TD_ZEBRA").expect("Should find TD_ZEBRA");

    assert!(
        td_alpha_pos < td_beta_pos,
        "TD_ALPHA should come before TD_BETA in enum"
    );
    assert!(
        td_beta_pos < td_zebra_pos,
        "TD_BETA should come before TD_ZEBRA in enum"
    );

    // Verify all keycodes are transformed
    assert!(content.contains("TD(TD_ZEBRA)"), "Should have TD(TD_ZEBRA)");
    assert!(content.contains("TD(TD_ALPHA)"), "Should have TD(TD_ALPHA)");
    assert!(content.contains("TD(TD_BETA)"), "Should have TD(TD_BETA)");
}

#[test]
fn test_tap_dance_empty_no_code() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let layout = create_test_layout();
    // No tap dances added

    let geometry = create_test_geometry();
    let mapping = create_test_mapping();
    let config = create_test_config(&temp_dir);
    let keycode_db = KeycodeDb::load().expect("Failed to load keycode database");

    let generator = FirmwareGenerator::new(&layout, &geometry, &mapping, &config, &keycode_db);
    let result = generator.generate();

    assert!(
        result.is_ok(),
        "Generation with no tap dances should succeed"
    );

    let (keymap_path, _) = result.unwrap();
    let content = fs::read_to_string(&keymap_path).expect("Should read keymap.c");

    // Should not contain tap dance code
    assert!(
        !content.contains("enum tap_dance_ids"),
        "Should not have tap dance enum"
    );
    assert!(
        !content.contains("tap_dance_action_t"),
        "Should not have actions array"
    );
    assert!(
        !content.contains("ACTION_TAP_DANCE"),
        "Should not have tap dance macros"
    );
}

#[test]
fn test_tap_dance_invalid_reference_passthrough() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let mut layout = create_test_layout();

    // Apply TD() keycode that doesn't exist in tap_dances
    layout.layers[0].keys[0].keycode = "TD(nonexistent)".to_string();

    let geometry = create_test_geometry();
    let mapping = create_test_mapping();
    let config = create_test_config(&temp_dir);
    let keycode_db = KeycodeDb::load().expect("Failed to load keycode database");

    let generator = FirmwareGenerator::new(&layout, &geometry, &mapping, &config, &keycode_db);
    let result = generator.generate();

    // Should still generate (validation would catch this, but generator is permissive)
    assert!(
        result.is_ok(),
        "Generation should not fail on missing tap dance ref"
    );

    let (keymap_path, _) = result.unwrap();
    let content = fs::read_to_string(&keymap_path).expect("Should read keymap.c");

    // Should pass through unchanged (validator will catch the error)
    assert!(
        content.contains("TD(nonexistent)"),
        "Should pass through invalid reference"
    );
}

#[test]
fn test_tap_dance_mixed_two_and_three_way() {
    use lazyqmk::models::TapDanceAction;

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let mut layout = create_test_layout();

    // Add one 2-way and one 3-way
    let td_two = TapDanceAction::new("two", "KC_A".to_string()).with_double_tap("KC_B".to_string());
    let td_three = TapDanceAction::new("three", "KC_X".to_string())
        .with_double_tap("KC_Y".to_string())
        .with_hold("KC_Z".to_string());

    layout.tap_dances.push(td_two);
    layout.tap_dances.push(td_three);

    layout.layers[0].keys[0].keycode = "TD(two)".to_string();
    layout.layers[0].keys[1].keycode = "TD(three)".to_string();

    let geometry = create_test_geometry();
    let mapping = create_test_mapping();
    let config = create_test_config(&temp_dir);
    let keycode_db = KeycodeDb::load().expect("Failed to load keycode database");

    let generator = FirmwareGenerator::new(&layout, &geometry, &mapping, &config, &keycode_db);
    let result = generator.generate();

    assert!(
        result.is_ok(),
        "Generation with mixed tap dances should succeed"
    );

    let (keymap_path, _) = result.unwrap();
    let content = fs::read_to_string(&keymap_path).expect("Should read keymap.c");

    // Check both are in enum
    assert!(content.contains("TD_TWO"), "Should have TD_TWO");
    assert!(content.contains("TD_THREE"), "Should have TD_THREE");

    // Check 2-way uses simple macro
    assert!(
        content.contains("ACTION_TAP_DANCE_DOUBLE(KC_A, KC_B)"),
        "Should have 2-way macro"
    );

    // Check 3-way has helper functions and advanced macro
    assert!(
        content.contains("void td_three_finished"),
        "Should have 3-way helpers"
    );
    assert!(
        content.contains("ACTION_TAP_DANCE_FN_ADVANCED(NULL, td_three_finished, td_three_reset)"),
        "Should have 3-way macro"
    );
}

/// Test RGB overlay ripple code generation.
#[test]
fn test_rgb_overlay_ripple_generation() {
    use fixtures::{test_geometry_basic, test_layout_basic};

    let keycode_db = KeycodeDb::load().expect("Failed to load keycode database");
    let mut layout = test_layout_basic(2, 3);

    // Enable RGB overlay ripple
    layout.rgb_overlay_ripple.enabled = true;
    layout.rgb_overlay_ripple.max_ripples = 4;
    layout.rgb_overlay_ripple.duration_ms = 500;
    layout.rgb_overlay_ripple.speed = 128;
    layout.rgb_overlay_ripple.band_width = 3;
    layout.rgb_overlay_ripple.amplitude_pct = 50;

    let geometry = test_geometry_basic(2, 3);
    let mapping = VisualLayoutMapping::build(&geometry);

    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let config = create_test_config(&temp_dir);

    let generator = FirmwareGenerator::new(&layout, &geometry, &mapping, &config, &keycode_db);

    // Generate keymap.c
    let keymap_c = generator
        .generate_keymap_c()
        .expect("Should generate keymap.c");

    // Verify ripple overlay code is present
    assert!(keymap_c.contains("#ifdef RGB_MATRIX_ENABLE"));
    assert!(keymap_c.contains("#ifdef LQMK_RIPPLE_OVERLAY_ENABLED"));
    assert!(keymap_c.contains("typedef struct {"));
    assert!(keymap_c.contains("ripple_t"));
    assert!(keymap_c.contains("static ripple_t ripples[LQMK_RIPPLE_MAX_RIPPLES]"));
    assert!(keymap_c.contains("static void lazyqmk_ripple_add(uint8_t led_index)"));
    assert!(keymap_c.contains("static uint8_t lazyqmk_ripple_distance(uint8_t led1, uint8_t led2)"));
    assert!(
        keymap_c.contains("static void lazyqmk_ripple_apply(uint8_t led_index, RGB *led_color)")
    );
    assert!(keymap_c
        .contains("bool rgb_matrix_indicators_advanced_user(uint8_t led_min, uint8_t led_max)"));
    assert!(keymap_c.contains("lazyqmk_ripple_apply(i, &color)"));

    // Generate config.h
    let config_h = generator
        .generate_merged_config_h()
        .expect("Should generate config.h");

    // Verify ripple configuration defines are present
    assert!(config_h.contains("RGB Overlay Ripple Configuration"));
    assert!(config_h.contains("#define LQMK_RIPPLE_OVERLAY_ENABLED"));
    assert!(config_h.contains("#define LQMK_RIPPLE_MAX_RIPPLES 4"));
    assert!(config_h.contains("#define LQMK_RIPPLE_DURATION_MS 500"));
    assert!(config_h.contains("#define LQMK_RIPPLE_SPEED 128"));
    assert!(config_h.contains("#define LQMK_RIPPLE_BAND_WIDTH 3"));
    assert!(config_h.contains("#define LQMK_RIPPLE_AMPLITUDE_PCT 50"));
    assert!(config_h.contains("#define LQMK_RIPPLE_TRIGGER_ON_PRESS 1"));
}

/// Test that ripple overlay code is NOT generated when disabled.
#[test]
fn test_rgb_overlay_ripple_disabled() {
    use fixtures::{test_geometry_basic, test_layout_basic};

    let keycode_db = KeycodeDb::load().expect("Failed to load keycode database");
    let layout = test_layout_basic(2, 3); // Ripple disabled by default

    let geometry = test_geometry_basic(2, 3);
    let mapping = VisualLayoutMapping::build(&geometry);

    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let config = create_test_config(&temp_dir);

    let generator = FirmwareGenerator::new(&layout, &geometry, &mapping, &config, &keycode_db);

    // Generate keymap.c
    let keymap_c = generator
        .generate_keymap_c()
        .expect("Should generate keymap.c");

    // Verify ripple overlay code is NOT present
    assert!(!keymap_c.contains("LQMK_RIPPLE_OVERLAY_ENABLED"));
    assert!(!keymap_c.contains("lazyqmk_ripple_add"));

    // Generate config.h
    let config_h = generator
        .generate_merged_config_h()
        .expect("Should generate config.h");

    // Verify ripple configuration defines are NOT present
    assert!(!config_h.contains("RGB Overlay Ripple Configuration"));
    assert!(!config_h.contains("#define LQMK_RIPPLE_OVERLAY_ENABLED"));
}

/// Comprehensive test verifying all critical fixes from code review (commit d0e0de7):
/// 1. Ripple trigger integrated with idle effect (not commented)
/// 2. __attribute__((weak)) on rgb_matrix_indicators_advanced_user
/// 3. Keypress->LED mapping (not hardcoded center LED)
/// 4. layer_base_colors dependency properly guarded
#[test]
fn test_rgb_overlay_ripple_with_idle_effect_integration() {
    use fixtures::{test_geometry_basic, test_layout_basic};

    let keycode_db = KeycodeDb::load().expect("Failed to load keycode database");
    let mut layout = test_layout_basic(2, 3);

    // Enable BOTH ripple overlay AND idle effect to test integration
    layout.rgb_overlay_ripple.enabled = true;
    layout.rgb_overlay_ripple.trigger_on_press = true;
    layout.idle_effect_settings.enabled = true;

    let geometry = test_geometry_basic(2, 3);
    let mapping = VisualLayoutMapping::build(&geometry);

    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let config = create_test_config(&temp_dir);

    let generator = FirmwareGenerator::new(&layout, &geometry, &mapping, &config, &keycode_db);

    // Generate keymap.c
    let keymap_c = generator
        .generate_keymap_c()
        .expect("Should generate keymap.c");

    // Fix 1: Ripple trigger must be integrated (not commented out) when idle effect enabled
    assert!(
        keymap_c.contains("lazyqmk_ripple_trigger(keycode, record)"),
        "Ripple trigger should be integrated into idle effect's process_record_user"
    );
    assert!(
        !keymap_c.contains("/*")
            || keymap_c.find("/*").unwrap() > keymap_c.find("lazyqmk_ripple_trigger").unwrap(),
        "Ripple trigger code should not be commented out"
    );

    // Fix 2: Must have __attribute__((weak)) to avoid duplicate symbol conflicts
    assert!(
        keymap_c.contains("__attribute__((weak)) bool rgb_matrix_indicators_advanced_user"),
        "rgb_matrix_indicators_advanced_user must have weak attribute"
    );

    // Fix 3: Must use actual keypress position, not hardcoded center LED
    assert!(
        keymap_c.contains("static uint8_t lazyqmk_matrix_to_led(uint8_t row, uint8_t col)"),
        "Must have matrix-to-LED mapping function"
    );
    assert!(
        keymap_c.contains("g_led_config.matrix_co[row][col]"),
        "Must use QMK's g_led_config for mapping"
    );
    assert!(
        keymap_c.contains("lazyqmk_matrix_to_led(record->event.key.row, record->event.key.col)"),
        "Ripple must use actual keypress matrix position"
    );
    assert!(
        !keymap_c.contains("RGB_MATRIX_LED_COUNT / 2")
            || keymap_c.find("RGB_MATRIX_LED_COUNT / 2").unwrap()
                > keymap_c.find("// Fallback to center").unwrap(),
        "Should not use hardcoded center LED except as fallback"
    );

    // Fix 4: layer_base_colors must be guarded to handle missing dependency
    assert!(
        keymap_c.contains("#ifdef LAYER_BASE_COLORS_LAYER_COUNT"),
        "layer_base_colors access must be guarded with ifdef"
    );

    // Verify the ripple trigger helper function exists with proper signature
    assert!(
        keymap_c
            .contains("static void lazyqmk_ripple_trigger(uint16_t keycode, keyrecord_t *record)"),
        "Must have ripple trigger helper function"
    );

    // Verify layer_base_colors table is actually generated
    assert!(
        keymap_c.contains("const uint8_t PROGMEM layer_base_colors"),
        "layer_base_colors table must be generated when RGB enabled"
    );
}
