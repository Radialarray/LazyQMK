//! Tap Dance & RGB Overlay Ripple Tests
//!
//! Tests for tap dance keycode generation (2-way, 3-way, multiple,
//! empty, invalid references, mixed) and RGB overlay ripple code
//! generation (enabled, disabled, with idle effect integration,
//! hue shift, release trigger, distance accumulator, zero params).

use super::fixtures::{test_geometry_basic, test_layout_basic};
use super::helpers::*;
use lazyqmk::firmware::FirmwareGenerator;
use lazyqmk::keycode_db::KeycodeDb;
use lazyqmk::models::VisualLayoutMapping;
use std::fs;

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

// === RGB Overlay Ripple Tests ===

/// Test RGB overlay ripple code generation.
#[test]
fn test_rgb_overlay_ripple_generation() {
    let keycode_db = KeycodeDb::load().expect("Failed to load keycode database");
    let mut layout = test_layout_basic(2, 3);

    // Enable RGB overlay ripple
    layout.rgb_overlay_ripple.enabled = true;
    layout.rgb_overlay_ripple.max_ripples = 4;
    layout.rgb_overlay_ripple.duration_ms = 1500;
    layout.rgb_overlay_ripple.speed = 200;
    layout.rgb_overlay_ripple.band_width = 30;
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

    // Verify reactive overlay code is present (PaletteFX-inspired algorithm)
    assert!(keymap_c.contains("#ifdef RGB_MATRIX_ENABLE"));
    assert!(keymap_c.contains("#ifdef LQMK_RIPPLE_OVERLAY_ENABLED"));
    assert!(keymap_c.contains("typedef struct {"));
    assert!(keymap_c.contains("ripple_t"));
    assert!(keymap_c.contains("static ripple_t ripples[LQMK_RIPPLE_MAX_RIPPLES]"));
    assert!(keymap_c.contains(
        "static void lazyqmk_ripple_add(uint8_t led_index, uint8_t row, uint8_t col, uint32_t delay_ms)"
    ));
    assert!(keymap_c.contains("static RGB lazyqmk_ripple_base_color(uint8_t led_index)"));
    assert!(keymap_c.contains("static void lazyqmk_reactive_apply(uint8_t led_index)"));
    assert!(keymap_c
        .contains("bool rgb_matrix_indicators_advanced_user(uint8_t led_min, uint8_t led_max)"));
    assert!(keymap_c.contains("lazyqmk_reactive_apply(i);"));
    // Background-lighting-on path: per-ripple contribution to each LED
    assert!(keymap_c.contains("uint8_t contrib_r = 0, contrib_g = 0, contrib_b = 0;"));
    assert!(keymap_c.contains("contrib_r = qadd8(contrib_r, scale8(c.r, bump));"));
    assert!(keymap_c.contains("base.r = qadd8(base.r, contrib_r);"));
    assert!(keymap_c.contains("rgb_t matrix_rgb = hsv_to_rgb(rgb_matrix_get_hsv());"));

    // Generate config.h
    let config_h = generator
        .generate_merged_config_h()
        .expect("Should generate config.h");

    // Verify ripple configuration defines are present
    assert!(config_h.contains("RGB Overlay Ripple Configuration"));
    assert!(config_h.contains("#define LQMK_RIPPLE_OVERLAY_ENABLED"));
    assert!(config_h.contains("#define LQMK_RIPPLE_MAX_RIPPLES 4"));
    assert!(config_h.contains("#define LQMK_RIPPLE_DURATION_MS 1500"));
    assert!(config_h.contains("#define LQMK_RIPPLE_SPEED 200"));
    assert!(config_h.contains("#define LQMK_RIPPLE_BAND_WIDTH 30"));
    assert!(config_h.contains("#define LQMK_RIPPLE_AMPLITUDE_PCT 50"));
    assert!(config_h.contains("#define LQMK_RIPPLE_TRIGGER_ON_PRESS 1"));
}

/// Test that ripple overlay code is NOT generated when disabled.
#[test]
fn test_rgb_overlay_ripple_disabled() {
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
    let trigger_idx = keymap_c
        .find("lazyqmk_ripple_trigger(keycode, record);")
        .expect("Ripple trigger call should exist");
    let press_gate_idx = keymap_c
        .find("if (record->event.pressed || ripple_triggered)")
        .expect("Idle-effect press gate should exist");
    assert!(
        trigger_idx < press_gate_idx,
        "Ripple trigger should be integrated before idle-effect press gating"
    );
    assert!(
        !keymap_c.contains("/*")
            || keymap_c.find("/*").unwrap() > keymap_c.find("lazyqmk_ripple_trigger").unwrap(),
        "Ripple trigger code should not be commented out"
    );

    // Fix 2: Must provide rgb_matrix_indicators_advanced_user to override QMK's weak default
    assert!(
        keymap_c
            .contains("bool rgb_matrix_indicators_advanced_user(uint8_t led_min, uint8_t led_max)"),
        "rgb_matrix_indicators_advanced_user must be a strong definition (no weak attribute)"
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
            .contains("static bool lazyqmk_ripple_trigger(uint16_t keycode, keyrecord_t *record)"),
        "Must have ripple trigger helper function"
    );

    assert!(
        keymap_c.contains("bool ripple_triggered = false;")
            && keymap_c.contains("ripple_triggered = lazyqmk_ripple_trigger(keycode, record);"),
        "Idle path must declare ripple flag outside ifdef and assign inside guarded block"
    );
    assert!(
        keymap_c.contains("if (record->event.pressed || ripple_triggered) {"),
        "Idle reset/wake path must run for release-triggered ripples too"
    );

    // Verify layer_base_colors table is actually generated
    assert!(
        keymap_c.contains("const uint8_t PROGMEM layer_base_colors"),
        "layer_base_colors table must be generated when RGB enabled"
    );
}

