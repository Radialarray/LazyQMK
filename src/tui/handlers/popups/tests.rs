//! Tests for popup handlers.
//!
//! Auto-extracted from the original 1802-line popups.rs. Tests cover
//! combo-edit preservation, tap-dance flow handling, and keycode
//! validation helpers.

use super::*;
use crate::models::{Layout, LayoutMetadata};
use crate::tui::editor::key_editor::{ComboEditPart, ComboKeycodeType as ComboType};
use crate::tui::handlers::popups::{
    extract_td_name, is_basic_or_layer_keycode,
};
use crate::tui::keycode_picker::KeycodePickerEvent;
use crate::tui::AppState;


    fn create_test_state() -> AppState {
        let layout = Layout {
            metadata: LayoutMetadata::default(),
            layers: vec![],
            categories: vec![],
            rgb_enabled: true,
            rgb_brightness: crate::models::RgbBrightness::default(),
            rgb_saturation: crate::models::RgbSaturation::default(),
            rgb_matrix_default_speed: 127,
            rgb_timeout_ms: 0,
            uncolored_key_behavior: crate::models::UncoloredKeyBehavior::default(),
            idle_effect_settings: crate::models::IdleEffectSettings::default(),
            rgb_overlay_ripple: crate::models::RgbOverlayRippleSettings::default(),
            palette_fx: crate::models::PaletteFxSettings::default(),
            tap_hold_settings: crate::models::TapHoldSettings::default(),
            combo_settings: crate::models::ComboSettings::default(),
            tap_dances: vec![],
        };
        let mut state = AppState::new(
            layout,
            None,
            crate::models::KeyboardGeometry::new("test", "test", 4, 12),
            crate::models::VisualLayoutMapping::default(),
            crate::config::Config::default(),
        )
        .unwrap();
        // Add a dummy key to select
        state.selected_position = crate::models::Position::new(0, 0);
        state
    }

    #[test]
    fn test_combo_edit_preserves_hold_behavior() {
        let mut state = create_test_state();

        // Setup: We are editing the TAP part of a Layer Tap (LT)
        state.key_editor_state.combo_edit = Some((
            ComboEditPart::Tap,
            ComboType::LayerTap {
                layer: "1".to_string(),
                tap_key: "KC_TRNS".to_string(),
            },
        ));

        // Action: Select a basic keycode 'KC_A'
        let event = KeycodePickerEvent::KeycodeSelected("KC_A".to_string());

        // Setup layout
        use crate::models::{KeyDefinition, Layer, Position};
        let mut layer = Layer::new(0, "Base", crate::models::RgbColor::default()).unwrap();
        layer.add_key(KeyDefinition::new(Position::new(0, 0), "KC_TRNS"));
        state.layout.layers.push(layer);
        state.current_layer = 0;
        state.selected_position = Position::new(0, 0);

        // Run handler
        let _ = handle_keycode_picker_event(&mut state, event);

        // Assert: The keycode should be LT(1, KC_A), NOT just KC_A
        let key = state.get_selected_key_mut().unwrap();
        assert_eq!(key.keycode, "LT(1, KC_A)");

        // Assert: Combo edit state is cleared
        assert!(state.key_editor_state.combo_edit.is_none());
    }

    #[test]
    fn test_combo_edit_rejects_non_basic_keycode() {
        let mut state = create_test_state();

        // Setup: Editing Tap part
        state.key_editor_state.combo_edit = Some((
            ComboEditPart::Tap,
            ComboType::LayerTap {
                layer: "1".to_string(),
                tap_key: "KC_TRNS".to_string(),
            },
        ));

        // Action: Try to select a parameterized keycode 'MO(1)'
        let event = KeycodePickerEvent::KeycodeSelected("MO(1)".to_string());

        let _ = handle_keycode_picker_event(&mut state, event);

        // Assert: Error set
        assert!(state
            .error_message
            .as_deref()
            .unwrap_or("")
            .contains("Only basic keycodes allowed"));

        // Assert: Combo edit state is PRESERVED (not cleared)
        assert!(state.key_editor_state.combo_edit.is_some());
    }

    #[test]
    fn test_mod_tap_edit_preserves_modifiers() {
        let mut state = create_test_state();

        // Setup: We are editing the TAP part of a Mod Tap (MT)
        // MT(MOD_LSFT, KC_Z)
        state.key_editor_state.combo_edit = Some((
            ComboEditPart::Tap,
            ComboType::ModTapCustom {
                modifier: "MOD_LSFT".to_string(),
                tap_key: "KC_Z".to_string(),
            },
        ));

        // Setup layout
        use crate::models::{KeyDefinition, Layer, Position};
        let mut layer = Layer::new(0, "Base", crate::models::RgbColor::default()).unwrap();
        layer.add_key(KeyDefinition::new(Position::new(0, 0), "KC_TRNS"));
        state.layout.layers.push(layer);
        state.current_layer = 0;
        state.selected_position = Position::new(0, 0);

        // Action: Select 'KC_ENTER'
        let event = KeycodePickerEvent::KeycodeSelected("KC_ENTER".to_string());

        let _ = handle_keycode_picker_event(&mut state, event);

        // Assert: The keycode should be MT(MOD_LSFT, KC_ENTER)
        let key = state.get_selected_key_mut().unwrap();
        assert_eq!(key.keycode, "MT(MOD_LSFT, KC_ENTER)");
    }

    #[test]
    fn test_td_keycode_opens_tap_dance_form() {
        let mut state = create_test_state();

        // Setup layout with a key
        use crate::models::{KeyDefinition, Layer, Position};
        let mut layer = Layer::new(0, "Base", crate::models::RgbColor::default()).unwrap();
        layer.add_key(KeyDefinition::new(Position::new(0, 0), "KC_TRNS"));
        state.layout.layers.push(layer);
        state.current_layer = 0;
        state.selected_position = Position::new(0, 0);

        // Action: Select TD() keycode from picker
        let event = KeycodePickerEvent::KeycodeSelected("TD()".to_string());

        let _ = handle_keycode_picker_event(&mut state, event);

        // Assert: Tap dance form should be open
        assert!(
            matches!(
                state.active_component,
                Some(ActiveComponent::TapDanceForm(_))
            ),
            "Expected TapDanceForm to be active, got: {:?}",
            state.active_component.as_ref().map(std::mem::discriminant)
        );
        assert_eq!(
            state.active_popup,
            Some(PopupType::TapDanceForm),
            "Expected popup to be TapDanceForm"
        );

        // Assert: Context should be FromKeycodePicker
        assert!(
            matches!(
                state.tap_dance_form_context,
                Some(crate::tui::TapDanceFormContext::FromKeycodePicker)
            ),
            "Expected context to be FromKeycodePicker, got: {:?}",
            state.tap_dance_form_context
        );

        // Assert: Pending keycode should be cleared (no further param flow)
        assert!(
            state.pending_keycode.keycode_template.is_none(),
            "Expected pending keycode to be cleared"
        );
    }

    #[test]
    fn test_td_keycode_edit_existing_opens_form() {
        use crate::models::{KeyDefinition, Layer, Position, TapDanceAction};

        let mut state = create_test_state();

        // Setup: Add a tap dance definition
        let td = TapDanceAction::new("slash", "KC_SLSH").with_double_tap("KC_BSLS");
        state.layout.add_tap_dance(td).unwrap();

        // Setup: Add a layer with a key that has this tap dance
        let mut layer = Layer::new(0, "Base", crate::models::RgbColor::default()).unwrap();
        layer.add_key(KeyDefinition::new(Position::new(0, 0), "TD(slash)"));
        state.layout.layers.push(layer);
        state.current_layer = 0;
        state.selected_position = Position::new(0, 0);

        // Action: Select TD() keycode from picker while on a key that already has TD(slash)
        let event = KeycodePickerEvent::KeycodeSelected("TD()".to_string());

        let _ = handle_keycode_picker_event(&mut state, event);

        // Assert: Tap dance form should be open
        assert!(
            matches!(
                state.active_component,
                Some(ActiveComponent::TapDanceForm(_))
            ),
            "Expected TapDanceForm to be active, got: {:?}",
            state.active_component.as_ref().map(std::mem::discriminant)
        );
        assert_eq!(state.active_popup, Some(PopupType::TapDanceForm));
        assert_eq!(
            state.tap_dance_form_context,
            Some(crate::tui::TapDanceFormContext::FromKeycodePicker)
        );
    }

    #[test]
    fn test_extract_td_name() {
        assert_eq!(extract_td_name("TD(slash)"), Some("slash".to_string()));
        assert_eq!(
            extract_td_name("TD(esc_caps)"),
            Some("esc_caps".to_string())
        );
        assert_eq!(
            extract_td_name("TD(my_dance_123)"),
            Some("my_dance_123".to_string())
        );
        assert_eq!(extract_td_name("TD()"), None); // Empty name
        assert_eq!(extract_td_name("KC_A"), None); // Not a TD
        assert_eq!(extract_td_name("TD(incomplete"), None); // Missing closing paren
    }

    #[test]
    fn test_is_basic_or_layer_keycode() {
        // Basic keycodes should be allowed
        assert!(is_basic_or_layer_keycode("KC_A"));
        assert!(is_basic_or_layer_keycode("KC_ENTER"));
        assert!(is_basic_or_layer_keycode("KC_LSFT"));

        // Simple layer keycodes should be allowed
        assert!(is_basic_or_layer_keycode("MO(1)"));
        assert!(is_basic_or_layer_keycode("TG(2)"));
        assert!(is_basic_or_layer_keycode("TO(3)"));
        assert!(is_basic_or_layer_keycode("TT(1)"));
        assert!(is_basic_or_layer_keycode("OSL(2)"));
        assert!(is_basic_or_layer_keycode("DF(0)"));

        // Complex parameterized keycodes should be rejected
        assert!(!is_basic_or_layer_keycode("LT(1, KC_SPC)"));
        assert!(!is_basic_or_layer_keycode("MT(MOD_LCTL, KC_A)"));
        assert!(!is_basic_or_layer_keycode("LM(1, MOD_LSFT)"));
        assert!(!is_basic_or_layer_keycode("LCTL_T(KC_A)"));

        // Layer references should be rejected
        assert!(!is_basic_or_layer_keycode("MO(@layer_id)"));
    }
