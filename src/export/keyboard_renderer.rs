//! Keyboard visual renderer for layout export.
//!
//! Generates ASCII/Unicode keyboard diagrams using box-drawing characters.
//! Supports split keyboards, tap-hold keys, and color reference indicators.

use crate::models::{keyboard_geometry::KeyboardGeometry, layout::Layout};
use anyhow::{Context, Result};
use std::collections::HashMap;

/// Renders a single layer as an ASCII/Unicode keyboard diagram.
///
/// # Arguments
///
/// * `layout` - The complete layout containing layer data
/// * `layer_idx` - Index of the layer to render
/// * `geometry` - Keyboard geometry for physical positioning
///
/// # Returns
///
/// A String containing the ASCII/Unicode keyboard diagram suitable for markdown code blocks.
///
/// # Example
///
/// ```text
/// Layer 0: Base
/// ┌────────┬────────┬────────┐                    ┌────────┬────────┬────────┐
/// │  Q [1] │  W [2] │  E [3] │                    │  U [7] │  I [8] │  O [9] │
/// │        │        │        │                    │        │        │        │
/// └────────┴────────┴────────┘                    └────────┴────────┴────────┘
/// ```
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

/// Represents a single key in the rendered grid.
#[derive(Debug, Clone)]
struct GridKey {
    /// Display text for the key (formatted keycode)
    label: String,
    /// Optional color reference number (1-based)
    color_ref: Option<usize>,
    /// Grid position (row, col in visual space)
    row: usize,
    col: usize,
    /// Key width in grid units (1 = normal, 2 = 2u, etc.)
    width: usize,
    /// Key height in grid units
    height: usize,
}

/// Represents the complete keyboard grid with positioning.
#[derive(Debug)]
struct KeyGrid {
    keys: Vec<GridKey>,
    max_row: usize,
    max_col: usize,
    /// Gap between split halves (column index where right half starts)
    split_gap: Option<usize>,
}

/// Builds a grid of keys with positioning from geometry.
fn build_key_grid(
    layout: &Layout,
    layer_idx: usize,
    geometry: &KeyboardGeometry,
) -> Result<KeyGrid> {
    let layer = layout
        .layers
        .get(layer_idx)
        .context("Layer index out of bounds")?;

    let mut keys = Vec::new();
    let mut max_row = 0;
    let mut max_col = 0;

    // Build color reference map
    let color_map = build_color_reference_map(layout, layer_idx);

    // Build a lookup from visual position (row, col) to key definition
    let mut visual_to_key = std::collections::HashMap::new();
    for key_def in &layer.keys {
        let key_pos = (key_def.position.row, key_def.position.col);
        visual_to_key.insert(key_pos, key_def);
    }

    // Process each key in geometry order
    // For each key in geometry, find the corresponding key definition by matching positions
    for key_geom in &geometry.keys {
        // Try to find key definition by checking all visual positions
        // The key_def.position is the table position (row, col) in the markdown
        // We need to find which key_def corresponds to this geometry key

        // Calculate the visual position from geometry
        let visual_row = key_geom.visual_y.round() as u8;
        let visual_col = key_geom.visual_x.round() as u8;

        // Look up key by visual position
        let key_def = match visual_to_key.get(&(visual_row, visual_col)) {
            Some(kd) => kd,
            None => {
                // Key exists in geometry but not in layout - skip it
                // This can happen with optional keys or when layout doesn't define all positions
                continue;
            }
        };

        // Convert visual position to grid coordinates
        // Use a simple scaling: divide by standard key width (assuming ~1u spacing)
        let row = (key_geom.visual_y / 1.25).round() as usize;
        let col = (key_geom.visual_x / 1.25).round() as usize;

        // Format key label (handle tap-hold keys)
        let label = format_keycode(&key_def.keycode);

        // Get color reference if key has color
        let color_ref = get_key_color_ref(layout, layer_idx, key_def, &color_map);

        // Determine key size (round to nearest integer)
        let width = key_geom.width.round().max(1.0) as usize;
        let height = key_geom.height.round().max(1.0) as usize;

        keys.push(GridKey {
            label,
            color_ref,
            row,
            col,
            width,
            height,
        });

        max_row = max_row.max(row);
        max_col = max_col.max(col + width - 1);
    }

    // Detect split keyboard gap
    let split_gap = detect_split_gap(&keys, max_col);

    Ok(KeyGrid {
        keys,
        max_row,
        max_col,
        split_gap,
    })
}

