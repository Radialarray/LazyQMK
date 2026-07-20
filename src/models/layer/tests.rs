//! Tests for layer.
//!
//! Auto-extracted from layer.rs.

use super::*;

    use super::*;

    #[test]
    fn test_key_definition_new() {
        let pos = Position::new(0, 0);
        let key = KeyDefinition::new(pos, "KC_A");

        assert_eq!(key.position, pos);
        assert_eq!(key.keycode, "KC_A");
        assert_eq!(key.label, None);
        assert_eq!(key.color_override, None);
        assert_eq!(key.category_id, None);
        assert!(!key.combo_participant);
    }

    #[test]
    fn test_key_definition_builder() {
        let pos = Position::new(0, 0);
        let color = RgbColor::new(255, 0, 0);
        let key = KeyDefinition::new(pos, "KC_A")
            .with_color(color)
            .with_category("navigation")
            .with_label("A");

        assert_eq!(key.color_override, Some(color));
        assert_eq!(key.category_id, Some("navigation".to_string()));
        assert_eq!(key.label, Some("A".to_string()));
    }

    #[test]
    fn test_key_definition_is_transparent() {
        let key = KeyDefinition::new(Position::new(0, 0), "KC_TRNS");
        assert!(key.is_transparent());

        let key = KeyDefinition::new(Position::new(0, 0), "KC_TRANSPARENT");
        assert!(key.is_transparent());

        let key = KeyDefinition::new(Position::new(0, 0), "KC_A");
        assert!(!key.is_transparent());
    }

    #[test]
    fn test_key_definition_is_no_op() {
        let key = KeyDefinition::new(Position::new(0, 0), "KC_NO");
        assert!(key.is_no_op());

        let key = KeyDefinition::new(Position::new(0, 0), "KC_A");
        assert!(!key.is_no_op());
    }

    #[test]
    fn test_layer_new_valid() {
        let color = RgbColor::new(255, 0, 0);
        let layer = Layer::new(0, "Base", color).unwrap();

        assert_eq!(layer.number, 0);
        assert_eq!(layer.name, "Base");
        assert_eq!(layer.default_color, color);
        assert_eq!(layer.category_id, None);
        assert!(layer.keys.is_empty());
    }

    #[test]
    fn test_layer_validate_name() {
        let color = RgbColor::new(255, 0, 0);

        assert!(Layer::new(0, "Base", color).is_ok());
        assert!(Layer::new(0, "A", color).is_ok());
        assert!(Layer::new(0, "", color).is_err());
        assert!(Layer::new(0, "a".repeat(51), color).is_err());
    }

    #[test]
    fn test_layer_add_and_get_key() {
        let mut layer = Layer::new(0, "Base", RgbColor::new(255, 0, 0)).unwrap();
        let pos = Position::new(0, 0);
        let key = KeyDefinition::new(pos, "KC_A");

        layer.add_key(key.clone());
        let retrieved = layer.get_key(pos).unwrap();
        assert_eq!(retrieved, &key);
    }

    #[test]
    fn test_layer_get_key_mut() {
        let mut layer = Layer::new(0, "Base", RgbColor::new(255, 0, 0)).unwrap();
        let pos = Position::new(0, 0);
        let key = KeyDefinition::new(pos, "KC_A");

        layer.add_key(key);
        {
            let key_mut = layer.get_key_mut(pos).unwrap();
            key_mut.keycode = "KC_B".to_string();
        }

        assert_eq!(layer.get_key(pos).unwrap().keycode, "KC_B");
    }

    #[test]
    fn test_layer_set_category() {
        let mut layer = Layer::new(0, "Base", RgbColor::new(255, 0, 0)).unwrap();
        layer.set_category(Some("navigation".to_string()));
        assert_eq!(layer.category_id, Some("navigation".to_string()));

        layer.set_category(None);
        assert_eq!(layer.category_id, None);
    }

    #[test]
    fn test_layer_set_default_color() {
        let mut layer = Layer::new(0, "Base", RgbColor::new(255, 0, 0)).unwrap();
        let new_color = RgbColor::new(0, 255, 0);
        layer.set_default_color(new_color);
        assert_eq!(layer.default_color, new_color);
    }

    #[test]
    fn test_validate_layer_number_default_limit() {
        // Test with default 8-layer limit
        assert!(validate_layer_number(0, 8).is_ok());
        assert!(validate_layer_number(1, 8).is_ok());
        assert!(validate_layer_number(7, 8).is_ok());
        assert!(validate_layer_number(8, 8).is_err());
        assert!(validate_layer_number(255, 8).is_err());
    }

    #[test]
    fn test_validate_layer_number_16bit_limit() {
        // Test with 16-layer limit (LAYER_STATE_16BIT)
        assert!(validate_layer_number(0, 16).is_ok());
        assert!(validate_layer_number(7, 16).is_ok());
        assert!(validate_layer_number(15, 16).is_ok());
        assert!(validate_layer_number(16, 16).is_err());
        assert!(validate_layer_number(255, 16).is_err());
    }

    #[test]
    fn test_validate_layer_number_32bit_limit() {
        // Test with 32-layer limit (LAYER_STATE_32BIT)
        assert!(validate_layer_number(0, 32).is_ok());
        assert!(validate_layer_number(15, 32).is_ok());
        assert!(validate_layer_number(31, 32).is_ok());
        assert!(validate_layer_number(32, 32).is_err());
        assert!(validate_layer_number(255, 32).is_err());
    }

    #[test]
    fn test_layer_new_respects_max_layer_limit() {
        let color = RgbColor::new(255, 0, 0);

        // These should succeed (within MAX_QMK_LAYER_LIMIT of 32)
        assert!(Layer::new(0, "Base", color).is_ok());
        assert!(Layer::new(7, "Lower", color).is_ok());
        assert!(Layer::new(15, "Layer15", color).is_ok());
        assert!(Layer::new(31, "Layer31", color).is_ok());

        // This should fail (equals MAX_QMK_LAYER_LIMIT)
        assert!(Layer::new(32, "Layer32", color).is_err());
        assert!(Layer::new(255, "Layer255", color).is_err());
    }

    #[test]
    fn test_validate_layer_number_error_message() {
        let result = validate_layer_number(10, 8);
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("exceeds maximum"));
        assert!(error_msg.contains("LAYER_STATE_16BIT"));
        assert!(error_msg.contains("LAYER_STATE_32BIT"));
    }
