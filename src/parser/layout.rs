//! Markdown layout file parsing and generation.
//!
//! This module handles parsing keyboard layouts from human-readable Markdown files
//! and generating them back for saving. The format uses YAML frontmatter for metadata
//! and Markdown tables for key assignments.

use crate::models::{Category, KeyDefinition, Layer, Layout, LayoutMetadata, Position, RgbColor};
use anyhow::{Context, Result};
use regex::Regex;
use std::path::Path;

/// Parsing state machine states.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ParseState {
    /// Reading YAML frontmatter (between --- markers)
    InFrontmatter,
    /// Reading main content (after frontmatter)
    InContent,
    /// Reading layer header line (## Layer N: Name)
    InLayerHeader,
    /// Reading layer properties (Color, Category)
    InLayerProperties,
    /// Reading layer table
    InLayerTable,
    /// Reading categories section
    InCategories,
}

/// Parses a Markdown layout file into a Layout structure.
///
/// # File Format
///
/// ```markdown
/// ---
/// name: "Layout Name"
/// description: "Description"
/// author: "Author"
/// created: "2024-01-15T10:30:00Z"
/// modified: "2024-01-20T15:45:00Z"
/// tags: ["tag1", "tag2"]
/// is_template: false
/// version: "1.0"
/// ---
///
/// # Layout Title
///
/// ## Layer 0: Base
/// **Color**: #808080
/// **Category**: optional-category-id
///
/// | KC_TAB | KC_Q | ... |
/// |--------|------|-----|
/// | KC_A   | KC_S | ... |
///
/// ## Categories
///
/// - category-id: Category Name (#RRGGBB)
/// ```
///
/// # Errors
///
/// Returns errors for:
/// - Invalid YAML frontmatter
/// - Malformed layer headers
/// - Invalid table structure
/// - Invalid keycodes or color syntax
pub fn parse_markdown_layout(path: &Path) -> Result<Layout> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read layout file: {}", path.display()))?;

    parse_markdown_layout_str(&content)
        .with_context(|| format!("Failed to parse layout file: {}", path.display()))
}

/// Parses a Markdown layout from a string.
pub fn parse_markdown_layout_str(content: &str) -> Result<Layout> {
    let lines: Vec<&str> = content.lines().collect();

    // Parse frontmatter
    let (metadata, content_start) = parse_frontmatter(&lines)?;

    // Create layout
    let mut layout = Layout {
        metadata,
        layers: Vec::new(),
        categories: Vec::new(),
    };

    // Parse content (layers and categories)
    parse_content(&lines[content_start..], &mut layout)?;

    // Validate the parsed layout
    layout.validate()?;

    Ok(layout)
}

/// Parses YAML frontmatter from the beginning of the file.
///
/// Returns the parsed metadata and the line index where content starts.
fn parse_frontmatter(lines: &[&str]) -> Result<(LayoutMetadata, usize)> {
    // Find frontmatter boundaries
    let mut start_idx = None;
    let mut end_idx = None;

    for (idx, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed == "---" {
            if start_idx.is_none() {
                start_idx = Some(idx);
            } else if end_idx.is_none() {
                end_idx = Some(idx);
                break;
            }
        }
    }

    let start =
        start_idx.ok_or_else(|| anyhow::anyhow!("Missing frontmatter start marker (---)"))?;
    let end = end_idx.ok_or_else(|| anyhow::anyhow!("Missing frontmatter end marker (---)"))?;

    // Extract YAML content (between the --- markers)
    let yaml_content = lines[start + 1..end].join("\n");

    // Parse YAML
    let metadata: LayoutMetadata =
        serde_yaml::from_str(&yaml_content).context("Failed to parse YAML frontmatter")?;

    // Validate metadata
    validate_metadata(&metadata)?;

    Ok((metadata, end + 1))
}

/// Validates metadata after parsing.
fn validate_metadata(metadata: &LayoutMetadata) -> Result<()> {
    if metadata.name.is_empty() {
        anyhow::bail!("Layout name cannot be empty");
    }

    if metadata.name.len() > 100 {
        anyhow::bail!(
            "Layout name exceeds maximum length of 100 characters (got {})",
            metadata.name.len()
        );
    }

    if metadata.modified < metadata.created {
        anyhow::bail!("Modified timestamp cannot be before created timestamp");
    }

    if metadata.version != "1.0" {
        anyhow::bail!(
            "Unsupported schema version '{}'. Only version '1.0' is supported.",
            metadata.version
        );
    }

    // Validate tags
    let tag_regex = Regex::new(r"^[a-z0-9-]+$").unwrap();
    for tag in &metadata.tags {
        if !tag_regex.is_match(tag) {
            anyhow::bail!(
                "Invalid tag '{}'. Tags must be lowercase with hyphens and alphanumeric characters only",
                tag
            );
        }
    }

    Ok(())
}

