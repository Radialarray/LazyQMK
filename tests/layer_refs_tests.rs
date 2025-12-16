//! Integration tests for layer reference tracking feature

use lazyqmk::models::{KeyDefinition, Layer, Position, RgbColor};
use lazyqmk::services::layer_refs::{
    build_layer_ref_index, check_transparency_conflict, parse_layer_keycode, LayerRefKind,
    LayerRefTarget,
};

#[test]
fn test_layer_refs_basic_flow() {
    // Create a simple 2-layer layout
    let mut layer0 = Layer::new(0, "Base", RgbColor::new(255, 0, 0)).unwrap();
    layer0.add_key(KeyDefinition::new(Position::new(0, 0), "KC_A"));
    layer0.add_key(KeyDefinition::new(Position::new(0, 1), "MO(1)")); // Momentary to layer 1
    layer0.add_key(KeyDefinition::new(Position::new(0, 2), "LT(1, KC_SPC)")); // Layer-tap to layer 1

    let mut layer1 = Layer::new(1, "Lower", RgbColor::new(0, 255, 0)).unwrap();
    layer1.add_key(KeyDefinition::new(Position::new(0, 0), "KC_1"));
    layer1.add_key(KeyDefinition::new(Position::new(0, 1), "KC_TRNS")); // Transparent
    layer1.add_key(KeyDefinition::new(Position::new(0, 2), "KC_2"));

    let layers = vec![layer0, layer1];

    // Build layer reference index
    let layer_refs = build_layer_ref_index(&layers);

    // Layer 1 should have 2 inbound references from layer 0
    let refs = layer_refs.get(&1).expect("Layer 1 should have refs");
    assert_eq!(refs.len(), 2);

    // Check the references are correct
    let mo_ref = refs.iter().find(|r| r.position == Position::new(0, 1));
    assert!(mo_ref.is_some());
    assert_eq!(mo_ref.unwrap().kind, LayerRefKind::Momentary);

    let lt_ref = refs.iter().find(|r| r.position == Position::new(0, 2));
    assert!(lt_ref.is_some());
    assert_eq!(lt_ref.unwrap().kind, LayerRefKind::TapHold);
}

#[test]
fn test_transparency_warning() {
    // Create a layout with a hold-like reference
    let mut layer0 = Layer::new(0, "Base", RgbColor::new(255, 0, 0)).unwrap();
    layer0.add_key(KeyDefinition::new(Position::new(1, 0), "LT(1, KC_SPACE)"));

    let layer1 = Layer::new(1, "Lower", RgbColor::new(0, 255, 0)).unwrap();

    let layers = vec![layer0, layer1];
    let layer_refs = build_layer_ref_index(&layers);

    // Assigning non-transparent key at position (1,0) on layer 1 should warn
    let warning = check_transparency_conflict(1, Position::new(1, 0), "KC_A", &layer_refs);
    assert!(warning.is_some());
    assert!(warning.unwrap().contains("Warning"));

    // Assigning transparent key should not warn
    let warning = check_transparency_conflict(1, Position::new(1, 0), "KC_TRNS", &layer_refs);
    assert!(warning.is_none());

    // Assigning at different position should not warn
    let warning = check_transparency_conflict(1, Position::new(2, 0), "KC_A", &layer_refs);
    assert!(warning.is_none());
}

#[test]
fn test_parse_all_layer_keycodes() {
    // Test parsing all supported layer keycode types
    assert_eq!(
        parse_layer_keycode("MO(1)"),
        Some((LayerRefTarget::Index(1), LayerRefKind::Momentary))
    );
    assert_eq!(
        parse_layer_keycode("TG(2)"),
        Some((LayerRefTarget::Index(2), LayerRefKind::Toggle))
    );
    assert_eq!(
        parse_layer_keycode("TO(3)"),
        Some((LayerRefTarget::Index(3), LayerRefKind::SwitchTo))
    );
    assert_eq!(
        parse_layer_keycode("TT(4)"),
        Some((LayerRefTarget::Index(4), LayerRefKind::TapToggle))
    );
    assert_eq!(
        parse_layer_keycode("OSL(5)"),
        Some((LayerRefTarget::Index(5), LayerRefKind::OneShot))
    );
    assert_eq!(
        parse_layer_keycode("DF(0)"),
        Some((LayerRefTarget::Index(0), LayerRefKind::DefaultSet))
    );
    assert_eq!(
        parse_layer_keycode("LT(1, KC_A)"),
        Some((LayerRefTarget::Index(1), LayerRefKind::TapHold))
    );
    assert_eq!(
        parse_layer_keycode("LM(2, MOD_LCTL)"),
        Some((LayerRefTarget::Index(2), LayerRefKind::LayerMod))
    );

    // Non-layer keycodes
    assert_eq!(parse_layer_keycode("KC_A"), None);
    assert_eq!(parse_layer_keycode("LCTL_T(KC_A)"), None);

    // UUID references should parse
    assert_eq!(
        parse_layer_keycode("MO(@layer-id)"),
        Some((
            LayerRefTarget::Uuid("@layer-id".to_string()),
            LayerRefKind::Momentary
        ))
    );
}

#[test]
fn test_layer_refs_self_reference() {
    // Layer can reference itself
    let mut layer0 = Layer::new(0, "Base", RgbColor::new(255, 0, 0)).unwrap();
    layer0.add_key(KeyDefinition::new(Position::new(0, 0), "TG(0)")); // Toggle self

    let layers = vec![layer0];
    let layer_refs = build_layer_ref_index(&layers);

    // Layer 0 should have 1 self-reference
    let refs = layer_refs.get(&0).expect("Layer 0 should have self-ref");
    assert_eq!(refs.len(), 1);
    assert_eq!(refs[0].from_layer, 0);
    assert_eq!(refs[0].to_layer, 0);
}

#[test]
fn test_layer_refs_ignores_out_of_bounds() {
    // References to non-existent layers should be ignored
    let mut layer0 = Layer::new(0, "Base", RgbColor::new(255, 0, 0)).unwrap();
    layer0.add_key(KeyDefinition::new(Position::new(0, 0), "MO(99)")); // Layer 99 doesn't exist

    let layers = vec![layer0];
    let layer_refs = build_layer_ref_index(&layers);

    // No references should be tracked
    assert!(layer_refs.is_empty());
}

#[test]
fn test_multiple_hold_like_refs_at_same_position() {
    // Multiple layers can reference the same position on another layer
    let mut layer0 = Layer::new(0, "Base", RgbColor::new(255, 0, 0)).unwrap();
    layer0.add_key(KeyDefinition::new(Position::new(0, 0), "MO(2)"));

    let mut layer1 = Layer::new(1, "Lower", RgbColor::new(0, 255, 0)).unwrap();
    layer1.add_key(KeyDefinition::new(Position::new(0, 0), "LT(2, KC_SPC)"));

    let layer2 = Layer::new(2, "Raise", RgbColor::new(0, 0, 255)).unwrap();

    let layers = vec![layer0, layer1, layer2];
    let layer_refs = build_layer_ref_index(&layers);

    // Layer 2 should have 2 references at position (0,0)
    let refs = layer_refs.get(&2).expect("Layer 2 should have refs");
    assert_eq!(refs.len(), 2);

    // Warning should mention both layers
    let warning = check_transparency_conflict(2, Position::new(0, 0), "KC_A", &layer_refs);
    assert!(warning.is_some());
    let msg = warning.unwrap();
    assert!(msg.contains("Layer 0"));
    assert!(msg.contains("Layer 1"));
}
