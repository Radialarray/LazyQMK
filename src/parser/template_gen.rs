//! Markdown layout file generation (serialization).
//!
//! This module handles generating human-readable Markdown files from Layout structures,
//! with atomic file writes for safety.

use crate::models::Layout;
use anyhow::{Context, Result};
use std::path::Path;

/// Generates a Markdown layout file from a Layout structure.
///
/// This performs an atomic write using a temp file + rename pattern to ensure
/// the file is never left in a corrupted state.
///
/// # Errors
///
/// Returns errors for:
/// - File I/O failures
/// - Permission issues
/// - Atomic rename failures
pub fn save_markdown_layout(layout: &Layout, path: &Path) -> Result<()> {
    let markdown = generate_markdown(layout)?;
    atomic_write(path, &markdown)
}

/// Generates Markdown content from a Layout.
pub fn generate_markdown(layout: &Layout) -> Result<String> {
    let mut output = String::new();

    // Generate frontmatter
    output.push_str(&generate_frontmatter(layout)?);
    output.push('\n');

    // Generate title
    output.push_str(&format!("# {}\n\n", layout.metadata.name));

    // Generate layers
    for layer in &layout.layers {
        output.push_str(&generate_layer(layer)?);
        output.push('\n');
    }

    // Generate categories section if any exist
    if !layout.categories.is_empty() {
        output.push_str("---\n\n");
        output.push_str(&generate_categories(layout));
    }

    Ok(output)
}

/// Generates YAML frontmatter from metadata.
fn generate_frontmatter(layout: &Layout) -> Result<String> {
    let yaml =
        serde_yaml::to_string(&layout.metadata).context("Failed to serialize metadata to YAML")?;

    Ok(format!("---\n{yaml}---\n"))
}

/// Generates a layer section with header, properties, and table.
fn generate_layer(layer: &crate::models::Layer) -> Result<String> {
    let mut output = String::new();

    // Layer header: ## Layer N: Name
    output.push_str(&format!("## Layer {}: {}\n", layer.number, layer.name));

    // Layer color: **Color**: #RRGGBB
    output.push_str(&format!("**Color**: {}\n", layer.default_color.to_hex()));

    // Optional layer category
    if let Some(cat_id) = &layer.category_id {
        output.push_str(&format!("**Category**: {cat_id}\n"));
    }

    // Layer colors enabled (only write if false, since true is the default)
    if !layer.layer_colors_enabled {
        output.push_str("**Layer Colors**: false\n");
    }

    output.push('\n');

    // Generate table
    output.push_str(&generate_table(layer)?);

    Ok(output)
}

/// Generates a Markdown table for a layer's keys.
fn generate_table(layer: &crate::models::Layer) -> Result<String> {
    use std::collections::HashMap;

    if layer.keys.is_empty() {
        return Ok(String::new());
    }

    // Group keys by row
    let mut rows: HashMap<u8, Vec<_>> = HashMap::new();
    let mut max_col = 0;

    for key in &layer.keys {
        let row = key.position.row;
        let col = key.position.col;

        max_col = max_col.max(col);
        rows.entry(row).or_default().push(key);
    }

    let num_cols = (max_col + 1) as usize;
    let mut row_nums: Vec<_> = rows.keys().copied().collect();
    row_nums.sort_unstable();

    let mut output = String::new();

    // Generate header row
    output.push('|');
    for col in 0..num_cols {
        output.push_str(&format!(" C{col} |"));
    }
    output.push('\n');

    // Generate separator row
    output.push('|');
    for _ in 0..num_cols {
        output.push_str("------|");
    }
    output.push('\n');

    // Generate data rows
    for row_num in row_nums {
        output.push('|');
        let row_keys = rows.get(&row_num).unwrap();

        // Create a map for quick lookup by column
        let mut col_map: HashMap<u8, &crate::models::KeyDefinition> = HashMap::new();
        for key in row_keys {
            col_map.insert(key.position.col, key);
        }

        for col in 0..num_cols {
            if let Some(key) = col_map.get(&(col as u8)) {
                output.push(' ');
                output.push_str(&serialize_keycode_syntax(key));
                output.push_str(" |");
            } else {
                output.push_str("  |"); // Empty cell
            }
        }
        output.push('\n');
    }

    output.push('\n');
    Ok(output)
}

/// Serializes a keycode with optional color and category syntax.
///
/// Formats:
/// - `KC_X` - basic keycode
/// - `KC_X{#RRGGBB}` - with color override
/// - `KC_X@category-id` - with category
/// - `KC_X{#RRGGBB}@category-id` - with both
fn serialize_keycode_syntax(key: &crate::models::KeyDefinition) -> String {
    let mut result = key.keycode.clone();

    // Add color override if present
    if let Some(color) = key.color_override {
        result.push_str(&format!("{{{}}}", color.to_hex()));
    }

    // Add category if present
    if let Some(cat_id) = &key.category_id {
        result.push_str(&format!("@{cat_id}"));
    }

    result
}

/// Generates the categories section.
fn generate_categories(layout: &Layout) -> String {
    let mut output = String::from("## Categories\n\n");

    for category in &layout.categories {
        output.push_str(&format!(
            "- {}: {} ({})\n",
            category.id,
            category.name,
            category.color.to_hex()
        ));
    }

    output
}