/// Parses the content section (layers and categories).
fn parse_content(lines: &[&str], layout: &mut Layout) -> Result<()> {
    let mut line_num = 0;

    while line_num < lines.len() {
        let line = lines[line_num].trim();

        // Skip empty lines and main title
        if line.is_empty() || line.starts_with("# ") {
            line_num += 1;
            continue;
        }

        // Check for layer header (## Layer N: Name)
        if line.starts_with("## Layer ") {
            line_num = parse_layer(lines, line_num, layout)
                .with_context(|| format!("Error parsing layer at line {}", line_num + 1))?;
            continue;
        }

        // Check for categories section (## Categories)
        if line == "## Categories" {
            line_num = parse_categories(lines, line_num, layout)
                .with_context(|| format!("Error parsing categories at line {}", line_num + 1))?;
            continue;
        }

        line_num += 1;
    }

    Ok(())
}

/// Parses a single layer section.
fn parse_layer(lines: &[&str], start_line: usize, layout: &mut Layout) -> Result<usize> {
    let mut line_num = start_line;
    let header_line = lines[line_num];

    // Parse layer header: ## Layer N: Name
    let layer_regex = Regex::new(r"^##\s+Layer\s+(\d+):\s+(.+)$").unwrap();
    let captures = layer_regex
        .captures(header_line)
        .ok_or_else(|| anyhow::anyhow!("Invalid layer header format: {}", header_line))?;

    let layer_number: u8 = captures[1]
        .parse()
        .context("Failed to parse layer number")?;
    let layer_name = captures[2].trim().to_string();

    line_num += 1;

    // Parse layer properties (Color and optional Category)
    let mut layer_color = None;
    let mut layer_category = None;

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

        // Table starts - break out of properties loop
        if line.starts_with("|") {
            break;
        }

        line_num += 1;
    }

    let color = layer_color.ok_or_else(|| {
        anyhow::anyhow!("Layer {} missing required **Color** property", layer_number)
    })?;

    // Create layer
    let mut layer = Layer::new(layer_number, layer_name, color)?;
    layer.category_id = layer_category;

    // Parse table
    line_num = parse_layer_table(lines, line_num, &mut layer)?;

    // Add layer to layout
    layout.add_layer(layer)?;

    Ok(line_num)
}

/// Parses a layer's key table.
fn parse_layer_table(lines: &[&str], start_line: usize, layer: &mut Layer) -> Result<usize> {
    let mut line_num = start_line;
    let mut row = 0;

    // Skip table header row
    if line_num < lines.len() && lines[line_num].starts_with("|") {
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
        if line.starts_with("|") {
            parse_table_row(line, row, layer).with_context(|| {
                format!("Error parsing table row {} at line {}", row, line_num + 1)
            })?;
            row += 1;
        }

        line_num += 1;
    }

    Ok(line_num)
}

/// Parses a single table row into key definitions.
fn parse_table_row(line: &str, row: u8, layer: &mut Layer) -> Result<()> {
    // Split by pipes and trim
    let cells: Vec<&str> = line
        .split('|')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();

    for (col, cell) in cells.iter().enumerate() {
        // Skip empty cells
        if cell.is_empty() {
            continue;
        }

        // Parse keycode syntax
        let key = parse_keycode_syntax(cell, row, col as u8)
            .with_context(|| format!("Error parsing cell at row {}, col {}: {}", row, col, cell))?;

        layer.add_key(key);
    }

    Ok(())
}

