//! Tests for db.
//!
//! Auto-extracted from db.rs.

use super::*;

use super::super::ParamType;
use super::*;

fn get_test_db() -> KeycodeDb {
    KeycodeDb::load().expect("Failed to load keycode database")
}

#[test]
fn test_load_database() {
    let db = get_test_db();
    assert!(db.keycode_count() > 100);
    assert!(db.category_count() > 5);
}

#[test]
fn test_is_valid_basic_keys() {
    let db = get_test_db();
    assert!(db.is_valid("KC_A"));
    assert!(db.is_valid("KC_B"));
    assert!(db.is_valid("KC_1"));
    assert!(db.is_valid("KC_ENT"));
    assert!(db.is_valid("KC_ENTER")); // Alias
}

#[test]
fn test_is_valid_special_keys() {
    let db = get_test_db();
    assert!(db.is_valid("KC_TRNS"));
    assert!(db.is_valid("KC_TRANSPARENT")); // Alias
    assert!(db.is_valid("KC_NO"));
}

#[test]
fn test_is_valid_layer_switching() {
    let db = get_test_db();
    assert!(db.is_valid("MO(0)"));
    assert!(db.is_valid("MO(1)"));
    assert!(db.is_valid("MO(5)")); // Pattern match
    assert!(db.is_valid("TG(0)"));
    assert!(db.is_valid("TO(3)"));
}

#[test]
fn test_is_valid_invalid_keys() {
    let db = get_test_db();
    assert!(!db.is_valid("INVALID_KEY"));
    assert!(!db.is_valid("KC_FOO"));
    assert!(!db.is_valid(""));
}

#[test]
fn test_get_keycode() {
    let db = get_test_db();
    let keycode = db.get("KC_A").unwrap();
    assert_eq!(keycode.code, "KC_A");
    assert_eq!(keycode.name, "A");
    assert_eq!(keycode.category, "basic");
}

#[test]
fn test_get_keycode_by_alias() {
    let db = get_test_db();
    let keycode = db.get("KC_ENTER").unwrap();
    assert_eq!(keycode.code, "KC_ENT");
    assert_eq!(keycode.name, "Enter");
}

#[test]
fn test_search_empty_query() {
    let db = get_test_db();
    let results = db.search("");
    assert_eq!(results.len(), db.keycode_count());
}

#[test]
fn test_search_exact_match() {
    let db = get_test_db();
    let results = db.search("KC_A");
    assert!(!results.is_empty());
    assert_eq!(results[0].code, "KC_A");
}

#[test]
fn test_search_partial_match() {
    let db = get_test_db();
    let results = db.search("arr");
    // Should match arrow keys (KC_LEFT, KC_RIGHT, KC_UP, KC_DOWN)
    assert!(!results.is_empty());
    assert!(results
        .iter()
        .any(|k| k.name.to_lowercase().contains("arrow")));
}

#[test]
fn test_search_case_insensitive() {
    let db = get_test_db();
    let results_upper = db.search("ENTER");
    let results_lower = db.search("enter");
    assert_eq!(results_upper.len(), results_lower.len());
    assert!(!results_upper.is_empty());
}

#[test]
fn test_search_in_category() {
    let db = get_test_db();
    let nav_keys = db.search_in_category("", "navigation");
    assert!(!nav_keys.is_empty());
    assert!(nav_keys.iter().all(|k| k.category == "navigation"));
}

#[test]
fn test_get_category_keycodes() {
    let db = get_test_db();
    let function_keys = db.get_category_keycodes("function");
    assert!(!function_keys.is_empty());
    assert!(function_keys.iter().any(|k| k.code == "KC_F1"));
    assert!(function_keys.iter().any(|k| k.code == "KC_F12"));
}

#[test]
fn test_get_category() {
    let db = get_test_db();
    let category = db.get_category("basic").unwrap();
    assert_eq!(category.id, "basic");
    assert_eq!(category.name, "Basic");
}

