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
/// Returns a map from source layer index to a list of (`target_layer`, keycode, position)
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

/// Formats a **visual** key position as a human-readable string.
///
/// `position` is a visual-grid [`Position`] (row, col as displayed in the UI).
/// Returns `"(row,col)"` format. A future enhancement could resolve this to
/// an actual key label using the keyboard layout.
fn format_key_position(position: crate::models::Position) -> String {
    format!("({},{})", position.row, position.col)
}

#[cfg(test)]
mod tests;

