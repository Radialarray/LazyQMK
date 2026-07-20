//! Keyboard visual renderer for layout export.
//!
//! Generates ASCII/Unicode keyboard diagrams using box-drawing characters.
//! Supports split keyboards, tap-hold keys, and color reference indicators.
//!
//! Module layout:
//! - [`rendering`] — grid building, split detection, box-drawing rendering.
//! - [`formatting`] — keycode → display text (`KC_A` → `A`, `LCTL_T(KC_X)` → multi-line).
//! - [`color_ref`] — color reference map builder + key color lookup.

use crate::models::{keyboard_geometry::KeyboardGeometry, layout::Layout};
use anyhow::{Context, Result};

mod color_ref;
mod formatting;
mod rendering;

use rendering::{build_key_grid, render_grid};

/// A single key positioned in the rendering grid.
pub struct GridKey {
    /// Display text for the key (formatted keycode)
    pub label: String,
    /// Optional color reference number (1-based)
    pub color_ref: Option<usize>,
    /// Row in the grid (0-based)
    pub row: usize,
    /// Column in the grid (0-based)
    pub col: usize,
    /// Width in grid units (1 = standard key)
    pub width: usize,
    /// Height in grid units (1 = standard key)
    pub height: usize,
}

/// Collection of keys arranged in a 2D grid.
pub struct KeyGrid {
    /// All keys in the grid
    pub keys: Vec<GridKey>,
    /// Maximum row index used
    pub max_row: usize,
    /// Maximum column index used
    pub max_col: usize,
    /// Gap between split halves (column index where right half starts)
    pub split_gap: Option<usize>,
}

/// Renders a single layer as an ASCII/Unicode keyboard diagram.
pub fn render_layer_diagram(
    layout: &Layout,
    layer_idx: usize,
    geometry: &KeyboardGeometry,
) -> Result<String> {
    let layer = layout
        .layers
        .get(layer_idx)
        .context("Layer index out of bounds")?;

    let mut output = String::new();

    // Add layer header
    use std::fmt::Write;
    writeln!(output, "Layer {}: {}", layer_idx, layer.name).unwrap();

    // Build key grid with positioning
    let key_grid = build_key_grid(layout, layer_idx, geometry)?;

    // Render the grid to ASCII/Unicode
    let diagram = render_grid(&key_grid);
    output.push_str(&diagram);

    Ok(output)
}

#[cfg(test)]
mod tests;