#[test]
fn test_categories() {
    let db = get_test_db();
    let categories = db.categories();
    assert!(categories.len() >= 8);
    assert!(categories.iter().any(|c| c.id == "basic"));
    assert!(categories.iter().any(|c| c.id == "navigation"));
    assert!(categories.iter().any(|c| c.id == "media"));
}

#[test]
fn test_is_parameterized() {
    let db = get_test_db();
    // Layer keycodes are parameterized
    assert!(db.is_parameterized("MO()"));
    assert!(db.is_parameterized("LT()"));
    assert!(db.is_parameterized("LM()"));
    // Modifier wrappers are parameterized
    assert!(db.is_parameterized("LCG()"));
    assert!(db.is_parameterized("LCTL()"));
    // Mod-taps are parameterized
    assert!(db.is_parameterized("LCTL_T()"));
    // Basic keys are NOT parameterized
    assert!(!db.is_parameterized("KC_A"));
    assert!(!db.is_parameterized("KC_ENT"));
}

#[test]
fn test_get_params_layer_only() {
    let db = get_test_db();
    let params = db.get_params("MO()").unwrap();
    assert_eq!(params.len(), 1);
    assert_eq!(params[0].param_type, ParamType::Layer);
}

#[test]
fn test_get_params_layer_tap() {
    let db = get_test_db();
    let params = db.get_params("LT()").unwrap();
    assert_eq!(params.len(), 2);
    assert_eq!(params[0].param_type, ParamType::Layer);
    assert_eq!(params[1].param_type, ParamType::Keycode);
}

#[test]
fn test_get_params_layer_mod() {
    let db = get_test_db();
    let params = db.get_params("LM()").unwrap();
    assert_eq!(params.len(), 2);
    assert_eq!(params[0].param_type, ParamType::Layer);
    assert_eq!(params[1].param_type, ParamType::Modifier);
}

#[test]
fn test_get_params_modifier_wrapper() {
    let db = get_test_db();
    let params = db.get_params("LCG()").unwrap();
    assert_eq!(params.len(), 1);
    assert_eq!(params[0].param_type, ParamType::Keycode);
}

#[test]
fn test_get_params_mod_tap() {
    let db = get_test_db();
    let params = db.get_params("LCTL_T()").unwrap();
    assert_eq!(params.len(), 1);
    assert_eq!(params[0].param_type, ParamType::Keycode);
}

#[test]
fn test_get_params_custom_mod_tap() {
    let db = get_test_db();
    let params = db.get_params("MT()").unwrap();
    assert_eq!(params.len(), 2);
    assert_eq!(params[0].param_type, ParamType::Modifier);
    assert_eq!(params[1].param_type, ParamType::Keycode);
}

#[test]
fn test_get_prefix() {
    assert_eq!(KeycodeDb::get_prefix("LCG()"), Some("LCG"));
    assert_eq!(KeycodeDb::get_prefix("LCTL_T()"), Some("LCTL_T"));
    assert_eq!(KeycodeDb::get_prefix("MO()"), Some("MO"));
    assert_eq!(KeycodeDb::get_prefix("KC_A"), None);
}

#[test]
fn test_get_mod_tap_display() {
    let db = get_test_db();
    // Basic modifiers
    assert_eq!(db.get_mod_tap_display("LCTL_T"), Some("CTL"));
    assert_eq!(db.get_mod_tap_display("RCTL_T"), Some("CTL"));
    assert_eq!(db.get_mod_tap_display("LSFT_T"), Some("SFT"));
    assert_eq!(db.get_mod_tap_display("LALT_T"), Some("ALT"));
    assert_eq!(db.get_mod_tap_display("LGUI_T"), Some("GUI"));
    // Combo modifiers
    assert_eq!(db.get_mod_tap_display("MEH_T"), Some("MEH"));
    assert_eq!(db.get_mod_tap_display("HYPR_T"), Some("HYP"));
    assert_eq!(db.get_mod_tap_display("LCAG_T"), Some("CAG"));
    // Aliases
    assert_eq!(db.get_mod_tap_display("CMD_T"), Some("GUI"));
    assert_eq!(db.get_mod_tap_display("OPT_T"), Some("ALT"));
    // Not a mod-tap
    assert_eq!(db.get_mod_tap_display("KC_A"), None);
}