/// Detects the gap between split keyboard halves.
///
/// Looks for a large horizontal gap in key positions.
fn detect_split_gap(keys: &[GridKey], max_col: usize) -> Option<usize> {
    if max_col < 5 {
        return None; // Too small to be split (need at least 6 columns total)
    }

    // Count keys per column
    let mut col_counts = vec![0; max_col + 1];
    for key in keys {
        for (i, count) in col_counts
            .iter_mut()
            .enumerate()
            .skip(key.col)
            .take(key.width)
        {
            if i <= max_col {
                *count += 1;
            }
        }
    }

    // Find largest gap (consecutive empty columns)
    let mut max_gap_start = None;
    let mut max_gap_size = 0;
    let mut current_gap_start = None;
    let mut current_gap_size = 0;

    for (col, &count) in col_counts.iter().enumerate() {
        if count == 0 {
            if current_gap_start.is_none() {
                current_gap_start = Some(col);
                current_gap_size = 1;
            } else {
                current_gap_size += 1;
            }
        } else if let Some(start) = current_gap_start {
            if current_gap_size > max_gap_size {
                max_gap_start = Some(start);
                max_gap_size = current_gap_size;
            }
            current_gap_start = None;
            current_gap_size = 0;
        }
    }

    // Check final gap
    if let Some(start) = current_gap_start {
        if current_gap_size > max_gap_size {
            max_gap_start = Some(start);
            max_gap_size = current_gap_size;
        }
    }

    // Only consider it a split if gap is significant (3+ columns)
    if max_gap_size >= 3 {
        max_gap_start
    } else {
        None
    }
}

/// Renders the key grid to ASCII/Unicode string.
fn render_grid(grid: &KeyGrid) -> String {
    // Calculate dimensions for each key cell
    // Standard key: 12 chars wide × 3 lines tall (including borders)
    const KEY_WIDTH: usize = 10; // Content width (12 - 2 for borders)
    const KEY_HEIGHT: usize = 3; // Total height including borders

    // Calculate total grid dimensions
    let grid_height = (grid.max_row + 1) * KEY_HEIGHT;
    let grid_width = if grid.split_gap.is_some() {
        // Add extra spacing for split gap
        (grid.max_col + 1) * (KEY_WIDTH + 2) + 20 // 20 chars for split gap
    } else {
        (grid.max_col + 1) * (KEY_WIDTH + 2)
    };

    // Create 2D character buffer
    let mut buffer = vec![vec![' '; grid_width]; grid_height];

    // Render each key
    for key in &grid.keys {
        let row_start = key.row * KEY_HEIGHT;
        let col_start = if let Some(gap) = grid.split_gap {
            if key.col >= gap {
                // Right half: add extra spacing
                key.col * (KEY_WIDTH + 2) + 20
            } else {
                key.col * (KEY_WIDTH + 2)
            }
        } else {
            key.col * (KEY_WIDTH + 2)
        };

        let key_width = KEY_WIDTH * key.width + (key.width - 1) * 2; // Account for borders between merged keys
        let key_height = KEY_HEIGHT * key.height;

        render_key_box(
            &mut buffer,
            row_start,
            col_start,
            key_width,
            key_height,
            &key.label,
            key.color_ref,
        );
    }

    // Convert buffer to string
    let mut output = String::new();
    for row in &buffer {
        let line: String = row.iter().collect();
        output.push_str(&line.trim_end());
        output.push('\n');
    }

    output
}