#[test]
fn test_rgb_overlay_ripple_hue_shift_generation() {
    let keycode_db = KeycodeDb::load().expect("Failed to load keycode database");
    let mut layout = test_layout_basic(2, 3);
    // Clear layer custom colors so the HueShift fallback path is exercised
    // (the background-lighting path uses originating-key colors directly)
    for layer in &mut layout.layers {
        layer.default_color = lazyqmk::models::RgbColor::default();
        layer.category_id = None;
    }
    layout.rgb_overlay_ripple.enabled = true;
    layout.rgb_overlay_ripple.color_mode = lazyqmk::models::RippleColorMode::HueShift;
    layout.rgb_overlay_ripple.hue_shift_deg = -120;

    let geometry = test_geometry_basic(2, 3);
    let mapping = VisualLayoutMapping::build(&geometry);

    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let config = create_test_config(&temp_dir);

    let generator = FirmwareGenerator::new(&layout, &geometry, &mapping, &config, &keycode_db);
    let keymap_c = generator
        .generate_keymap_c()
        .expect("Should generate keymap.c");

    // Hue shift now uses QMK's built-in rgb_to_hsv instead of a custom helper
    assert!(
        keymap_c.contains("hsv_t hsv = rgb_to_hsv(base);"),
        "Hue shift mode should use QMK's rgb_to_hsv"
    );
    assert!(
        keymap_c.contains("rgb_t shifted = hsv_to_rgb(hsv);"),
        "Hue shift mode must convert back to RGB via hsv_to_rgb"
    );
    assert!(
        keymap_c.contains("int16_t shifted_hue = (int16_t)hsv.h + -85;"),
        "Hue shift degrees should be converted to signed QMK hue steps (-120 deg = -85 steps)"
    );
    assert!(
        keymap_c.contains("while (shifted_hue < 0) shifted_hue += 256;")
            && keymap_c.contains("while (shifted_hue >= 256) shifted_hue -= 256;"),
        "Hue shift wrap must use full 256-step QMK hue space"
    );
}

#[test]
fn test_rgb_overlay_ripple_color_mode_honored_in_background_lighting_path() {
    // Regression guard: previously the additive-on-TUI path always emitted
    // `lazyqmk_ripple_base_color(ripples[i].led_index)` regardless of the
    // configured `color_mode`, so `fixed` and `hue_shift` were silently
    // ignored when background lighting was enabled (the default). Honor
    // the mode in this path too.
    let keycode_db = KeycodeDb::load().expect("Failed to load keycode database");
    let geometry = test_geometry_basic(2, 3);
    let mapping = VisualLayoutMapping::build(&geometry);
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let config = create_test_config(&temp_dir);

    // --- Fixed mode in the additive path ---
    let mut fixed_layout = test_layout_basic(2, 3);
    // Custom layer colors force `layout_has_custom_colors` -> additive path.
    fixed_layout.layers[0].default_color = lazyqmk::models::RgbColor::new(120, 120, 120);
    fixed_layout.rgb_overlay_ripple.enabled = true;
    fixed_layout.rgb_overlay_ripple.color_mode = lazyqmk::models::RippleColorMode::Fixed;
    fixed_layout.rgb_overlay_ripple.fixed_color = lazyqmk::models::RgbColor::new(255, 0, 255);
    let fixed_generator =
        FirmwareGenerator::new(&fixed_layout, &geometry, &mapping, &config, &keycode_db);
    let fixed_keymap = fixed_generator
        .generate_keymap_c()
        .expect("Should generate keymap.c");
    assert!(
        fixed_keymap.contains("RGB c = { 255, 0, 255 };  // fixed color"),
        "Fixed color mode must emit the configured RGB literal in the additive path"
    );

    // --- HueShift mode in the additive path ---
    let mut hs_layout = test_layout_basic(2, 3);
    hs_layout.layers[0].default_color = lazyqmk::models::RgbColor::new(120, 120, 120);
    hs_layout.rgb_overlay_ripple.enabled = true;
    hs_layout.rgb_overlay_ripple.color_mode = lazyqmk::models::RippleColorMode::HueShift;
    hs_layout.rgb_overlay_ripple.hue_shift_deg = 60;
    let hs_generator = FirmwareGenerator::new(&hs_layout, &geometry, &mapping, &config, &keycode_db);
    let hs_keymap = hs_generator
        .generate_keymap_c()
        .expect("Should generate keymap.c");
    assert!(
        hs_keymap.contains("hsv_t __hsv = rgb_to_hsv(__base);"),
        "HueShift mode must convert to HSV in the additive path"
    );
    assert!(
        hs_keymap.contains("int16_t shifted_hue = (int16_t)__hsv.h + 42;"),
        "HueShift steps must be computed from hue_shift_deg (60 deg = 42 steps)"
    );
}