#[test]
fn test_parse_tap_hold_layer_tap() {
    let db = get_test_db();
    let info = db.parse_tap_hold("LT(1, KC_A)").unwrap();
    assert_eq!(info.tap_hold_type, TapHoldType::LayerTap);
    assert_eq!(info.prefix, "LT");
    assert_eq!(info.arg1, "1");
    assert_eq!(info.arg2, Some("KC_A".to_string()));
}

#[test]
fn test_parse_tap_hold_mod_tap() {
    let db = get_test_db();
    let info = db.parse_tap_hold("MT(MOD_LCTL, KC_A)").unwrap();
    assert_eq!(info.tap_hold_type, TapHoldType::ModTap);
    assert_eq!(info.prefix, "MT");
    assert_eq!(info.arg1, "MOD_LCTL");
    assert_eq!(info.arg2, Some("KC_A".to_string()));
}

#[test]
fn test_parse_tap_hold_mod_tap_named() {
    let db = get_test_db();
    let info = db.parse_tap_hold("LCTL_T(KC_A)").unwrap();
    assert_eq!(info.tap_hold_type, TapHoldType::ModTapNamed);
    assert_eq!(info.prefix, "LCTL_T");
    assert_eq!(info.arg1, "KC_A");
    assert_eq!(info.arg2, None);
}

#[test]
fn test_parse_tap_hold_layer_mod() {
    let db = get_test_db();
    let info = db.parse_tap_hold("LM(1, MOD_LCTL)").unwrap();
    assert_eq!(info.tap_hold_type, TapHoldType::LayerMod);
    assert_eq!(info.prefix, "LM");
    assert_eq!(info.arg1, "1");
    assert_eq!(info.arg2, Some("MOD_LCTL".to_string()));
}

#[test]
fn test_parse_tap_hold_swap_hands() {
    let db = get_test_db();
    let info = db.parse_tap_hold("SH_T(KC_A)").unwrap();
    assert_eq!(info.tap_hold_type, TapHoldType::SwapHands);
    assert_eq!(info.prefix, "SH_T");
    assert_eq!(info.arg1, "KC_A");
    assert_eq!(info.arg2, None);
}

#[test]
fn test_parse_tap_hold_not_tap_hold() {
    let db = get_test_db();
    assert!(db.parse_tap_hold("KC_A").is_none());
    assert!(db.parse_tap_hold("MO(1)").is_none());
    assert!(db.parse_tap_hold("TG(2)").is_none());
}

#[test]
fn test_is_layer_keycode() {
    let db = get_test_db();
    // Simple layer keycodes
    assert!(db.is_layer_keycode("MO(1)"));
    assert!(db.is_layer_keycode("TG(2)"));
    assert!(db.is_layer_keycode("TO(0)"));
    assert!(db.is_layer_keycode("DF(1)"));
    assert!(db.is_layer_keycode("OSL(3)"));
    assert!(db.is_layer_keycode("TT(2)"));
    assert!(db.is_layer_keycode("PDF(0)"));
    // Compound layer keycodes
    assert!(db.is_layer_keycode("LT(1, KC_A)"));
    assert!(db.is_layer_keycode("LM(2, MOD_LCTL)"));
    // UUID references
    assert!(db.is_layer_keycode("MO(@abc-123)"));
    assert!(db.is_layer_keycode("LT(@layer-id, KC_SPC)"));
    // Not layer keycodes
    assert!(!db.is_layer_keycode("KC_A"));
    assert!(!db.is_layer_keycode("LCTL_T(KC_A)"));
    assert!(!db.is_layer_keycode("KC_TRNS")); // Non-parameterized
}

