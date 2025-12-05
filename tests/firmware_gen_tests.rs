//! Integration tests for firmware generation pipeline.
//!
//! Tests the complete flow:
//! 1. Validation of layouts before generation
//! 2. Generation of keymap.c and config.h files
//! 3. File writing with atomic operations
//! 4. Coordinate system transformations (visual -> matrix -> LED)

use chrono::Utc;
use keyboard_configurator::config::{BuildConfig, Config, PathConfig};
use keyboard_configurator::firmware::{FirmwareGenerator, FirmwareValidator};
use keyboard_configurator::keycode_db::KeycodeDb;
use keyboard_configurator::models::{
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
    assert_eq!(resolved.unwrap(), "LT(1, KC_SPC)", "Should resolve to layer index 1");
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
    assert_eq!(resolved.unwrap(), "LM(1, MOD_LSFT)", "Should resolve to layer index 1");
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
    
    assert!(resolved.is_none(), "Invalid layer reference should return None");
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
    
    assert!(result.is_ok(), "Generation with parameterized keycodes should succeed");
    
    let (keymap_path, _) = result.unwrap();
    let content = fs::read_to_string(&keymap_path).expect("Should be able to read keymap.c");
    
    // LT with @uuid should be resolved to index
    assert!(content.contains("LT(1, KC_SPC)"), "LT should be resolved to layer index: {}", content);
    // MT and SH_T should pass through as-is
    assert!(content.contains("MT(MOD_LCTL, KC_A)"), "MT should be in output");
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
        uncolored_key_behavior: keyboard_configurator::models::UncoloredKeyBehavior::default(),
        tap_hold_settings: keyboard_configurator::models::TapHoldSettings::default(),
        rgb_enabled: true,
        rgb_brightness: keyboard_configurator::models::RgbBrightness::default(),
        rgb_timeout_ms: 0,
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
        ui: Default::default(),
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
        content.contains("// Generated by keyboard-configurator"),
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

/// DEPRECATED: vial.json is no longer generated since migration to standard QMK
#[test]
#[ignore]
fn test_generation_vial_json_structure() {
    // This test is deprecated - vial.json is no longer generated
    // Kept for reference
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