/// Renders a single key box with Unicode box-drawing characters.
fn render_key_box(
    buffer: &mut [Vec<char>],
    row: usize,
    col: usize,
    width: usize,
    height: usize,
    label: &str,
    color_ref: Option<usize>,
) {
    let max_row = buffer.len();
    let max_col = buffer[0].len();

    // Check bounds
    if row >= max_row || col >= max_col {
        return; // Skip if out of bounds
    }

    let actual_width = width.min(max_col - col);
    let actual_height = height.min(max_row - row);

    if actual_width < 4 || actual_height < 3 {
        return; // Too small to render
    }

    // Draw top border: ┌───────┐
    if row < max_row {
        if col < max_col {
            buffer[row][col] = '┌';
        }
        for c in 1..actual_width.saturating_sub(1) {
            if col + c < max_col {
                buffer[row][col + c] = '─';
            }
        }
        if col + actual_width - 1 < max_col {
            buffer[row][col + actual_width - 1] = '┐';
        }
    }

    // Draw bottom border: └───────┘
    let bottom_row = row + actual_height - 1;
    if bottom_row < max_row {
        if col < max_col {
            buffer[bottom_row][col] = '└';
        }
        for c in 1..actual_width.saturating_sub(1) {
            if col + c < max_col {
                buffer[bottom_row][col + c] = '─';
            }
        }
        if col + actual_width - 1 < max_col {
            buffer[bottom_row][col + actual_width - 1] = '┘';
        }
    }

    // Draw left and right borders: │
    for r in 1..actual_height.saturating_sub(1) {
        if row + r < max_row {
            if col < max_col {
                buffer[row + r][col] = '│';
            }
            if col + actual_width - 1 < max_col {
                buffer[row + r][col + actual_width - 1] = '│';
            }
        }
    }

    // Render key content
    let content_width = actual_width.saturating_sub(2); // Exclude borders
    let content_start_col = col + 1;

    // Check if this is a tap-hold key (contains " / ")
    if label.contains(" / ") {
        let parts: Vec<&str> = label.split(" / ").collect();
        if parts.len() == 2 {
            // Line 1: Hold action
            let hold = parts[0];
            if row + 1 < max_row {
                write_centered_text(
                    buffer,
                    row + 1,
                    content_start_col,
                    content_width,
                    hold,
                    max_col,
                );
            }

            // Line 2: Tap action
            let tap = parts[1];
            if row + 2 < max_row {
                write_centered_text(
                    buffer,
                    row + 2,
                    content_start_col,
                    content_width,
                    tap,
                    max_col,
                );
            }

            // Line 3: Color reference (if present and space available)
            if let Some(ref_num) = color_ref {
                if actual_height > 3 && row + 3 < max_row {
                    let color_text = format!("[{}]", ref_num);
                    write_centered_text(
                        buffer,
                        row + 3,
                        content_start_col,
                        content_width,
                        &color_text,
                        max_col,
                    );
                }
            }
        }
    } else {
        // Simple keycode: center in key
        let middle_row = row + (actual_height / 2);
        if middle_row < max_row {
            write_centered_text(
                buffer,
                middle_row,
                content_start_col,
                content_width,
                label,
                max_col,
            );
        }

        // Color reference below label
        if let Some(ref_num) = color_ref {
            let color_row = middle_row + 1;
            if color_row < max_row {
                let color_text = format!("[{}]", ref_num);
                write_centered_text(
                    buffer,
                    color_row,
                    content_start_col,
                    content_width,
                    &color_text,
                    max_col,
                );
            }
        }
    }
}

/// Writes text centered in a row.
fn write_centered_text(
    buffer: &mut [Vec<char>],
    row: usize,
    start_col: usize,
    width: usize,
    text: &str,
    max_col: usize,
) {
    if row >= buffer.len() {
        return;
    }

    let text_len = text.chars().count();
    if text_len > width {
        // Truncate if too long
        let truncated: String = text.chars().take(width).collect();
        for (i, ch) in truncated.chars().enumerate() {
            if start_col + i < max_col {
                buffer[row][start_col + i] = ch;
            }
        }
    } else {
        // Center the text
        let padding = (width - text_len) / 2;
        let text_start = start_col + padding;
        for (i, ch) in text.chars().enumerate() {
            if text_start + i < max_col {
                buffer[row][text_start + i] = ch;
            }
        }
    }
}