/// Parses keycode syntax with optional color and category.
///
/// Formats:
/// - `KC_X` - basic keycode
/// - `KC_X{#RRGGBB}` - with color override
/// - `KC_X@category-id` - with category
/// - `KC_X{#RRGGBB}@category-id` - with both
fn parse_keycode_syntax(cell: &str, row: u8, col: u8) -> Result<KeyDefinition> {
    let keycode_regex =
        Regex::new(r"^([A-Z_0-9()]+)(?:\{(#[0-9A-Fa-f]{6})\})?(?:@([a-z][a-z0-9-]*))?\s*$")
            .unwrap();

    let captures = keycode_regex
        .captures(cell)
        .ok_or_else(|| anyhow::anyhow!("Invalid keycode syntax: {}", cell))?;

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

/// Parses the categories section.
fn parse_categories(lines: &[&str], start_line: usize, layout: &mut Layout) -> Result<usize> {
    let mut line_num = start_line + 1; // Skip "## Categories" header

    let category_regex =
        Regex::new(r"^-\s+([a-z][a-z0-9-]*):\s+(.+?)\s+\(#([0-9A-Fa-f]{6})\)$").unwrap();

    while line_num < lines.len() {
        let line = lines[line_num].trim();

        // Skip empty lines
        if line.is_empty() {
            line_num += 1;
            continue;
        }

        // Stop at next section
        if line.starts_with("##") {
            break;
        }

        // Parse category line: - id: Name (#RRGGBB)
        if let Some(captures) = category_regex.captures(line) {
            let id = captures[1].to_string();
            let name = captures[2].to_string();
            let color_hex = format!("#{}", &captures[3]);
            let color = RgbColor::from_hex(&color_hex)?;

            let category = Category::new(&id, &name, color)?;
            layout.add_category(category)?;
        }

        line_num += 1;
    }

    Ok(line_num)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_frontmatter() {
        let lines = vec![
            "---",
            "name: \"Test Layout\"",
            "description: \"A test layout\"",
            "author: \"test\"",
            "created: \"2024-01-15T10:30:00Z\"",
            "modified: \"2024-01-20T15:45:00Z\"",
            "tags: [\"test\", \"example\"]",
            "is_template: false",
            "version: \"1.0\"",
            "---",
            "",
            "# Content starts here",
        ];

        let (metadata, content_start) = parse_frontmatter(&lines).unwrap();
        assert_eq!(metadata.name, "Test Layout");
        assert_eq!(metadata.description, "A test layout");
        assert_eq!(metadata.author, "test");
        assert_eq!(metadata.tags, vec!["test", "example"]);
        assert!(!metadata.is_template);
        assert_eq!(metadata.version, "1.0");
        assert_eq!(content_start, 10);
    }

    #[test]
    fn test_parse_keycode_syntax() {
        // Basic keycode
        let key = parse_keycode_syntax("KC_A", 0, 0).unwrap();
        assert_eq!(key.keycode, "KC_A");
        assert_eq!(key.position, Position::new(0, 0));
        assert_eq!(key.color_override, None);
        assert_eq!(key.category_id, None);

        // With color override
        let key = parse_keycode_syntax("KC_A{#FF0000}", 0, 1).unwrap();
        assert_eq!(key.keycode, "KC_A");
        assert_eq!(key.color_override, Some(RgbColor::new(255, 0, 0)));

        // With category
        let key = parse_keycode_syntax("KC_LEFT@navigation", 1, 0).unwrap();
        assert_eq!(key.keycode, "KC_LEFT");
        assert_eq!(key.category_id, Some("navigation".to_string()));

        // With both
        let key = parse_keycode_syntax("KC_A{#00FF00}@symbols", 1, 1).unwrap();
        assert_eq!(key.keycode, "KC_A");
        assert_eq!(key.color_override, Some(RgbColor::new(0, 255, 0)));
        assert_eq!(key.category_id, Some("symbols".to_string()));
    }

    #[test]
    fn test_parse_complete_layout() {
        let content = r#"---
name: "Test Layout"
description: "A test"
author: "test"
created: "2024-01-15T10:30:00Z"
modified: "2024-01-20T15:45:00Z"
tags: ["test"]
is_template: false
version: "1.0"
---

# Test Layout

## Layer 0: Base
**Color**: #808080

| C0   | C1   |
|------|------|
| KC_A | KC_B |
| KC_C | KC_D |

## Categories

- navigation: Navigation (#0000FF)
"#;

        let layout = parse_markdown_layout_str(content).unwrap();
        assert_eq!(layout.metadata.name, "Test Layout");
        assert_eq!(layout.layers.len(), 1);
        assert_eq!(layout.layers[0].keys.len(), 4);
        println!("Parsed {} categories", layout.categories.len());
        for cat in &layout.categories {
            println!("Category: {} - {} - {:?}", cat.id, cat.name, cat.color);
        }
        assert_eq!(layout.categories.len(), 1);
    }
}
