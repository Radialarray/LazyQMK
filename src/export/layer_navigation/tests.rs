//! Tests for layer_navigation.
//!
//! Auto-extracted from layer_navigation.rs.

use super::*;

use super::*;
use crate::models::{KeyDefinition, Layer, Position, RgbColor};

#[test]
fn test_generate_layer_navigation_no_refs() {
    let mut layout = Layout::new("Test").unwrap();
    let mut layer0 = Layer::new(0, "Base", RgbColor::new(255, 0, 0)).unwrap();
    layer0.add_key(KeyDefinition::new(Position::new(0, 0), "KC_A"));
    layout.add_layer(layer0).unwrap();

    let output = generate_layer_navigation(&layout);

    assert!(output.contains("## Layer Navigation"));
    assert!(output.contains("Layer 0 (Base)"));
    assert!(output.contains("[No outbound references]"));
}

#[test]
fn test_generate_layer_navigation_single_ref() {
    let mut layout = Layout::new("Test").unwrap();

    let mut layer0 = Layer::new(0, "Base", RgbColor::new(255, 0, 0)).unwrap();
    layer0.add_key(KeyDefinition::new(Position::new(0, 0), "MO(1)"));
    layout.add_layer(layer0).unwrap();

    let layer1 = Layer::new(1, "Lower", RgbColor::new(0, 255, 0)).unwrap();
    layout.add_layer(layer1).unwrap();

    let output = generate_layer_navigation(&layout);

    assert!(output.contains("Layer 0 (Base)"));
    assert!(output.contains("└─→ MO(1) on (0,0) → Layer 1 (Lower)"));
    assert!(output.contains("Layer 1 (Lower)"));
    assert!(output.contains("[No outbound references]"));
}

#[test]
fn test_generate_layer_navigation_multiple_refs() {
    let mut layout = Layout::new("Test").unwrap();

    let mut layer0 = Layer::new(0, "Base", RgbColor::new(255, 0, 0)).unwrap();
    layer0.add_key(KeyDefinition::new(Position::new(0, 0), "MO(1)"));
    layer0.add_key(KeyDefinition::new(Position::new(0, 1), "LT(2, KC_SPC)"));
    layer0.add_key(KeyDefinition::new(Position::new(0, 2), "TG(3)"));
    layout.add_layer(layer0).unwrap();

    let layer1 = Layer::new(1, "Lower", RgbColor::new(0, 255, 0)).unwrap();
    layout.add_layer(layer1).unwrap();

    let layer2 = Layer::new(2, "Raise", RgbColor::new(0, 0, 255)).unwrap();
    layout.add_layer(layer2).unwrap();

    let layer3 = Layer::new(3, "Adjust", RgbColor::new(255, 255, 0)).unwrap();
    layout.add_layer(layer3).unwrap();

    let output = generate_layer_navigation(&layout);

    assert!(output.contains("Layer 0 (Base)"));
    assert!(output.contains("├─→ MO(1) on (0,0) → Layer 1 (Lower)"));
    assert!(output.contains("├─→ LT(2, KC_SPC) on (0,1) → Layer 2 (Raise)"));
    assert!(output.contains("└─→ TG(3) on (0,2) → Layer 3 (Adjust)"));
}

#[test]
fn test_generate_layer_navigation_cross_references() {
    let mut layout = Layout::new("Test").unwrap();

    let mut layer0 = Layer::new(0, "Base", RgbColor::new(255, 0, 0)).unwrap();
    layer0.add_key(KeyDefinition::new(Position::new(0, 0), "MO(1)"));
    layout.add_layer(layer0).unwrap();

    let mut layer1 = Layer::new(1, "Lower", RgbColor::new(0, 255, 0)).unwrap();
    layer1.add_key(KeyDefinition::new(Position::new(1, 0), "TO(0)")); // Back to base
    layout.add_layer(layer1).unwrap();

    let output = generate_layer_navigation(&layout);

    assert!(output.contains("Layer 0 (Base)"));
    assert!(output.contains("└─→ MO(1) on (0,0) → Layer 1 (Lower)"));
    assert!(output.contains("Layer 1 (Lower)"));
    assert!(output.contains("└─→ TO(0) on (1,0) → Layer 0 (Base)"));
}

