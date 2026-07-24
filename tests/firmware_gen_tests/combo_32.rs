//! 32-Combo Exhaustion Tests
//!
//! Verifies that a layout with `combo_settings.enabled = true` and the maximum
//! allowed 32 combos generates a complete `process_combo_event` switch over
//! all 32 cases, and emits the correct C statements for each `ComboAction`.

use super::fixtures::{test_geometry_basic, test_layout_basic};
use super::helpers::*;
use lazyqmk::firmware::FirmwareGenerator;
use lazyqmk::keycode_db::KeycodeDb;
use lazyqmk::models::{ComboAction, ComboDefinition, Position, VisualLayoutMapping};
use std::fs;

/// Maximum number of combos allowed by the model.
const COMBO_CAP: usize = 32;

/// Grid dimensions: 4 rows × 16 cols = 64 keys — enough for 32 combos of 2
/// distinct visual positions each, with every pair unique.
const ROWS: usize = 4;
const COLS: usize = 16;

#[test]
fn test_combo_settings_exhaust_32_combos_generated() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let mut layout = test_layout_basic(ROWS, COLS);
    layout.combo_settings.enabled = true;

    let actions = ComboAction::all();

    // Walk the grid top-to-bottom, two adjacent columns per combo. With
    // 32 combos × 2 positions = 64 keys, every pair is unique.
    for combo_idx in 0..COMBO_CAP {
        let row = (combo_idx / (COLS / 2)) as u8;
        let pair_col = (combo_idx % (COLS / 2)) as u8;
        let col1 = pair_col * 2;
        let col2 = col1 + 1;

        let action = actions[combo_idx % actions.len()].clone();
        let combo = ComboDefinition::new(
            Position::new(row, col1),
            Position::new(row, col2),
            action,
        );
        layout.combo_settings.combos.push(combo);
    }

    assert_eq!(
        layout.combo_settings.combos.len(),
        COMBO_CAP,
        "Test setup should populate exactly {COMBO_CAP} combos"
    );

    let geometry = test_geometry_basic(ROWS, COLS);
    let mapping = VisualLayoutMapping::build(&geometry);
    let config = create_test_config(&temp_dir);
    let keycode_db = KeycodeDb::load().expect("Failed to load keycode database");

    let generator = FirmwareGenerator::new(&layout, &geometry, &mapping, &config, &keycode_db);
    let (keymap_path, config_path) = generator
        .generate()
        .expect("Generation with 32 combos should succeed");

    let keymap_content = fs::read_to_string(&keymap_path).expect("Should read keymap.c");
    let config_content = fs::read_to_string(&config_path).expect("Should read config.h");

    // 1. config.h must declare COMBO_COUNT 32.
    assert!(
        config_content.contains("#define COMBO_COUNT 32"),
        "config.h should contain '#define COMBO_COUNT 32'; got: {}",
        config_content
    );

    // 2. enum combo_events must list every COMBO_N from 0..31.
    for idx in 0..COMBO_CAP {
        let token = format!("COMBO_{}", idx);
        assert!(
            keymap_content.contains(&token),
            "keymap.c should contain {} in the combo enum; got: {}",
            token,
            keymap_content
        );
    }

    // 3. process_combo_event must switch over all 32 cases.
    for idx in 0..COMBO_CAP {
        let case_str = format!("case COMBO_{}:", idx);
        assert!(
            keymap_content.contains(&case_str),
            "keymap.c should contain {} in process_combo_event; got: {}",
            case_str,
            keymap_content
        );
    }

    // 4. combo_state array must be sized to 32.
    assert!(
        keymap_content.contains("combo_state[32];"),
        "keymap.c should declare combo_state[32]; got: {}",
        keymap_content
    );

    // 5. Each action's C emit must appear.
    //    Bootloader -> reset_keyboard();
    assert!(
        keymap_content.contains("reset_keyboard();"),
        "Bootloader action should emit reset_keyboard(); got: {}",
        keymap_content
    );

    //    DisableLighting -> rgb_matrix_disable_noeeprom() + rgb_matrix_enable_noeeprom()
    //    (the toggle emits both, since the toggle covers the "off" branch).
    assert!(
        keymap_content.contains("rgb_matrix_disable_noeeprom();"),
        "DisableLighting action should emit rgb_matrix_disable_noeeprom(); got: {}",
        keymap_content
    );
    assert!(
        keymap_content.contains("rgb_matrix_enable_noeeprom();"),
        "DisableLighting action should emit rgb_matrix_enable_noeeprom(); got: {}",
        keymap_content
    );

    //    DisableEffects -> rgb_matrix_mode_noeeprom(...) — TUI_LAYER_COLORS when
    //    the layout has custom colors, SOLID_COLOR otherwise. Both are valid.
    assert!(
        keymap_content.contains("rgb_matrix_mode_noeeprom(RGB_MATRIX_SOLID_COLOR)")
            || keymap_content
                .contains("rgb_matrix_mode_noeeprom(RGB_MATRIX_TUI_LAYER_COLORS)"),
        "DisableEffects action should emit rgb_matrix_mode_noeeprom(...); got: {}",
        keymap_content
    );
}
