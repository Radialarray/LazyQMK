//! Tests for tap_dance_docs.
//!
//! Auto-extracted from tap_dance_docs.rs.

use super::*;

    use super::*;
    use crate::models::{KeyDefinition, Layer, Position, RgbColor, TapDanceAction};

    #[test]
    fn test_generate_tap_dance_docs_empty() {
        let layout = Layout::new("Test").unwrap();
        let db = KeycodeDb::load().unwrap();
        let output = generate_tap_dance_docs(&layout, &db);
        assert!(output.is_empty());
    }

    #[test]
    fn test_generate_tap_dance_docs_single_tap_only() {
        let mut layout = Layout::new("Test").unwrap();

        let tap_dance = TapDanceAction::new("simple_dance", "KC_A");
        layout.add_tap_dance(tap_dance).unwrap();

        let db = KeycodeDb::load().unwrap();
        let output = generate_tap_dance_docs(&layout, &db);

        assert!(output.contains("## Tap Dance Actions"));
        assert!(output.contains("### TD(0): simple_dance"));
        assert!(output.contains("- **Single Tap:** KC_A"));
        assert!(!output.contains("- **Double Tap:"));
        assert!(!output.contains("- **Hold:"));
    }

    #[test]
    fn test_generate_tap_dance_docs_two_way() {
        let mut layout = Layout::new("Test").unwrap();

        let tap_dance = TapDanceAction::new("quote_dance", "KC_QUOT").with_double_tap("KC_DQUO");
        layout.add_tap_dance(tap_dance).unwrap();

        let db = KeycodeDb::load().unwrap();
        let output = generate_tap_dance_docs(&layout, &db);

        assert!(output.contains("### TD(0): quote_dance"));
        assert!(output.contains("- **Single Tap:** KC_QUOT"));
        assert!(output.contains("- **Double Tap:** KC_DQUO"));
        assert!(!output.contains("- **Hold:"));
    }

    #[test]
    fn test_generate_tap_dance_docs_three_way() {
        let mut layout = Layout::new("Test").unwrap();

        let tap_dance = TapDanceAction::new("bracket_dance", "KC_LBRC")
            .with_double_tap("KC_RBRC")
            .with_hold("KC_LSFT");
        layout.add_tap_dance(tap_dance).unwrap();

        let db = KeycodeDb::load().unwrap();
        let output = generate_tap_dance_docs(&layout, &db);

        assert!(output.contains("### TD(0): bracket_dance"));
        assert!(output.contains("- **Single Tap:** KC_LBRC"));
        assert!(output.contains("- **Double Tap:** KC_RBRC"));
        assert!(output.contains("- **Hold:** KC_LSFT"));
    }

    #[test]
    fn test_generate_tap_dance_docs_with_key_references() {
        let mut layout = Layout::new("Test").unwrap();

        // Add tap dance
        let tap_dance = TapDanceAction::new("quote_dance", "KC_QUOT").with_double_tap("KC_DQUO");
        layout.add_tap_dance(tap_dance).unwrap();

        // Add layer with keys referencing the tap dance
        let mut layer = Layer::new(0, "Base", RgbColor::new(255, 255, 255)).unwrap();
        let key_ref = KeyDefinition::new(Position::new(2, 3), "TD(quote_dance)");
        layer.add_key(key_ref);
        layout.add_layer(layer).unwrap();

        let db = KeycodeDb::load().unwrap();
        let output = generate_tap_dance_docs(&layout, &db);

        assert!(output.contains("**Keys Using Tap Dance:**"));
        assert!(output.contains("- Layer 0, Position (2,3): TD(quote_dance)"));
    }

    #[test]
    fn test_generate_tap_dance_docs_multiple_references() {
        let mut layout = Layout::new("Test").unwrap();

        // Add two tap dances
        let td1 = TapDanceAction::new("td1", "KC_A").with_double_tap("KC_B");
        let td2 = TapDanceAction::new("td2", "KC_C").with_double_tap("KC_D");
        layout.add_tap_dance(td1).unwrap();
        layout.add_tap_dance(td2).unwrap();

        // Add layer with multiple references
        let mut layer = Layer::new(0, "Base", RgbColor::new(255, 255, 255)).unwrap();
        layer.add_key(KeyDefinition::new(Position::new(0, 0), "TD(td1)"));
        layer.add_key(KeyDefinition::new(Position::new(1, 5), "TD(td2)"));
        layout.add_layer(layer).unwrap();

        let db = KeycodeDb::load().unwrap();
        let output = generate_tap_dance_docs(&layout, &db);

        assert!(output.contains("Layer 0, Position (0,0): TD(td1)"));
        assert!(output.contains("Layer 0, Position (1,5): TD(td2)"));
    }

    #[test]
    fn test_generate_tap_dance_docs_multiple_layers() {
        let mut layout = Layout::new("Test").unwrap();

        // Add tap dance
        let tap_dance = TapDanceAction::new("dance", "KC_A");
        layout.add_tap_dance(tap_dance).unwrap();

        // Add layer 0
        let mut layer0 = Layer::new(0, "Base", RgbColor::new(255, 255, 255)).unwrap();
        layer0.add_key(KeyDefinition::new(Position::new(0, 0), "KC_A"));
        layer0.add_key(KeyDefinition::new(Position::new(0, 1), "TD(dance)"));
        layout.add_layer(layer0).unwrap();

        // Add layer 1
        let mut layer1 = Layer::new(1, "Alt", RgbColor::new(255, 0, 0)).unwrap();
        layer1.add_key(KeyDefinition::new(Position::new(0, 0), "KC_B"));
        layer1.add_key(KeyDefinition::new(Position::new(1, 3), "TD(dance)"));
        layout.add_layer(layer1).unwrap();

        let db = KeycodeDb::load().unwrap();
        let output = generate_tap_dance_docs(&layout, &db);

        assert!(output.contains("Layer 0, Position (0,1): TD(dance)"));
        assert!(output.contains("Layer 1, Position (1,3): TD(dance)"));
    }

    #[test]
    fn test_generate_tap_dance_docs_no_references() {
        let mut layout = Layout::new("Test").unwrap();

        // Add tap dance but no keys referencing it
        let tap_dance = TapDanceAction::new("unused_dance", "KC_A");
        layout.add_tap_dance(tap_dance).unwrap();

        // Add a regular key
        let mut layer = Layer::new(0, "Base", RgbColor::new(255, 255, 255)).unwrap();
        layer.add_key(KeyDefinition::new(Position::new(0, 0), "KC_B"));
        layout.add_layer(layer).unwrap();

        let db = KeycodeDb::load().unwrap();
        let output = generate_tap_dance_docs(&layout, &db);

        // Should have tap dance definition but not "Keys Using Tap Dance" section
        assert!(output.contains("### TD(0): unused_dance"));
        assert!(!output.contains("**Keys Using Tap Dance:**"));
    }
