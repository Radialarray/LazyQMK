//! Tests for the bootloader-combo decoupling (LazyQMK-epl0.5).
//!
//! Before the configurable combo system was extended to handle the bootloader
//! action, the idle-effect generator hard-coded a Q+R / U+P combo with a
//! 1500 ms hold that called `bootloader_jump()`.  These tests pin down the
//! decoupling: the hard-coded block must be gone, and the configurable
//! combo system must continue to emit `reset_keyboard()` for any combo
//! with `ComboAction::Bootloader`.

use super::*;
use crate::models::layer::Position;

#[test]
fn test_bootloader_combo_no_hardcoded_fallback() {
    // Verify that the hardcoded Q+R / U+P bootloader combo block is no longer
    // emitted anywhere in the generated keymap regardless of feature flags.
    let (mut layout, geometry, mapping, config, keycode_db) = create_test_setup();

    layout.idle_effect_settings.enabled = false;
    layout.combo_settings.enabled = false;
    layout.combo_settings.combos.clear();

    let generator = FirmwareGenerator::new(&layout, &geometry, &mapping, &config, &keycode_db);
    let keymap_c = generator.generate_keymap_c().unwrap();

    assert!(
        !keymap_c.contains("bootloader_combo_timer"),
        "keymap should not contain hardcoded bootloader_combo_timer; got: {keymap_c}"
    );
    assert!(
        !keymap_c.contains("bootloader_combo_active"),
        "keymap should not contain hardcoded bootloader_combo_active; got: {keymap_c}"
    );
    assert!(
        !keymap_c.contains("Q+R (left) or U+P"),
        "keymap should not contain the hardcoded Q+R / U+P combo comment; got: {keymap_c}"
    );
    assert!(
        !keymap_c.contains("> 1500"),
        "keymap should not contain the hardcoded 1500 ms timer check; got: {keymap_c}"
    );
}

#[test]
fn test_bootloader_combo_via_general_combos() {
    // Verify that bootloader is now triggered through the configurable combo
    // system (combo.rs) instead of the hardcoded idle-effect fallback.
    let (mut layout, geometry, mapping, config, keycode_db) = create_test_setup();

    layout.idle_effect_settings.enabled = false;
    layout.combo_settings.enabled = true;
    layout
        .combo_settings
        .add_combo(crate::models::ComboDefinition::with_duration(
            Position::new(0, 0),
            Position::new(0, 1),
            crate::models::ComboAction::Bootloader,
            1000,
        ))
        .unwrap();

    let generator = FirmwareGenerator::new(&layout, &geometry, &mapping, &config, &keycode_db);
    let keymap_c = generator.generate_keymap_c().unwrap();

    assert!(
        keymap_c.contains("case COMBO_0:"),
        "COMBO_0 case should be present in generated keymap; got: {keymap_c}"
    );
    assert!(
        keymap_c.contains("reset_keyboard();"),
        "reset_keyboard() should be emitted by the bootloader combo; got: {keymap_c}"
    );
    assert!(
        keymap_c.contains("combo_0_keys[] = {KC_A, KC_B, COMBO_END}"),
        "combo key array should reference the base-layer keycodes (KC_A, KC_B); got: {keymap_c}"
    );
    assert!(
        !keymap_c.contains("bootloader_combo_timer"),
        "general combo system should not emit the hardcoded timer; got: {keymap_c}"
    );
    assert!(
        !keymap_c.contains("Q+R (left) or U+P"),
        "general combo system should not emit the hardcoded Q+R / U+P comment; got: {keymap_c}"
    );
}