/// Formats a keycode for display.
///
/// Handles tap-hold keys with split display (e.g., "LT(1, KC_A)" -> "L1 / A")
fn format_keycode(keycode: &str) -> String {
    // Handle Layer Tap: LT(layer, keycode)
    if let Some(inner) = keycode.strip_prefix("LT(") {
        if let Some(args) = inner.strip_suffix(')') {
            let parts: Vec<&str> = args.split(',').map(str::trim).collect();
            if parts.len() == 2 {
                let layer = parts[0].trim_start_matches('@'); // Remove @ prefix
                let tap = strip_kc_prefix(parts[1]);
                return format!("L{} / {}", layer, tap);
            }
        }
    }

    // Handle Mod Tap: MT(mod, keycode)
    if let Some(inner) = keycode.strip_prefix("MT(") {
        if let Some(args) = inner.strip_suffix(')') {
            let parts: Vec<&str> = args.split(',').map(str::trim).collect();
            if parts.len() == 2 {
                let mod_display = format_modifier(parts[0]);
                let tap = strip_kc_prefix(parts[1]);
                return format!("{} / {}", mod_display, tap);
            }
        }
    }

    // Handle named mod-tap: LCTL_T(keycode), LSFT_T(keycode), etc.
    for (prefix, mod_name) in &[
        ("LCTL_T", "CTL"),
        ("LSFT_T", "SFT"),
        ("LALT_T", "ALT"),
        ("LGUI_T", "GUI"),
        ("RCTL_T", "CTL"),
        ("RSFT_T", "SFT"),
        ("RALT_T", "ALT"),
        ("RGUI_T", "GUI"),
    ] {
        if let Some(inner) = keycode.strip_prefix(prefix) {
            if let Some(tap) = inner.strip_prefix('(').and_then(|s| s.strip_suffix(')')) {
                return format!("{} / {}", mod_name, strip_kc_prefix(tap));
            }
        }
    }

    // Handle Layer Mod: LM(layer, mod)
    if let Some(inner) = keycode.strip_prefix("LM(") {
        if let Some(args) = inner.strip_suffix(')') {
            let parts: Vec<&str> = args.split(',').map(str::trim).collect();
            if parts.len() == 2 {
                let layer = parts[0].trim_start_matches('@');
                let mod_display = format_modifier(parts[1]);
                return format!("L{}+{}", layer, mod_display);
            }
        }
    }

    // Handle MO (momentary layer)
    if let Some(inner) = keycode.strip_prefix("MO(") {
        if let Some(layer) = inner.strip_suffix(')') {
            let layer = layer.trim_start_matches('@');
            return format!("▼L{}", layer);
        }
    }

    // Handle TG (toggle layer)
    if let Some(inner) = keycode.strip_prefix("TG(") {
        if let Some(layer) = inner.strip_suffix(')') {
            let layer = layer.trim_start_matches('@');
            return format!("TG{}", layer);
        }
    }

    // Handle Tap Dance: TD(name)
    if let Some(inner) = keycode.strip_prefix("TD(") {
        if let Some(name) = inner.strip_suffix(')') {
            return format!("TD:{}", name);
        }
    }

    // Simple keycode: strip KC_ prefix
    strip_kc_prefix(keycode)
}

/// Strips the "KC_" prefix from a keycode.
fn strip_kc_prefix(keycode: &str) -> String {
    keycode.strip_prefix("KC_").unwrap_or(keycode).to_string()
}

/// Formats a modifier string for compact display.
fn format_modifier(mod_str: &str) -> String {
    let mut result = String::new();

    if mod_str.contains("LCTL") || mod_str.contains("RCTL") {
        result.push('C');
    }
    if mod_str.contains("LSFT") || mod_str.contains("RSFT") {
        result.push('S');
    }
    if mod_str.contains("LALT") || mod_str.contains("RALT") {
        result.push('A');
    }
    if mod_str.contains("LGUI") || mod_str.contains("RGUI") {
        result.push('G');
    }

    if result.is_empty() {
        // Fallback: take first 3 chars
        mod_str.chars().take(3).collect()
    } else {
        result
    }
}

