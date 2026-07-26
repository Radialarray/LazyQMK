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

/// Regression test for LazyQMK-62q8: combo key arrays must resolve the layer
/// UUID in `LT(@uuid, ...)`, `MO(@uuid)`, `TG(@uuid)`, etc. into the numeric
/// index. Otherwise the generated C contains the literal `@uuid` string and
/// fails compilation with `error: stray '@' in program`.
#[test]
fn test_combo_resolves_layer_uuid_in_base_layer_keycodes() {
    let (mut layout, geometry, mapping, config, keycode_db) = create_test_setup();

    // Disable idle effect to keep the keymap minimal.
    layout.idle_effect_settings.enabled = false;

    // Add a second layer with a known UUID prefix that we can grep for in
    // the generated output. We'll reference this layer via an LT in the
    // base layer, then put a combo over the LT position.
    let mut layer1 = crate::models::layer::Layer::new(1, "Lower", RgbColor::new(0, 0, 0)).unwrap();
    layer1.id = "deadbeef-0000-0000-0000-000000000001".to_string();
    layer1.add_key(KeyDefinition::new(Position::new(0, 0), "KC_TRNS"));
    layer1.add_key(KeyDefinition::new(Position::new(0, 1), "KC_TRNS"));
    layout.add_layer(layer1).unwrap();

    // Replace the base-layer key at (0, 0) with an LT referencing the layer UUID.
    layout
        .get_layer_mut(0)
        .unwrap()
        .get_key_mut(Position::new(0, 0))
        .unwrap()
        .keycode = "LT(@deadbeef-0000-0000-0000-000000000001, KC_TAB)".to_string();

    // Enable combos with a combo that spans the LT position.
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

    // The combo key array should reference the resolved numeric layer (1),
    // not the literal UUID. We grep for the substring "LT(1, KC_TAB)" to
    // confirm resolution.
    assert!(
        keymap_c.contains("combo_0_keys[] = {LT(1, KC_TAB), KC_B, COMBO_END}")
            || keymap_c.contains("combo_0_keys[] = {KC_B, LT(1, KC_TAB), COMBO_END}"),
        "combo key array should resolve LT(@uuid, KC_TAB) to LT(1, KC_TAB); got: {keymap_c}"
    );
    assert!(
        !keymap_c.contains("@deadbeef"),
        "no raw layer UUID should leak into the generated keymap.c; got: {keymap_c}"
    );
    // Specifically no stray '@' inside combo_N_keys[] arrays.
    assert!(
        !keymap_c.contains("combo_0_keys[] = {") || !keymap_c.contains("@deadbeef"),
        "combo_0_keys must not contain the unresolved UUID; got: {keymap_c}"
    );
}

/// Regression test for LazyQMK-vuid: the generated `process_combo_event`
/// function must have balanced braces. When the early-return for the non-base
/// layer (`if (get_highest_layer(layer_state) != 0) { return; }`) was emitted
/// without its closing brace, QMK's `quantum/keymap_introspection.c` (which
/// `#include`s `KEYMAP_C`) parsed every subsequent introspection function as
/// nested inside `process_combo_event`, producing
/// `error: static declaration of 'keymap_layer_count_raw' follows non-static
/// declaration`.
///
/// This test sanity-checks that the generated block between
/// `void process_combo_event(` and `#endif // COMBO_ENABLE` has matched braces
/// and contains the expected early-return guard.
#[test]
fn test_process_combo_event_braces_are_balanced() {
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

    let combo_start = keymap_c
        .find("void process_combo_event(")
        .expect("process_combo_event should be in the generated keymap");
    let combo_end = keymap_c
        .find("#endif // COMBO_ENABLE")
        .expect("#endif // COMBO_ENABLE should be in the generated keymap");
    assert!(
        combo_start < combo_end,
        "process_combo_event must appear before #endif // COMBO_ENABLE"
    );

    let combo_block = &keymap_c[combo_start..combo_end];

    // Count braces inside the combo block. They must balance.
    let opens = combo_block.chars().filter(|&c| c == '{').count();
    let closes = combo_block.chars().filter(|&c| c == '}').count();
    assert_eq!(
        opens, closes,
        "braces inside the COMBO_ENABLE block must be balanced \
         ({opens} open vs {closes} close); block:\n{combo_block}"
    );

    // The early-return guard must be present and immediately closed so that
    // the rest of the function body is at the function scope.
    assert!(
        combo_block.contains("if (get_highest_layer(layer_state) != 0) {"),
        "early-return guard should be present; block:\n{combo_block}"
    );
    assert!(
        combo_block.contains("return;\n    }\n"),
        "early-return guard must close its brace before the rest of the function body; \
         block:\n{combo_block}"
    );
}
