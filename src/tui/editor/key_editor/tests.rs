//! Tests for key_editor.
//!
//! Auto-extracted from key_editor.rs.

use super::*;

    use super::*;

    #[test]
    fn test_is_key_assigned() {
        assert!(!is_key_assigned("KC_NO"));
        assert!(!is_key_assigned("KC_TRNS"));
        assert!(!is_key_assigned("XXXXXXX"));
        assert!(!is_key_assigned("_______"));
        assert!(!is_key_assigned(""));
        assert!(is_key_assigned("KC_A"));
        assert!(is_key_assigned("LT(1, KC_SPC)"));
    }

    #[test]
    fn test_parse_keycode_with_db_simple() {
        let db = KeycodeDb::load().expect("Failed to load keycode database");

        let result = parse_keycode_with_db(&db, "KC_A");
        assert!(result.is_some());
        let parsed = result.unwrap();
        assert_eq!(parsed.category, "basic");
        assert_eq!(parsed.name, "A");
    }

    #[test]
    fn test_parse_keycode_with_db_mod_combo() {
        let db = KeycodeDb::load().expect("Failed to load keycode database");

        let result = parse_keycode_with_db(&db, "LCG(KC_Q)");
        assert!(result.is_some());
        let parsed = result.unwrap();
        assert_eq!(parsed.category, "mod_combo");
        assert_eq!(parsed.name, "Ctrl+GUI+");
        assert_eq!(parsed.params, vec!["KC_Q"]);
    }

    #[test]
    fn test_parse_keycode_with_db_mod_tap() {
        let db = KeycodeDb::load().expect("Failed to load keycode database");

        let result = parse_keycode_with_db(&db, "LCTL_T(KC_A)");
        assert!(result.is_some());
        let parsed = result.unwrap();
        assert_eq!(parsed.category, "mod_tap");
        assert_eq!(parsed.name, "Ctrl-Tap");
        assert_eq!(parsed.params, vec!["KC_A"]);
    }

    #[test]
    fn test_parse_keycode_with_db_layer_tap() {
        let db = KeycodeDb::load().expect("Failed to load keycode database");

        let result = parse_keycode_with_db(&db, "LT(1, KC_SPC)");
        assert!(result.is_some());
        let parsed = result.unwrap();
        assert_eq!(parsed.category, "layers");
        assert_eq!(parsed.name, "Layer-Tap");
        assert_eq!(parsed.params, vec!["1", "KC_SPC"]);
    }

    #[test]
    fn test_get_keycode_breakdown_mod_tap() {
        let db = KeycodeDb::load().expect("Failed to load keycode database");

        let result = get_keycode_breakdown(&db, "LCTL_T(KC_A)", None);
        assert!(result.is_some());
        let (label1, val1, label2, val2) = result.unwrap();
        assert_eq!(label1, "Hold");
        assert_eq!(val1, "Ctrl");
        assert_eq!(label2, "Tap");
        assert_eq!(val2, "KC_A");
    }

    #[test]
    fn test_get_keycode_breakdown_mod_combo() {
        let db = KeycodeDb::load().expect("Failed to load keycode database");

        let result = get_keycode_breakdown(&db, "LCG(KC_Q)", None);
        assert!(result.is_some());
        let (label1, val1, label2, val2) = result.unwrap();
        assert_eq!(label1, "Modifier");
        assert_eq!(val1, "Ctrl+GUI+");
        assert_eq!(label2, "Key");
        assert_eq!(val2, "KC_Q");
    }

    #[test]
    fn test_get_keycode_breakdown_layer_tap() {
        let db = KeycodeDb::load().expect("Failed to load keycode database");

        let result = get_keycode_breakdown(&db, "LT(2, KC_SPC)", None);
        assert!(result.is_some());
        let (label1, val1, label2, val2) = result.unwrap();
        assert_eq!(label1, "Hold");
        assert_eq!(val1, "Layer 2");
        assert_eq!(label2, "Tap");
        assert_eq!(val2, "KC_SPC");
    }

    #[test]
    fn test_get_keycode_breakdown_simple_keycode() {
        let db = KeycodeDb::load().expect("Failed to load keycode database");

        // Simple keycodes should return None (no breakdown needed)
        let result = get_keycode_breakdown(&db, "KC_A", None);
        assert!(result.is_none());
    }

    #[test]
    fn test_get_keycode_breakdown_layer_mod() {
        let db = KeycodeDb::load().expect("Failed to load keycode database");

        let result = get_keycode_breakdown(&db, "LM(1, MOD_LSFT)", None);
        assert!(result.is_some());
        let (label1, val1, label2, val2) = result.unwrap();
        assert_eq!(label1, "Layer");
        assert_eq!(val1, "1");
        assert_eq!(label2, "Modifier");
        assert_eq!(val2, "MOD_LSFT");
    }

    #[test]
    fn test_get_keycode_breakdown_custom_mod_tap() {
        let db = KeycodeDb::load().expect("Failed to load keycode database");

        let result = get_keycode_breakdown(&db, "MT(MOD_LCTL, KC_A)", None);
        assert!(result.is_some());
        let (label1, val1, label2, val2) = result.unwrap();
        assert_eq!(label1, "Hold");
        // MT() is named "Mod-Tap (Custom)" in the DB, trim_end_matches("-Tap") doesn't apply
        assert_eq!(val1, "Mod-Tap (Custom)");
        assert_eq!(label2, "Tap");
        assert_eq!(val2, "KC_A");
    }

    #[test]
    fn test_get_keycode_breakdown_tap_dance() {
        use crate::models::{Layout, TapDanceAction};

        let db = KeycodeDb::load().expect("Failed to load keycode database");
        let mut layout = Layout::new("Test").unwrap();

        // Add a tap dance definition
        let td = TapDanceAction::new("slash", "KC_SLSH")
            .with_double_tap("KC_BSLS")
            .with_hold("KC_QUES");
        layout.add_tap_dance(td).unwrap();

        let result = get_keycode_breakdown(&db, "TD(slash)", Some(&layout));
        assert!(result.is_some());
        let (label1, val1, label2, val2) = result.unwrap();
        assert_eq!(label1, "Single/Double");
        assert_eq!(val1, "KC_SLSH  /  KC_BSLS");
        assert_eq!(label2, "Hold");
        assert_eq!(val2, "Hold: KC_QUES");
    }

    #[test]
    fn test_get_keycode_breakdown_tap_dance_two_way() {
        use crate::models::{Layout, TapDanceAction};

        let db = KeycodeDb::load().expect("Failed to load keycode database");
        let mut layout = Layout::new("Test").unwrap();

        // Add a 2-way tap dance (no hold)
        let td = TapDanceAction::new("esc_caps", "KC_ESC").with_double_tap("KC_CAPS");
        layout.add_tap_dance(td).unwrap();

        let result = get_keycode_breakdown(&db, "TD(esc_caps)", Some(&layout));
        assert!(result.is_some());
        let (label1, val1, label2, val2) = result.unwrap();
        assert_eq!(label1, "Single/Double");
        assert_eq!(val1, "KC_ESC  /  KC_CAPS");
        assert_eq!(label2, "Hold");
        assert_eq!(val2, "(2-way)");
    }

    #[test]
    fn test_extract_td_name() {
        assert_eq!(extract_td_name("TD(slash)"), Some("slash".to_string()));
        assert_eq!(
            extract_td_name("TD(esc_caps)"),
            Some("esc_caps".to_string())
        );
        assert_eq!(extract_td_name("TD()"), None); // Empty name
        assert_eq!(extract_td_name("KC_A"), None); // Not a TD
    }
