//! Color reference map builder and per-key color lookup.
//!
//! Scans a layer's keys for distinct RGB colors (skipping near-black),
//! assigns each a 1-based reference number, and lets other modules look
//! up the reference for a given key.

use crate::models::layout::Layout;
use std::collections::HashMap;

/// Returns a `HashMap` mapping RGB color to reference number (1-based).
pub(super) fn build_color_reference_map(layout: &Layout, layer_idx: usize) -> HashMap<String, usize> {
    let mut color_map = HashMap::new();
    let mut next_ref = 1;

    let layer = match layout.layers.get(layer_idx) {
        Some(l) => l,
        None => return color_map,
    };

    // Collect all unique colors on this layer
    for key in &layer.keys {
        let (color, _is_key_specific) = layout.resolve_display_color(layer_idx, key);

        // Skip black/off colors
        if color.r < 10 && color.g < 10 && color.b < 10 {
            continue;
        }

        let color_hex = format!("#{:02X}{:02X}{:02X}", color.r, color.g, color.b);

        if let std::collections::hash_map::Entry::Vacant(e) = color_map.entry(color_hex) {
            e.insert(next_ref);
            next_ref += 1;
        }
    }

    color_map
}

/// Gets the color reference number for a key.
pub(super) fn get_key_color_ref(
    layout: &Layout,
    layer_idx: usize,
    key: &crate::models::layer::KeyDefinition,
    color_map: &HashMap<String, usize>,
) -> Option<usize> {
    let (color, _is_key_specific) = layout.resolve_display_color(layer_idx, key);

    // Skip black/off colors
    if color.r < 10 && color.g < 10 && color.b < 10 {
        return None;
    }

    let color_hex = format!("#{:02X}{:02X}{:02X}", color.r, color.g, color.b);
    color_map.get(&color_hex).copied()
}
