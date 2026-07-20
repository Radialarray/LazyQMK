//! Tests for keyboard.
//!
//! Auto-extracted from keyboard.rs.

use super::*;

    use super::*;

    #[test]
    fn test_format_simple_keycode() {
        assert_eq!(crate::keycode_db::format::strip_kc_prefix("KC_A"), "A");
        assert_eq!(crate::keycode_db::format::strip_kc_prefix("KC_SPC"), "SPC");
        assert_eq!(
            crate::keycode_db::format::strip_kc_prefix("KC_ENTER"),
            "ENTER"
        );
        assert_eq!(crate::keycode_db::format::strip_kc_prefix("MO(1)"), "MO(1)");
    }

    #[test]
    fn test_format_modifier() {
        assert_eq!(crate::keycode_db::format::format_modifier("MOD_LCTL"), "C");
        assert_eq!(crate::keycode_db::format::format_modifier("MOD_LSFT"), "S");
        assert_eq!(crate::keycode_db::format::format_modifier("MOD_LALT"), "A");
        assert_eq!(crate::keycode_db::format::format_modifier("MOD_LGUI"), "G");
        assert_eq!(
            crate::keycode_db::format::format_modifier("MOD_LCTL | MOD_LSFT"),
            "CS"
        );
        assert_eq!(
            crate::keycode_db::format::format_modifier("MOD_LCTL | MOD_LSFT | MOD_LALT"),
            "CSA"
        );
        assert_eq!(
            crate::keycode_db::format::format_modifier("MOD_LCTL | MOD_LSFT | MOD_LALT | MOD_LGUI"),
            "CSAG"
        );
    }

    #[test]
    fn test_truncate() {
        assert_eq!(KeyboardWidget::truncate("ABC", 5), "ABC");
        assert_eq!(KeyboardWidget::truncate("ABCDEF", 5), "ABCDE");
        assert_eq!(KeyboardWidget::truncate("", 5), "");
    }

    #[test]
    fn test_color_indicator_legend() {
        assert_eq!(
            KeyboardWidget::color_indicator_legend(),
            "i key override  c key category  L layer category  d layer default  - colors off"
        );
    }