/// Performs an atomic file write using temp file + rename pattern.
///
/// This ensures the target file is never left in a corrupted state:
/// 1. Write to temporary file
/// 2. Verify write success
/// 3. Atomic rename to target path
fn atomic_write(path: &Path, content: &str) -> Result<()> {
    // Create temporary file path
    let temp_path = path.with_extension("md.tmp");

    // Write to temp file
    std::fs::write(&temp_path, content)
        .with_context(|| format!("Failed to write to temporary file: {}", temp_path.display()))?;

    // Atomic rename
    std::fs::rename(&temp_path, path)
        .with_context(|| format!("Failed to rename temporary file to: {}", path.display()))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Category, KeyDefinition, Layer, LayoutMetadata, Position, RgbColor};
    use crate::parser::layout::parse_markdown_layout_str;
    use chrono::Utc;

    fn create_test_layout() -> Layout {
        let metadata = LayoutMetadata {
            name: "Test Layout".to_string(),
            description: "A test layout".to_string(),
            author: "test".to_string(),
            created: Utc::now(),
            modified: Utc::now(),
            tags: vec!["test".to_string()],
            is_template: false,
            version: "1.0".to_string(),
            layout_variant: None,
        };

        let mut layer = Layer {
            number: 0,
            name: "Base".to_string(),
            default_color: RgbColor::new(128, 128, 128),
            category_id: None,
            keys: vec![],
            layer_colors_enabled: true,
        };

        // Add some keys
        layer.keys.push(KeyDefinition {
            position: Position { row: 0, col: 0 },
            keycode: "KC_A".to_string(),
            label: None,
            color_override: None,
            category_id: None,
            combo_participant: false,
        });

        layer.keys.push(KeyDefinition {
            position: Position { row: 0, col: 1 },
            keycode: "KC_B".to_string(),
            label: None,
            color_override: Some(RgbColor::new(255, 0, 0)),
            category_id: None,
            combo_participant: false,
        });

        let category = Category {
            id: "navigation".to_string(),
            name: "Navigation".to_string(),
            color: RgbColor::new(0, 0, 255),
        };

        Layout {
            metadata,
            layers: vec![layer],
            categories: vec![category],
        }
    }

    #[test]
    fn test_generate_frontmatter() {
        let layout = create_test_layout();
        let frontmatter = generate_frontmatter(&layout).unwrap();

        println!("Generated frontmatter:\n{frontmatter}");

        assert!(frontmatter.starts_with("---\n"));
        assert!(frontmatter.ends_with("---\n"));
        // YAML may use single quotes or no quotes depending on content
        assert!(frontmatter.contains("name:") && frontmatter.contains("Test Layout"));
        assert!(frontmatter.contains("version:") && frontmatter.contains("1.0"));
    }

    #[test]
    fn test_serialize_keycode_syntax() {
        // Basic keycode
        let key = KeyDefinition {
            position: Position { row: 0, col: 0 },
            keycode: "KC_A".to_string(),
            label: None,
            color_override: None,
            category_id: None,
            combo_participant: false,
        };
        assert_eq!(serialize_keycode_syntax(&key), "KC_A");

        // With color override
        let key_with_color = KeyDefinition {
            position: Position { row: 0, col: 0 },
            keycode: "KC_A".to_string(),
            label: None,
            color_override: Some(RgbColor::new(255, 0, 0)),
            category_id: None,
            combo_participant: false,
        };
        assert_eq!(serialize_keycode_syntax(&key_with_color), "KC_A{#FF0000}");

        // With category
        let key_with_category = KeyDefinition {
            position: Position { row: 0, col: 0 },
            keycode: "KC_LEFT".to_string(),
            label: None,
            color_override: None,
            category_id: Some("navigation".to_string()),
            combo_participant: false,
        };
        assert_eq!(
            serialize_keycode_syntax(&key_with_category),
            "KC_LEFT@navigation"
        );

        // With both
        let key_with_both = KeyDefinition {
            position: Position { row: 0, col: 0 },
            keycode: "KC_A".to_string(),
            label: None,
            color_override: Some(RgbColor::new(0, 255, 0)),
            category_id: Some("symbols".to_string()),
            combo_participant: false,
        };
        assert_eq!(
            serialize_keycode_syntax(&key_with_both),
            "KC_A{#00FF00}@symbols"
        );
    }

    #[test]
    fn test_generate_categories() {
        let layout = create_test_layout();
        let categories_section = generate_categories(&layout);

        assert!(categories_section.contains("## Categories"));
        assert!(categories_section.contains("- navigation: Navigation (#0000FF)"));
    }

    #[test]
    fn test_round_trip() {
        let layout = create_test_layout();

        // Generate markdown
        let markdown = generate_markdown(&layout).unwrap();

        println!("Generated markdown:\n{markdown}");

        // Parse it back
        let parsed_layout = parse_markdown_layout_str(&markdown).unwrap();

        // Verify key data is preserved
        assert_eq!(parsed_layout.metadata.name, layout.metadata.name);
        assert_eq!(parsed_layout.layers.len(), layout.layers.len());
        println!("Original categories: {}", layout.categories.len());
        println!("Parsed categories: {}", parsed_layout.categories.len());
        assert_eq!(parsed_layout.categories.len(), layout.categories.len());
        assert_eq!(
            parsed_layout.layers[0].keys.len(),
            layout.layers[0].keys.len()
        );
    }
}
