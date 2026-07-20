//! Tests for category.
//!
//! Auto-extracted from category.rs.

use super::*;

    use super::*;

    #[test]
    fn test_validate_hex_color_valid_long() {
        let result = validate_and_parse_hex("#FF0000");
        assert!(result.is_ok());
        let color = result.unwrap();
        assert_eq!(color.r, 255);
        assert_eq!(color.g, 0);
        assert_eq!(color.b, 0);
    }

    #[test]
    fn test_validate_hex_color_valid_short() {
        let result = validate_and_parse_hex("#F0F");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_hex_color_invalid_format() {
        assert!(validate_and_parse_hex("FF0000").is_err());
        assert!(validate_and_parse_hex("#FF00").is_err());
        assert!(validate_and_parse_hex("#GG0000").is_err());
    }

    #[test]
    fn test_category_is_in_use_by_key() {
        use crate::models::{KeyDefinition, Layer, Layout, Position};

        let mut layout = Layout::new("Test").unwrap();
        let mut layer = Layer::new(0, "Base", RgbColor::new(255, 255, 255)).unwrap();

        let mut key = KeyDefinition::new(Position::new(0, 0), "KC_A");
        key = key.with_category("navigation");
        layer.add_key(key);

        layout.add_layer(layer).unwrap();

        assert!(category_is_in_use(&layout, "navigation"));
        assert!(!category_is_in_use(&layout, "symbols"));
    }

    #[test]
    fn test_category_is_in_use_by_layer() {
        use crate::models::{Layer, Layout};

        let mut layout = Layout::new("Test").unwrap();
        let mut layer = Layer::new(0, "Base", RgbColor::new(255, 255, 255)).unwrap();
        layer.category_id = Some("navigation".to_string());

        layout.add_layer(layer).unwrap();

        assert!(category_is_in_use(&layout, "navigation"));
        assert!(!category_is_in_use(&layout, "symbols"));
    }
