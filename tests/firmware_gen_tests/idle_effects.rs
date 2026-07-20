//! Idle Effect Tests
//!
//! Tests that idle effect screensaver code is generated correctly
//! in both keymap.c and config.h.

use super::helpers::*;
use lazyqmk::firmware::FirmwareGenerator;
use lazyqmk::keycode_db::KeycodeDb;
use std::fs;

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
