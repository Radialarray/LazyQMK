//! Layer phase: parse layer tables and keycode cells.

use crate::models::{KeyDefinition, Layer, Position, RgbColor};
use anyhow::{Context, Result};

/// Parses a single layer section.
pub(super) fn parse_layer(
    lines: &[&str],
    start_line: usize,
    layout: &mut crate::models::Layout,
) -> Result<usize> {
    let mut line_num = start_line;
    let header_line = lines[line_num];

    // Parse layer header: ## Layer N: Name
    let captures = super::layer_regex()
        .captures(header_line)
        .ok_or_else(|| anyhow::anyhow!("Invalid layer header format: {header_line}"))?;

    let layer_number: u8 = captures[1]
        .parse()
        .context("Failed to parse layer number")?;
    let layer_name = captures[2].trim().to_string();

    line_num += 1;

    // Parse layer properties (Color and optional Category)
    let mut layer_color = None;
    let mut layer_category = None;
    let mut layer_colors_enabled = true; // Default to true
    let mut layer_id = None; // Optional layer ID for persistence

    while line_num < lines.len() {
        let line = lines[line_num].trim();

        if line.is_empty() {
            line_num += 1;
            continue;
        }

        // Parse color: **Color**: #RRGGBB
        if line.starts_with("**Color**:") {
            let color_str = line.strip_prefix("**Color**:").unwrap().trim();
            layer_color =
                Some(RgbColor::from_hex(color_str).context("Failed to parse layer color")?);
            line_num += 1;
            continue;
        }

        // Parse optional ID: **ID**: uuid
        if line.starts_with("**ID**:") {
            layer_id = Some(line.strip_prefix("**ID**:").unwrap().trim().to_string());
            line_num += 1;
            continue;
        }

        // Parse optional category: **Category**: category-id
        if line.starts_with("**Category**:") {
            let category_id = line
                .strip_prefix("**Category**:")
                .unwrap()
                .trim()
                .to_string();
            layer_category = Some(category_id);
            line_num += 1;
            continue;
        }

        // Parse optional layer colors enabled: **Layer Colors**: true/false
        if line.starts_with("**Layer Colors**:") {
            let value = line
                .strip_prefix("**Layer Colors**:")
                .unwrap()
                .trim()
                .to_lowercase();
            layer_colors_enabled = value == "true" || value == "yes" || value == "1";
            line_num += 1;
            continue;
        }

        // Table starts - break out of properties loop
        if line.starts_with('|') {
            break;
        }

        line_num += 1;
    }

    let color = layer_color.ok_or_else(|| {
        anyhow::anyhow!("Layer {layer_number} missing required **Color** property")
    })?;

    // Create layer
    let mut layer = Layer::new(layer_number, layer_name, color)?;
    // Use persisted ID if available, otherwise keep the generated one
    if let Some(id) = layer_id {
        layer.id = id;
    }
    layer.category_id = layer_category;
    layer.layer_colors_enabled = layer_colors_enabled;

    // Parse table
    line_num = parse_layer_table(lines, line_num, &mut layer)?;

    // Add layer to layout
    layout.add_layer(layer)?;

    Ok(line_num)
}

/// Parses a layer's Markdown key table and adds keys to `layer`.
///
/// Each table row maps to a **visual** row, and each cell maps to a **visual** column.
/// The resulting `KeyDefinition` objects all carry visual-grid [`Position`] values.
pub(super) fn parse_layer_table(
    lines: &[&str],
    start_line: usize,
    layer: &mut Layer,
) -> Result<usize> {
    let mut line_num = start_line;
    let mut row = 0;

    // Skip table header row
    if line_num < lines.len() && lines[line_num].starts_with('|') {
        line_num += 1;
    }

    // Skip separator row (|---|---|)
    if line_num < lines.len() && lines[line_num].contains("---") {
        line_num += 1;
    }

    // Parse data rows
    while line_num < lines.len() {
        let line = lines[line_num].trim();

        // Stop at empty line or next section
        if line.is_empty() || line.starts_with("##") || line.starts_with("---") {
            break;
        }

        // Parse table row
        if line.starts_with('|') {
            parse_table_row(line, row, layer).with_context(|| {
                format!("Error parsing table row {} at line {}", row, line_num + 1)
            })?;
            row += 1;
        }

        line_num += 1;
    }

    Ok(line_num)
}

/// Parses a single Markdown table row into key definitions at **visual** row `row`.
///
/// `row` is a 0-based **visual** row index. Each non-empty cell becomes a
/// `KeyDefinition` with `Position::new(row, col_index)` — visual coordinates.
pub(super) fn parse_table_row(line: &str, row: u8, layer: &mut Layer) -> Result<()> {
    // Split by pipes and trim, keeping empty cells to preserve column indices
    // This is critical for split keyboards where gaps between halves are empty cells
    let cells: Vec<&str> = line.split('|').map(str::trim).collect();

    // Skip leading empty element from split (line starts with '|')
    // and trailing empty element (line ends with '|')
    let cells = if cells.len() >= 2 {
        &cells[1..cells.len() - 1]
    } else {
        &cells[..]
    };

    for (col, cell) in cells.iter().enumerate() {
        // Skip empty cells (gaps in split keyboards) but preserve column index
        if cell.is_empty() {
            continue;
        }

        // Parse keycode syntax
        let key = parse_keycode_syntax(cell, row, col as u8)
            .with_context(|| format!("Error parsing cell at row {row}, col {col}: {cell}"))?;

        layer.add_key(key);
    }

    Ok(())
}

/// Parses keycode syntax (with optional color / category) into a [`KeyDefinition`].
///
/// `row` and `col` are **visual** grid coordinates; they are stored verbatim in
/// the returned `KeyDefinition.position` — no coordinate conversion happens here.
///
/// Formats:
/// - `KC_X` - basic keycode
/// - `KC_X{#RRGGBB}` - with color override
/// - `KC_X@category-id` - with category
/// - `KC_X{#RRGGBB}@category-id` - with both
pub(super) fn parse_keycode_syntax(cell: &str, row: u8, col: u8) -> Result<KeyDefinition> {
    // Updated regex to support:
    // - Basic keycodes: KC_A, KC_LEFT, etc.
    // - Parameterized keycodes: LT(0, KC_A), MT(MOD_LCTL, KC_A)
    // - Layer UUIDs inside params: LT(@f85996a8-8dbd-403d-a804-fac1f2bc751d, KC_R)
    // - With optional color suffix: {#RRGGBB}
    // - With optional category suffix: @category-id
    // Pattern breakdown:
    //   [A-Z_][A-Z_0-9]*  - Keycode prefix (must start with letter or underscore)
    //   (?:\([^)]*\))?    - Optional parentheses with anything inside (for params)
    //   (?:\{...\})?      - Optional color override
    //   (?:@...)?         - Optional category suffix (@ only allowed here, not in keycode)
    let captures = super::keycode_regex()
        .captures(cell)
        .ok_or_else(|| anyhow::anyhow!("Invalid keycode syntax: {cell}"))?;

    let keycode = captures[1].to_string();
    let color_override = captures
        .get(2)
        .map(|m| RgbColor::from_hex(m.as_str()))
        .transpose()?;
    let category_id = captures.get(3).map(|m| m.as_str().to_string());

    let position = Position::new(row, col);
    let mut key = KeyDefinition::new(position, keycode);

    if let Some(color) = color_override {
        key = key.with_color(color);
    }

    if let Some(cat_id) = category_id {
        key = key.with_category(&cat_id);
    }

    Ok(key)
}