#[test]
fn test_generate_layer_navigation_ignores_transparent() {
    let mut layout = Layout::new("Test").unwrap();

    let mut layer0 = Layer::new(0, "Base", RgbColor::new(255, 0, 0)).unwrap();
    layer0.add_key(KeyDefinition::new(Position::new(0, 0), "KC_TRNS"));
    layer0.add_key(KeyDefinition::new(Position::new(0, 1), "KC_NO"));
    layer0.add_key(KeyDefinition::new(Position::new(0, 2), "MO(1)"));
    layout.add_layer(layer0).unwrap();

    let layer1 = Layer::new(1, "Lower", RgbColor::new(0, 255, 0)).unwrap();
    layout.add_layer(layer1).unwrap();

    let output = generate_layer_navigation(&layout);

    // Should only show MO(1), not KC_TRNS or KC_NO
    assert!(output.contains("└─→ MO(1) on (0,2) → Layer 1 (Lower)"));
    assert!(!output.contains("KC_TRNS"));
    assert!(!output.contains("KC_NO"));
}

#[test]
fn test_generate_layer_navigation_invalid_layer_ref() {
    let mut layout = Layout::new("Test").unwrap();

    let mut layer0 = Layer::new(0, "Base", RgbColor::new(255, 0, 0)).unwrap();
    layer0.add_key(KeyDefinition::new(Position::new(0, 0), "MO(99)")); // Out of bounds
    layout.add_layer(layer0).unwrap();

    let output = generate_layer_navigation(&layout);

    // Should show no outbound references since layer 99 doesn't exist
    assert!(output.contains("[No outbound references]"));
    assert!(!output.contains("MO(99)"));
}

#[test]
fn test_format_key_position() {
    assert_eq!(format_key_position(Position::new(0, 0)), "(0,0)");
    assert_eq!(format_key_position(Position::new(2, 5)), "(2,5)");
}

#[test]
fn test_generate_layer_navigation_duplicate_refs() {
    let mut layout = Layout::new("Test").unwrap();

    let mut layer0 = Layer::new(0, "Base", RgbColor::new(255, 0, 0)).unwrap();
    // Add same reference twice at different positions
    layer0.add_key(KeyDefinition::new(Position::new(0, 0), "MO(1)"));
    layer0.add_key(KeyDefinition::new(Position::new(0, 1), "MO(1)"));
    layout.add_layer(layer0).unwrap();

    let layer1 = Layer::new(1, "Lower", RgbColor::new(0, 255, 0)).unwrap();
    layout.add_layer(layer1).unwrap();

    let output = generate_layer_navigation(&layout);

    // Should only show one reference to layer 1 (deduplication)
    let mo1_count = output.matches("MO(1)").count();
    assert_eq!(
        mo1_count, 1,
        "Should deduplicate identical layer references"
    );
}

#[test]
fn test_build_outbound_references() {
    let mut layout = Layout::new("Test").unwrap();

    let mut layer0 = Layer::new(0, "Base", RgbColor::new(255, 0, 0)).unwrap();
    layer0.add_key(KeyDefinition::new(Position::new(0, 0), "MO(1)"));
    layer0.add_key(KeyDefinition::new(Position::new(0, 1), "LT(2, KC_SPC)"));
    layout.add_layer(layer0).unwrap();

    let layer1 = Layer::new(1, "Lower", RgbColor::new(0, 255, 0)).unwrap();
    layout.add_layer(layer1).unwrap();

    let layer2 = Layer::new(2, "Raise", RgbColor::new(0, 0, 255)).unwrap();
    layout.add_layer(layer2).unwrap();

    let refs = build_outbound_references(&layout);

    assert_eq!(refs.len(), 1); // Only layer 0 has outbound refs
    assert_eq!(refs.get(&0).unwrap().len(), 2); // Two refs from layer 0

    let layer0_refs = refs.get(&0).unwrap();
    assert!(layer0_refs
        .iter()
        .any(|(to, kc, _)| *to == 1 && kc == "MO(1)"));
    assert!(layer0_refs
        .iter()
        .any(|(to, kc, _)| *to == 2 && kc == "LT(2, KC_SPC)"));
}
