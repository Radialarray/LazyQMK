//! Parameterized Keycode Tests
//!
//! Tests that LT(), LM(), MO(), TG(), MT(), and SH_T() keycodes
//! with layer references are resolved correctly.

use super::helpers::*;
use lazyqmk::keycode_db::KeycodeDb;

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
