//! Rendering logic: grid building, split detection, box-drawing rendering.

use crate::models::{keyboard_geometry::KeyboardGeometry, layout::Layout};
use anyhow::{Context, Result};

use super::color_ref::{build_color_reference_map, get_key_color_ref};
use super::formatting::format_keycode;
use super::{GridKey, KeyGrid};

/// Builds a grid of keys with positioning from geometry.
pub(super) fn build_key_grid(
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
pub(super) fn detect_split_gap(keys: &[GridKey], max_col: usize) -> Option<usize> {
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
pub(super) fn render_grid(grid: &KeyGrid) -> String {
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
pub(super) fn render_key_box(
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
pub(super) fn write_centered_text(
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