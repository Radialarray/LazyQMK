//! Tests for tui module.
//!
//! Auto-extracted from src/tui/mod.rs.
use super::*;

    use super::*;

    #[test]
    fn test_extract_base_keyboard_no_variant() {
        let result = AppState::extract_base_keyboard("crkbd");
        assert_eq!(result, "crkbd");
    }

    #[test]
    fn test_extract_base_keyboard_with_manufacturer() {
        let result = AppState::extract_base_keyboard("keebart/corne_choc_pro");
        assert_eq!(result, "keebart/corne_choc_pro");
    }

    #[test]
    fn test_extract_base_keyboard_standard_variant() {
        let result = AppState::extract_base_keyboard("keebart/corne_choc_pro/standard");
        assert_eq!(result, "keebart/corne_choc_pro");
    }

    #[test]
    fn test_extract_base_keyboard_mini_variant() {
        let result = AppState::extract_base_keyboard("keebart/corne_choc_pro/mini");
        assert_eq!(result, "keebart/corne_choc_pro");
    }

    #[test]
    fn test_extract_base_keyboard_normal_variant() {
        let result = AppState::extract_base_keyboard("manufacturer/keyboard/normal");
        assert_eq!(result, "manufacturer/keyboard");
    }

    #[test]
    fn test_extract_base_keyboard_full_variant() {
        let result = AppState::extract_base_keyboard("manufacturer/keyboard/full");
        assert_eq!(result, "manufacturer/keyboard");
    }

    #[test]
    fn test_extract_base_keyboard_compact_variant() {
        let result = AppState::extract_base_keyboard("manufacturer/keyboard/compact");
        assert_eq!(result, "manufacturer/keyboard");
    }

    #[test]
    fn test_extract_base_keyboard_non_variant_subdirectory() {
        // "custom" is not a recognized variant pattern, so it should be kept
        let result = AppState::extract_base_keyboard("manufacturer/keyboard/custom");
        assert_eq!(result, "manufacturer/keyboard/custom");
    }

    #[test]
    fn test_extract_base_keyboard_revision_variant() {
        // "rev2" IS recognized as a variant pattern (starts with "rev"), so it's stripped
        let result = AppState::extract_base_keyboard("manufacturer/keyboard/rev2");
        assert_eq!(result, "manufacturer/keyboard");
    }

    #[test]
    fn test_extract_base_keyboard_deep_path_with_variant() {
        let result = AppState::extract_base_keyboard("a/b/c/standard");
        assert_eq!(result, "a/b/c");
    }

    // === PendingKeycodeState Tests ===

    #[test]
    fn test_pending_keycode_new() {
        let state = PendingKeycodeState::new();
        assert!(state.keycode_template.is_none());
        assert!(state.params.is_empty());
    }

    #[test]
    fn test_pending_keycode_reset() {
        let mut state = PendingKeycodeState::new();
        state.keycode_template = Some("LT()".to_string());
        state.params.push("@layer-uuid".to_string());
        state.params.push("KC_SPC".to_string());

        state.reset();

        assert!(state.keycode_template.is_none());
        assert!(state.params.is_empty());
    }

    #[test]
    fn test_build_keycode_layer_tap() {
        let mut state = PendingKeycodeState::new();
        state.keycode_template = Some("LT()".to_string());
        state.params.push("@abc-123".to_string());
        state.params.push("KC_SPC".to_string());

        let result = state.build_keycode();
        assert_eq!(result, Some("LT(@abc-123, KC_SPC)".to_string()));
    }

    #[test]
    fn test_build_keycode_mod_tap() {
        let mut state = PendingKeycodeState::new();
        state.keycode_template = Some("MT()".to_string());
        state.params.push("MOD_LCTL | MOD_LSFT".to_string());
        state.params.push("KC_A".to_string());

        let result = state.build_keycode();
        assert_eq!(result, Some("MT(MOD_LCTL | MOD_LSFT, KC_A)".to_string()));
    }

    #[test]
    fn test_build_keycode_layer_mod() {
        let mut state = PendingKeycodeState::new();
        state.keycode_template = Some("LM()".to_string());
        state.params.push("@layer-uuid".to_string());
        state.params.push("MOD_LSFT".to_string());

        let result = state.build_keycode();
        assert_eq!(result, Some("LM(@layer-uuid, MOD_LSFT)".to_string()));
    }

    #[test]
    fn test_build_keycode_swap_hands_tap() {
        let mut state = PendingKeycodeState::new();
        state.keycode_template = Some("SH_T()".to_string());
        state.params.push("KC_SPC".to_string());

        let result = state.build_keycode();
        assert_eq!(result, Some("SH_T(KC_SPC)".to_string()));
    }

    #[test]
    fn test_build_keycode_incomplete_lt() {
        let mut state = PendingKeycodeState::new();
        state.keycode_template = Some("LT()".to_string());
        state.params.push("@abc-123".to_string());
        // Missing second parameter

        let result = state.build_keycode();
        // Should still build with one param (though it's invalid QMK)
        assert_eq!(result, Some("LT(@abc-123)".to_string()));
    }

    #[test]
    fn test_build_keycode_no_template() {
        let state = PendingKeycodeState::new();
        let result = state.build_keycode();
        assert!(result.is_none(), "No template should return None");
    }

    #[test]
    fn test_build_keycode_empty_params() {
        let mut state = PendingKeycodeState::new();
        state.keycode_template = Some("LT()".to_string());
        // No params added

        let result = state.build_keycode();
        assert!(result.is_none(), "Empty params should return None");
    }