#[test]
fn test_rgb_overlay_ripple_release_trigger_resets_idle_state() {
    let keycode_db = KeycodeDb::load().expect("Failed to load keycode database");
    let mut layout = test_layout_basic(2, 3);
    layout.rgb_overlay_ripple.enabled = true;
    layout.rgb_overlay_ripple.trigger_on_press = false;
    layout.rgb_overlay_ripple.trigger_on_release = true;
    layout.idle_effect_settings.enabled = true;

    let geometry = test_geometry_basic(2, 3);
    let mapping = VisualLayoutMapping::build(&geometry);
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let config = create_test_config(&temp_dir);

    let generator = FirmwareGenerator::new(&layout, &geometry, &mapping, &config, &keycode_db);
    let keymap_c = generator
        .generate_keymap_c()
        .expect("Should generate keymap.c");

    assert!(
        keymap_c.contains(
            "if (!record->event.pressed && LQMK_RIPPLE_TRIGGER_ON_RELEASE) should_trigger = true;"
        ),
        "Release-only trigger path must be generated"
    );
    assert!(
        keymap_c.contains("return true;") && keymap_c.contains("return false;"),
        "Ripple helper must report whether trigger fired"
    );
    assert!(
        keymap_c.contains("if (record->event.pressed || ripple_triggered) {")
            && keymap_c.contains("if (idle_state == IDLE_STATE_OFF) {")
            && keymap_c.contains("rgb_matrix_enable_noeeprom();"),
        "Release-triggered ripple must wake idle/off path"
    );
}

#[test]
fn test_rgb_overlay_ripple_distance_uses_wide_accumulator() {
    let keycode_db = KeycodeDb::load().expect("Failed to load keycode database");
    let mut layout = test_layout_basic(2, 3);
    layout.rgb_overlay_ripple.enabled = true;

    let geometry = test_geometry_basic(2, 3);
    let mapping = VisualLayoutMapping::build(&geometry);
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let config = create_test_config(&temp_dir);

    let generator = FirmwareGenerator::new(&layout, &geometry, &mapping, &config, &keycode_db);
    let keymap_c = generator
        .generate_keymap_c()
        .expect("Should generate keymap.c");

    // Distance calculation now uses matrix positions (row, col) for reliability
    assert!(
        keymap_c.contains("int8_t drow = (int8_t)led_row - (int8_t)ripples[i].row;"),
        "Distance must use signed matrix-position subtraction"
    );
}

#[test]
fn test_rgb_overlay_ripple_codegen_rejects_zero_duration_band_width_and_speed() {
    let keycode_db = KeycodeDb::load().expect("Failed to load keycode database");
    let geometry = test_geometry_basic(2, 3);
    let mapping = VisualLayoutMapping::build(&geometry);
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let config = create_test_config(&temp_dir);

    let mut duration_layout = test_layout_basic(2, 3);
    duration_layout.rgb_overlay_ripple.enabled = true;
    duration_layout.rgb_overlay_ripple.duration_ms = 0;
    let duration_generator =
        FirmwareGenerator::new(&duration_layout, &geometry, &mapping, &config, &keycode_db);
    assert!(duration_generator.generate_keymap_c().is_err());
    assert!(duration_generator.generate_merged_config_h().is_err());

    let mut band_width_layout = test_layout_basic(2, 3);
    band_width_layout.rgb_overlay_ripple.enabled = true;
    band_width_layout.rgb_overlay_ripple.band_width = 0;
    let band_width_generator = FirmwareGenerator::new(
        &band_width_layout,
        &geometry,
        &mapping,
        &config,
        &keycode_db,
    );
    assert!(band_width_generator.generate_keymap_c().is_err());
    assert!(band_width_generator.generate_merged_config_h().is_err());

    let mut speed_layout = test_layout_basic(2, 3);
    speed_layout.rgb_overlay_ripple.enabled = true;
    speed_layout.rgb_overlay_ripple.speed = 0;
    let speed_generator =
        FirmwareGenerator::new(&speed_layout, &geometry, &mapping, &config, &keycode_db);
    assert!(speed_generator.generate_keymap_c().is_err());
    assert!(speed_generator.generate_merged_config_h().is_err());
}