#[test]
fn test_parse_layer_keycode_simple() {
    let db = get_test_db();
    let (prefix, layer_ref, suffix) = db.parse_layer_keycode("MO(1)").unwrap();
    assert_eq!(prefix, "MO");
    assert_eq!(layer_ref, "1");
    assert_eq!(suffix, "");
}

#[test]
fn test_parse_layer_keycode_with_uuid() {
    let db = get_test_db();
    let (prefix, layer_ref, suffix) = db.parse_layer_keycode("TG(@abc-123)").unwrap();
    assert_eq!(prefix, "TG");
    assert_eq!(layer_ref, "@abc-123");
    assert_eq!(suffix, "");
}

#[test]
fn test_parse_layer_keycode_compound() {
    let db = get_test_db();
    let (prefix, layer_ref, suffix) = db.parse_layer_keycode("LT(1, KC_A)").unwrap();
    assert_eq!(prefix, "LT");
    assert_eq!(layer_ref, "1");
    assert_eq!(suffix, ", KC_A)");
}

#[test]
fn test_parse_layer_keycode_lm() {
    let db = get_test_db();
    let (prefix, layer_ref, suffix) = db.parse_layer_keycode("LM(@layer-id, MOD_LSFT)").unwrap();
    assert_eq!(prefix, "LM");
    assert_eq!(layer_ref, "@layer-id");
    assert_eq!(suffix, ", MOD_LSFT)");
}

#[test]
fn test_parse_layer_keycode_not_layer() {
    let db = get_test_db();
    assert!(db.parse_layer_keycode("KC_A").is_none());
    assert!(db.parse_layer_keycode("LCTL_T(KC_A)").is_none());
}

#[test]
fn test_parse_tap_dance_keycode_valid() {
    let db = get_test_db();
    assert_eq!(
        db.parse_tap_dance_keycode("TD(esc_caps)"),
        Some("esc_caps".to_string())
    );
    assert_eq!(
        db.parse_tap_dance_keycode("TD(shift_123)"),
        Some("shift_123".to_string())
    );
    assert_eq!(
        db.parse_tap_dance_keycode("TD(MY_TAP_DANCE)"),
        Some("MY_TAP_DANCE".to_string())
    );
}

#[test]
fn test_parse_tap_dance_keycode_invalid() {
    let db = get_test_db();
    // Not a tap dance keycode
    assert_eq!(db.parse_tap_dance_keycode("KC_A"), None);
    assert_eq!(db.parse_tap_dance_keycode("MO(1)"), None);

    // Invalid names (not C identifiers)
    assert_eq!(db.parse_tap_dance_keycode("TD(my-tap)"), None);
    assert_eq!(db.parse_tap_dance_keycode("TD(my tap)"), None);
    assert_eq!(db.parse_tap_dance_keycode("TD(my@tap)"), None);

    // Empty name
    assert_eq!(db.parse_tap_dance_keycode("TD()"), None);
    assert_eq!(db.parse_tap_dance_keycode("TD(  )"), None);
}

#[test]
fn test_get_simple_layer_prefixes() {
    let db = get_test_db();
    let prefixes = db.get_simple_layer_prefixes();
    assert!(prefixes.contains(&"MO"));
    assert!(prefixes.contains(&"TG"));
    assert!(prefixes.contains(&"TO"));
    assert!(prefixes.contains(&"DF"));
    assert!(prefixes.contains(&"OSL"));
    assert!(prefixes.contains(&"TT"));
    assert!(prefixes.contains(&"PDF"));
    // LT and LM are compound, not simple
    assert!(!prefixes.contains(&"LT"));
    assert!(!prefixes.contains(&"LM"));
}

#[test]
fn test_get_compound_layer_prefixes() {
    let db = get_test_db();
    let prefixes = db.get_compound_layer_prefixes();
    assert!(prefixes.contains(&"LT"));
    assert!(prefixes.contains(&"LM"));
    // Simple layer keycodes should not be here
    assert!(!prefixes.contains(&"MO"));
    assert!(!prefixes.contains(&"TG"));
}