/// Builds a color reference map for a layer.
///
/// Returns a HashMap mapping RGB color to reference number (1-based).
fn build_color_reference_map(layout: &Layout, layer_idx: usize) -> HashMap<String, usize> {
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
fn get_key_color_ref(
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{
        keyboard_geometry::KeyGeometry,
        layer::{KeyDefinition, Layer, Position},
        RgbColor,
    };

    fn create_test_geometry() -> KeyboardGeometry {
        let mut geom = KeyboardGeometry::new("test", "LAYOUT", 4, 12);

        // Create a simple 3x3 grid layout
        // Use exact integer positions so visual coordinates match
        for row in 0..3 {
            for col in 0..3 {
                let key = KeyGeometry::new(
                    (row, col),
                    (row * 3 + col) as u8,
                    col as f32, // Use exact integers instead of 1.25 spacing
                    row as f32,
                );
                geom.add_key(key);
            }
        }

        geom
    }

    fn create_test_layout() -> Layout {
        let mut layout = Layout::new("Test Layout").unwrap();

        let mut layer = Layer::new(0, "Base", RgbColor::new(255, 255, 255)).unwrap();

        // Add 3x3 grid of keys
        for row in 0..3 {
            for col in 0..3 {
                let keycode = format!("KC_{}", (b'A' + (row * 3 + col)) as char);
                layer.add_key(KeyDefinition::new(Position::new(row, col), &keycode));
            }
        }

        layout.add_layer(layer).unwrap();
        layout
    }

    #[test]
    fn test_format_keycode_simple() {
        assert_eq!(format_keycode("KC_A"), "A");
        assert_eq!(format_keycode("KC_SPACE"), "SPACE");
        assert_eq!(format_keycode("KC_ENTER"), "ENTER");
    }

    #[test]
    fn test_format_keycode_layer_tap() {
        assert_eq!(format_keycode("LT(1, KC_A)"), "L1 / A");
        assert_eq!(format_keycode("LT(@abc-123, KC_SPC)"), "Labc-123 / SPC");
    }

    #[test]
    fn test_format_keycode_mod_tap() {
        assert_eq!(format_keycode("LCTL_T(KC_A)"), "CTL / A");
        assert_eq!(format_keycode("LSFT_T(KC_SPC)"), "SFT / SPC");
    }

    #[test]
    fn test_format_keycode_layer_mod() {
        assert_eq!(format_keycode("LM(1, MOD_LCTL)"), "L1+C");
    }

    #[test]
    fn test_format_keycode_momentary_layer() {
        assert_eq!(format_keycode("MO(1)"), "▼L1");
        assert_eq!(format_keycode("MO(@abc-123)"), "▼Labc-123");
    }

    #[test]
    fn test_format_keycode_tap_dance() {
        assert_eq!(format_keycode("TD(quote_dance)"), "TD:quote_dance");
    }

    #[test]
    fn test_format_modifier() {
        assert_eq!(format_modifier("MOD_LCTL"), "C");
        assert_eq!(format_modifier("MOD_LSFT"), "S");
        assert_eq!(format_modifier("MOD_LCTL | MOD_LSFT"), "CS");
        assert_eq!(
            format_modifier("MOD_LCTL | MOD_LSFT | MOD_LALT | MOD_LGUI"),
            "CSAG"
        );
    }

    #[test]
    fn test_strip_kc_prefix() {
        assert_eq!(strip_kc_prefix("KC_A"), "A");
        assert_eq!(strip_kc_prefix("KC_SPACE"), "SPACE");
        assert_eq!(strip_kc_prefix("MO(1)"), "MO(1)");
    }

    #[test]
    fn test_build_key_grid() {
        let layout = create_test_layout();
        let geometry = create_test_geometry();

        let grid = build_key_grid(&layout, 0, &geometry).unwrap();

        assert_eq!(grid.keys.len(), 9); // 3x3 grid
        assert!(grid.max_row >= 2);
        assert!(grid.max_col >= 2);
    }

    #[test]
    fn test_render_layer_diagram() {
        let layout = create_test_layout();
        let geometry = create_test_geometry();

        let diagram = render_layer_diagram(&layout, 0, &geometry).unwrap();

        // Basic checks
        assert!(diagram.contains("Layer 0: Base"));
        assert!(diagram.contains('┌')); // Contains box drawing chars
        assert!(diagram.contains('│'));
        assert!(diagram.contains('─'));
    }

    #[test]
    fn test_detect_split_gap_no_split() {
        let keys = vec![
            GridKey {
                label: "A".to_string(),
                color_ref: None,
                row: 0,
                col: 0,
                width: 1,
                height: 1,
            },
            GridKey {
                label: "B".to_string(),
                color_ref: None,
                row: 0,
                col: 1,
                width: 1,
                height: 1,
            },
            GridKey {
                label: "C".to_string(),
                color_ref: None,
                row: 0,
                col: 2,
                width: 1,
                height: 1,
            },
        ];

        assert_eq!(detect_split_gap(&keys, 2), None);
    }

    #[test]
    fn test_detect_split_gap_with_split() {
        let keys = vec![
            GridKey {
                label: "A".to_string(),
                color_ref: None,
                row: 0,
                col: 0,
                width: 1,
                height: 1,
            },
            GridKey {
                label: "B".to_string(),
                color_ref: None,
                row: 0,
                col: 1,
                width: 1,
                height: 1,
            },
            // Gap at columns 2, 3, 4
            GridKey {
                label: "C".to_string(),
                color_ref: None,
                row: 0,
                col: 5,
                width: 1,
                height: 1,
            },
        ];

        assert_eq!(detect_split_gap(&keys, 5), Some(2));
    }
}
