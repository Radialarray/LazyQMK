//! Tests for display.
//!
//! Auto-extracted from display.rs.

use super::*;

use super::*;

fn get_test_db() -> KeycodeDb {
    KeycodeDb::load().expect("Failed to load keycode database")
}

#[test]
fn test_simple_keycode() {
    let db = get_test_db();
    let meta = db.get_display_metadata("KC_A", None, None);
    assert_eq!(meta.display.primary, "A");
    assert!(meta.display.secondary.is_none());
    assert!(meta.display.tertiary.is_none());
    assert_eq!(meta.details.len(), 1);
    assert_eq!(meta.details[0].kind, ActionKind::Simple);
}

#[test]
fn test_transparent_key() {
    let db = get_test_db();
    let meta = db.get_display_metadata("KC_TRNS", None, None);
    assert_eq!(meta.display.primary, "▽");
    assert!(meta.display.secondary.is_none());
}

#[test]
fn test_no_key() {
    let db = get_test_db();
    let meta = db.get_display_metadata("KC_NO", None, None);
    assert_eq!(meta.display.primary, "");
}

#[test]
fn test_layer_tap() {
    let db = get_test_db();
    let meta = db.get_display_metadata("LT(1, KC_ESC)", None, None);
    assert_eq!(meta.display.primary, "ESC");
    assert_eq!(meta.display.secondary, Some("L1".to_string()));
    assert!(meta.display.tertiary.is_none());
    assert_eq!(meta.details.len(), 2);
    assert_eq!(meta.details[0].kind, ActionKind::Tap);
    assert_eq!(meta.details[1].kind, ActionKind::Hold);
}

#[test]
fn test_mod_tap_named() {
    let db = get_test_db();
    let meta = db.get_display_metadata("LCTL_T(KC_A)", None, None);
    assert_eq!(meta.display.primary, "A");
    assert_eq!(meta.display.secondary, Some("CTL".to_string()));
    assert_eq!(meta.details.len(), 2);
    assert_eq!(meta.details[0].kind, ActionKind::Tap);
    assert_eq!(meta.details[1].kind, ActionKind::Hold);
}

#[test]
fn test_mod_tap_custom() {
    let db = get_test_db();
    let meta = db.get_display_metadata("MT(MOD_LCTL, KC_SPC)", None, None);
    assert_eq!(meta.display.primary, "SPC");
    assert_eq!(meta.display.secondary, Some("C".to_string()));
}

#[test]
fn test_momentary_layer() {
    let db = get_test_db();
    let meta = db.get_display_metadata("MO(1)", None, None);
    assert_eq!(meta.display.primary, "▼L1");
    assert!(meta.display.secondary.is_none());
    assert_eq!(meta.details[0].kind, ActionKind::Layer);
}

#[test]
fn test_toggle_layer() {
    let db = get_test_db();
    let meta = db.get_display_metadata("TG(2)", None, None);
    assert_eq!(meta.display.primary, "TG2");
}

#[test]
fn test_tap_dance_with_info() {
    let db = get_test_db();
    let td_info = TapDanceDisplayInfo {
        single_tap: "KC_A".to_string(),
        double_tap: Some("KC_B".to_string()),
        hold: Some("KC_C".to_string()),
    };
    let meta = db.get_display_metadata("TD(my_dance)", Some(&td_info), None);
    assert_eq!(meta.display.primary, "A");
    assert_eq!(meta.display.secondary, Some("B".to_string()));
    assert_eq!(meta.display.tertiary, Some("C".to_string()));
    assert_eq!(meta.details.len(), 3);
}

#[test]
fn test_tap_dance_without_info() {
    let db = get_test_db();
    let meta = db.get_display_metadata("TD(my_dance)", None, None);
    assert_eq!(meta.display.primary, "TD:my_dance");
}

#[test]
fn test_layer_mod() {
    let db = get_test_db();
    let meta = db.get_display_metadata("LM(1, MOD_LCTL)", None, None);
    assert_eq!(meta.display.primary, "L1");
    assert_eq!(meta.display.secondary, Some("C".to_string()));
}

