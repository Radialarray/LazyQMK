//! Tests for layer_refs.
//!
//! Auto-extracted from layer_refs.rs.

use super::*;

    use super::*;
    use crate::models::{KeyDefinition, Layer, RgbColor};

    #[test]
    fn test_parse_layer_keycode_simple() {
        assert_eq!(
            parse_layer_keycode("MO(1)"),
            Some((LayerRefTarget::Index(1), LayerRefKind::Momentary))
        );
        assert_eq!(
            parse_layer_keycode("TG(2)"),
            Some((LayerRefTarget::Index(2), LayerRefKind::Toggle))
        );
        assert_eq!(
            parse_layer_keycode("TO(0)"),
            Some((LayerRefTarget::Index(0), LayerRefKind::SwitchTo))
        );
        assert_eq!(
            parse_layer_keycode("TT(3)"),
            Some((LayerRefTarget::Index(3), LayerRefKind::TapToggle))
        );
        assert_eq!(
            parse_layer_keycode("OSL(1)"),
            Some((LayerRefTarget::Index(1), LayerRefKind::OneShot))
        );
        assert_eq!(
            parse_layer_keycode("DF(0)"),
            Some((LayerRefTarget::Index(0), LayerRefKind::DefaultSet))
        );
    }

    #[test]
    fn test_parse_layer_keycode_compound() {
        assert_eq!(
            parse_layer_keycode("LT(2, KC_SPC)"),
            Some((LayerRefTarget::Index(2), LayerRefKind::TapHold))
        );
        assert_eq!(
            parse_layer_keycode("LT(1, KC_A)"),
            Some((LayerRefTarget::Index(1), LayerRefKind::TapHold))
        );
        assert_eq!(
            parse_layer_keycode("LM(3, MOD_LSFT)"),
            Some((LayerRefTarget::Index(3), LayerRefKind::LayerMod))
        );

        assert_eq!(
            parse_layer_keycode("LT(1, KC_A)"),
            Some((LayerRefTarget::Index(1), LayerRefKind::TapHold))
        );
        assert_eq!(
            parse_layer_keycode("LM(3, MOD_LSFT)"),
            Some((LayerRefTarget::Index(3), LayerRefKind::LayerMod))
        );
    }

    #[test]
    fn test_parse_layer_keycode_invalid() {
        // Non-layer keycodes
        assert_eq!(parse_layer_keycode("KC_A"), None);
        assert_eq!(parse_layer_keycode("KC_TRNS"), None);
        assert_eq!(parse_layer_keycode("LCTL_T(KC_A)"), None);

        // UUID references should parse to LayerRefTarget::Uuid
        assert_eq!(
            parse_layer_keycode("MO(@layer-id)"),
            Some((
                LayerRefTarget::Uuid("@layer-id".to_string()),
                LayerRefKind::Momentary
            ))
        );
        assert_eq!(
            parse_layer_keycode("LT(@abc-123, KC_SPC)"),
            Some((
                LayerRefTarget::Uuid("@abc-123".to_string()),
                LayerRefKind::TapHold
            ))
        );

        // Malformed
        assert_eq!(parse_layer_keycode("MO("), None);
        assert_eq!(parse_layer_keycode("MO(abc)"), None);
        assert_eq!(parse_layer_keycode("LT(1)"), None); // Missing second param
    }

    #[test]
    fn test_layer_ref_kind_is_hold_like() {
        assert!(LayerRefKind::Momentary.is_hold_like());
        assert!(LayerRefKind::TapHold.is_hold_like());
        assert!(LayerRefKind::TapToggle.is_hold_like());
        assert!(LayerRefKind::LayerMod.is_hold_like());

        assert!(!LayerRefKind::Toggle.is_hold_like());
        assert!(!LayerRefKind::OneShot.is_hold_like());
        assert!(!LayerRefKind::SwitchTo.is_hold_like());
        assert!(!LayerRefKind::DefaultSet.is_hold_like());
    }

    #[test]
    fn test_build_layer_ref_index_basic() {
        let mut layer0 = Layer::new(0, "Base", RgbColor::new(255, 0, 0)).unwrap();
        layer0.add_key(KeyDefinition::new(Position::new(0, 0), "MO(1)"));
        layer0.add_key(KeyDefinition::new(Position::new(0, 1), "LT(2, KC_SPC)"));

        let layer1 = Layer::new(1, "Lower", RgbColor::new(0, 255, 0)).unwrap();
        let layer2 = Layer::new(2, "Raise", RgbColor::new(0, 0, 255)).unwrap();

        let layers = vec![layer0, layer1, layer2];
        let index = build_layer_ref_index(&layers);

        // Layer 1 has one reference
        assert_eq!(index.get(&1).unwrap().len(), 1);
        assert_eq!(index.get(&1).unwrap()[0].kind, LayerRefKind::Momentary);
        assert_eq!(index.get(&1).unwrap()[0].position, Position::new(0, 0));

        // Layer 2 has one reference
        assert_eq!(index.get(&2).unwrap().len(), 1);
        assert_eq!(index.get(&2).unwrap()[0].kind, LayerRefKind::TapHold);
        assert_eq!(index.get(&2).unwrap()[0].position, Position::new(0, 1));

        // Layer 0 has no inbound references
        assert!(!index.contains_key(&0));
    }

    #[test]
    fn test_build_layer_ref_index_multiple_refs() {
        let mut layer0 = Layer::new(0, "Base", RgbColor::new(255, 0, 0)).unwrap();
        layer0.add_key(KeyDefinition::new(Position::new(0, 0), "MO(1)"));
        layer0.add_key(KeyDefinition::new(Position::new(0, 1), "TG(1)"));

        let mut layer1 = Layer::new(1, "Lower", RgbColor::new(0, 255, 0)).unwrap();
        layer1.add_key(KeyDefinition::new(Position::new(1, 0), "MO(1)")); // Self-reference

        let layers = vec![layer0, layer1];
        let index = build_layer_ref_index(&layers);

        // Layer 1 has three references (2 from layer 0, 1 self-reference)
        assert_eq!(index.get(&1).unwrap().len(), 3);
    }

    #[test]
    fn test_build_layer_ref_index_ignores_invalid() {
        let mut layer0 = Layer::new(0, "Base", RgbColor::new(255, 0, 0)).unwrap();
        layer0.add_key(KeyDefinition::new(Position::new(0, 0), "KC_A")); // Not a layer keycode
        layer0.add_key(KeyDefinition::new(Position::new(0, 1), "KC_TRNS")); // Transparent
        layer0.add_key(KeyDefinition::new(Position::new(0, 2), "MO(@uuid)")); // UUID ref
        layer0.add_key(KeyDefinition::new(Position::new(0, 3), "MO(99)")); // Out of bounds

        let layer1 = Layer::new(1, "Lower", RgbColor::new(0, 255, 0)).unwrap();

        let layers = vec![layer0, layer1];
        let index = build_layer_ref_index(&layers);

        // No valid references
        assert!(index.is_empty());
    }

    #[test]
    fn test_is_transparent() {
        assert!(is_transparent("KC_TRNS"));
        assert!(is_transparent("KC_TRANSPARENT"));
        assert!(!is_transparent("KC_A"));
        assert!(!is_transparent("MO(1)"));
    }

    #[test]
    fn test_check_transparency_conflict_no_conflict() {
        let mut index = HashMap::new();
        index.insert(
            1,
            vec![LayerRef {
                from_layer: 0,
                to_layer: 1,
                position: Position::new(0, 0),
                kind: LayerRefKind::Momentary,
                keycode: "MO(1)".to_string(),
            }],
        );

        // No conflict if assigning transparent
        assert!(check_transparency_conflict(1, Position::new(0, 0), "KC_TRNS", &index).is_none());

        // No conflict if position doesn't have references
        assert!(check_transparency_conflict(1, Position::new(1, 1), "KC_A", &index).is_none());

        // No conflict if layer has no inbound references
        assert!(check_transparency_conflict(2, Position::new(0, 0), "KC_A", &index).is_none());
    }

    #[test]
    fn test_check_transparency_conflict_with_conflict() {
        let mut index = HashMap::new();
        index.insert(
            1,
            vec![LayerRef {
                from_layer: 0,
                to_layer: 1,
                position: Position::new(0, 0),
                kind: LayerRefKind::Momentary,
                keycode: "MO(1)".to_string(),
            }],
        );

        // Conflict: non-transparent key at position with hold-like reference
        let warning = check_transparency_conflict(1, Position::new(0, 0), "KC_A", &index);
        assert!(warning.is_some());
        let msg = warning.unwrap();
        assert!(msg.contains("Layer 0"));
        assert!(msg.contains("Momentary"));
    }

    #[test]
    fn test_check_transparency_conflict_only_hold_like() {
        let mut index = HashMap::new();
        index.insert(
            1,
            vec![
                // Hold-like - should trigger warning
                LayerRef {
                    from_layer: 0,
                    to_layer: 1,
                    position: Position::new(0, 0),
                    kind: LayerRefKind::TapHold,
                    keycode: "LT(1, KC_SPC)".to_string(),
                },
                // Not hold-like - should NOT trigger warning
                LayerRef {
                    from_layer: 0,
                    to_layer: 1,
                    position: Position::new(0, 1),
                    kind: LayerRefKind::Toggle,
                    keycode: "TG(1)".to_string(),
                },
            ],
        );

        // Position with TapHold should warn
        let warning = check_transparency_conflict(1, Position::new(0, 0), "KC_A", &index);
        assert!(warning.is_some());

        // Position with Toggle should NOT warn
        let warning = check_transparency_conflict(1, Position::new(0, 1), "KC_A", &index);
        assert!(warning.is_none());
    }

    #[test]
    fn test_check_transparency_conflict_multiple_refs() {
        let mut index = HashMap::new();
        index.insert(
            1,
            vec![
                LayerRef {
                    from_layer: 0,
                    to_layer: 1,
                    position: Position::new(0, 0),
                    kind: LayerRefKind::Momentary,
                    keycode: "MO(1)".to_string(),
                },
                LayerRef {
                    from_layer: 2,
                    to_layer: 1,
                    position: Position::new(0, 0),
                    kind: LayerRefKind::TapHold,
                    keycode: "LT(1, KC_A)".to_string(),
                },
            ],
        );

        let warning = check_transparency_conflict(1, Position::new(0, 0), "KC_B", &index);
        assert!(warning.is_some());
        let msg = warning.unwrap();
        // Should mention both layers
        assert!(msg.contains("Layer 0"));
        assert!(msg.contains("Layer 2"));
    }
