//! Tests for layer_picker.
//!
//! Auto-extracted from layer_picker.rs.

use super::*;

    use super::*;
    use crate::models::RgbColor;

    fn create_test_layers() -> Vec<Layer> {
        vec![
            Layer::new(0, "Base", RgbColor::new(255, 255, 255)).unwrap(),
            Layer::new(1, "Navigation", RgbColor::new(0, 255, 0)).unwrap(),
            Layer::new(2, "Symbols", RgbColor::new(255, 0, 0)).unwrap(),
        ]
    }

    #[test]
    fn test_layer_picker_state_new() {
        let state = LayerPickerState::new();
        assert_eq!(state.selected, 0);
        assert!(state.keycode_prefix.is_empty());
        assert!(state.extra_param.is_none());
    }

    #[test]
    fn test_layer_picker_with_prefix() {
        let state = LayerPickerState::with_prefix("MO");
        assert_eq!(state.keycode_prefix, "MO");
        assert!(state.extra_param.is_none());
    }

    #[test]
    fn test_build_keycode_with_extra() {
        let layers = create_test_layers();
        let mut state = LayerPickerState::with_prefix("LT");
        state.extra_param = Some("KC_SPC".to_string());
        state.selected = 2;

        let keycode = state.build_keycode(&layers[2]);
        assert!(keycode.starts_with("LT(@"));
        assert!(keycode.contains(", KC_SPC)"));
        assert!(keycode.contains(&layers[2].id));
    }

    #[test]
    fn test_navigation() {
        let mut state = LayerPickerState::new();
        let layer_count = 3;

        // Navigate down
        state.select_next(layer_count);
        assert_eq!(state.selected, 1);

        state.select_next(layer_count);
        assert_eq!(state.selected, 2);

        // Wrap around
        state.select_next(layer_count);
        assert_eq!(state.selected, 0);

        // Navigate up (wrap)
        state.select_previous(layer_count);
        assert_eq!(state.selected, 2);
    }