#[test]
fn test_one_shot_modifier() {
    let db = get_test_db();
    let meta = db.get_display_metadata("OSM(MOD_LSFT)", None, None);
    assert_eq!(meta.display.primary, "OSS");
}

#[test]
fn test_modifier_wrapper() {
    let db = get_test_db();
    let meta = db.get_display_metadata("LCTL(KC_C)", None, None);
    assert_eq!(meta.display.primary, "C+C");
}

#[test]
fn test_layer_tap_with_uuid() {
    let db = get_test_db();
    // Build a layer ID to number map
    let mut layer_map = std::collections::HashMap::new();
    layer_map.insert("abc123-uuid".to_string(), 1);

    let meta = db.get_display_metadata("LT(@abc123-uuid, KC_ESC)", None, Some(&layer_map));
    assert_eq!(meta.display.primary, "ESC");
    assert_eq!(meta.display.secondary, Some("L1".to_string()));
    assert_eq!(meta.details[1].code, "Layer 1");
}

#[test]
fn test_momentary_layer_with_uuid() {
    let db = get_test_db();
    // Build a layer ID to number map
    let mut layer_map = std::collections::HashMap::new();
    layer_map.insert("def456-uuid".to_string(), 2);

    let meta = db.get_display_metadata("MO(@def456-uuid)", None, Some(&layer_map));
    assert_eq!(meta.display.primary, "▼L2");
    assert_eq!(
        meta.details[0].description,
        "Momentary: Activate layer 2 while held"
    );
}

#[test]
fn test_default_layer() {
    let db = get_test_db();
    let meta = db.get_display_metadata("DF(1)", None, None);
    assert_eq!(meta.display.primary, "DF1");
    assert!(meta.display.secondary.is_none());
    assert_eq!(meta.details[0].kind, ActionKind::Layer);
    assert_eq!(
        meta.details[0].description,
        "Default: Set layer 1 as default"
    );
}

#[test]
fn test_default_layer_with_uuid() {
    let db = get_test_db();
    let mut layer_map = std::collections::HashMap::new();
    layer_map.insert("layer-uuid-123".to_string(), 0);

    let meta = db.get_display_metadata("DF(@layer-uuid-123)", None, Some(&layer_map));
    assert_eq!(meta.display.primary, "DF0");
    assert_eq!(meta.details[0].code, "Layer 0");
}

#[test]
fn test_tap_toggle_layer() {
    let db = get_test_db();
    let meta = db.get_display_metadata("TT(3)", None, None);
    assert_eq!(meta.display.primary, "TT3");
    assert!(meta.display.secondary.is_none());
    assert_eq!(meta.details[0].kind, ActionKind::Layer);
    assert!(meta.details[0].description.contains("Tap-Toggle"));
}

#[test]
fn test_tap_toggle_layer_with_uuid() {
    let db = get_test_db();
    let mut layer_map = std::collections::HashMap::new();
    layer_map.insert("tt-layer-uuid".to_string(), 2);

    let meta = db.get_display_metadata("TT(@tt-layer-uuid)", None, Some(&layer_map));
    assert_eq!(meta.display.primary, "TT2");
    assert_eq!(meta.details[0].code, "Layer 2");
}

#[test]
fn test_pdf_layer() {
    let db = get_test_db();
    let meta = db.get_display_metadata("PDF(1)", None, None);
    assert_eq!(meta.display.primary, "PDF1");
    assert!(meta.display.secondary.is_none());
    assert_eq!(meta.details[0].kind, ActionKind::Layer);
    assert!(meta.details[0].description.contains("Per-layer Default"));
}

#[test]
fn test_pdf_layer_with_uuid() {
    let db = get_test_db();
    let mut layer_map = std::collections::HashMap::new();
    layer_map.insert("pdf-layer-uuid".to_string(), 4);

    let meta = db.get_display_metadata("PDF(@pdf-layer-uuid)", None, Some(&layer_map));
    assert_eq!(meta.display.primary, "PDF4");
    assert_eq!(meta.details[0].code, "Layer 4");
}
