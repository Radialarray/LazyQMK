//! Layer navigation diagram generator for layout exports.
//!
//! Generates a markdown section showing how layers are accessed via layer-switching
//! keycodes like LT (Layer Tap), MO (Momentary), TG (Toggle), etc.

use crate::models::Layout;
use crate::services::layer_refs::{parse_layer_keycode, LayerRefTarget};
use std::collections::{HashMap, HashSet};
use std::fmt::Write as _;

/// Generates a markdown layer navigation diagram section.
///
/// Analyzes the layout to find all layer-switching keycodes and builds a tree
/// structure showing which layers reference which other layers. The output uses
/// ASCII art to create a visual hierarchy.
///
/// # Arguments
///
/// * `layout` - The Layout to analyze for layer references
///
/// # Returns
///
/// A formatted markdown string with the layer navigation section
///
/// # Example Output
///
/// ```markdown
/// ## Layer Navigation
///
/// Shows how layers are accessed via LT (Layer Tap) and layer switching keys.
///
/// Layer 0 (Base)
///   ├─→ LT(1) on Tab → Layer 1 (Symbols)
///   ├─→ LT(2) on Esc → Layer 2 (Navigation)
///   └─→ LT(3) on W   → Layer 3 (Numbers)
///
/// Layer 1 (Symbols)
///   └─→ [No outbound references]
/// ```
///
/// # Example
///
/// ```no_run
/// use lazyqmk::export::layer_navigation::generate_layer_navigation;
/// use lazyqmk::models::Layout;
///
/// let layout = Layout::new("Test").unwrap();
/// let diagram = generate_layer_navigation(&layout);
/// println!("{}", diagram);
/// ```
pub fn generate_layer_navigation(layout: &Layout) -> String {
    let mut output = String::new();

    output.push_str("## Layer Navigation\n\n");
    output
        .push_str("Shows how layers are accessed via LT (Layer Tap) and layer switching keys.\n\n");

    // Build outbound references map (from_layer -> [(to_layer, keycode, position)])
    let outbound_refs = build_outbound_references(layout);

    // Generate output for each layer
    for (layer_idx, layer) in layout.layers.iter().enumerate() {
        let _ = writeln!(output, "Layer {} ({})", layer_idx, layer.name);

        if let Some(refs) = outbound_refs.get(&layer_idx) {
            if refs.is_empty() {
                output.push_str("  └─→ [No outbound references]\n");
            } else {
                // Sort references by target layer for consistent output
                let mut sorted_refs: Vec<_> = refs.iter().collect();
                sorted_refs.sort_by_key(|(to_layer, _, _)| *to_layer);

                let ref_count = sorted_refs.len();
                for (idx, (to_layer, keycode, position)) in sorted_refs.iter().enumerate() {
                    let is_last = idx == ref_count - 1;
                    let prefix = if is_last { "└─→" } else { "├─→" };

                    // Get the target layer name
                    let target_layer_name = layout
                        .layers
                        .get(*to_layer)
                        .map(|l| l.name.as_str())
                        .unwrap_or("Unknown");

                    // Format the position as a key name (e.g., "Tab" instead of (0,0))
                    let key_name = format_key_position(*position);

                    let _ = writeln!(
                        output,
                        "  {} {} on {} → Layer {} ({})",
                        prefix, keycode, key_name, to_layer, target_layer_name
                    );
                }
            }
        } else {
            output.push_str("  └─→ [No outbound references]\n");
        }

        output.push('\n');
    }

    output
}

/// Builds a map of outbound layer references.
///
/// Returns a map from source layer index to a list of (target_layer, keycode, position)
/// tuples representing all layer-switching keycodes on that layer.
fn build_outbound_references(
    layout: &Layout,
) -> HashMap<usize, Vec<(usize, String, crate::models::Position)>> {
    let mut outbound: HashMap<usize, Vec<(usize, String, crate::models::Position)>> =
        HashMap::new();

    // Build map from layer UUID to index for resolution
    let mut id_to_index: HashMap<String, usize> = HashMap::new();
    for (idx, layer) in layout.layers.iter().enumerate() {
        id_to_index.insert(layer.id.clone(), idx);
    }

    // Scan all layers for layer-switching keycodes
    for (from_layer_idx, layer) in layout.layers.iter().enumerate() {
        for key in &layer.keys {
            // Skip transparent/no-op keys
            if key.is_transparent() || key.is_no_op() {
                continue;
            }

            // Try to parse as layer keycode
            if let Some((target, _kind)) = parse_layer_keycode(&key.keycode) {
                let to_layer_idx = match target {
                    LayerRefTarget::Index(idx) => {
                        // Only track references to existing layers
                        if idx < layout.layers.len() {
                            Some(idx)
                        } else {
                            None
                        }
                    }
                    LayerRefTarget::Uuid(uuid) => {
                        // Strip leading '@' if present and resolve against layer IDs
                        let trimmed = uuid.strip_prefix('@').unwrap_or(uuid.as_str());
                        id_to_index.get(trimmed).copied()
                    }
                };

                if let Some(to_layer) = to_layer_idx {
                    outbound.entry(from_layer_idx).or_default().push((
                        to_layer,
                        key.keycode.clone(),
                        key.position,
                    ));
                }
            }
        }
    }

    // Remove duplicate references (same from->to, keep first occurrence)
    for refs in outbound.values_mut() {
        let mut seen = HashSet::new();
        refs.retain(|(to_layer, keycode, _pos)| {
            let key = (*to_layer, keycode.clone());
            seen.insert(key)
        });
    }

    outbound
}

/// Formats a key position as a human-readable string.
///
/// For now, returns the position as "(row,col)" format. Future enhancement could
/// map positions to actual key labels based on the keyboard layout.
fn format_key_position(position: crate::models::Position) -> String {
    format!("({},{})", position.row, position.col)
}

#[cfg(test)]
mod tests {
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
}
