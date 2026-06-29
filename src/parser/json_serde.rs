//! JSON serialization/deserialization for keyboard layouts.
//!
//! This module provides a pure serde-based alternative to the custom Markdown
//! parser. The `Layout` struct already derives `Serialize`/`Deserialize`, so
//! reading and writing JSON is a single function call each.
//!
//! # Migration
//!
//! Legacy `.md` files are auto-detected and migrated to `.json` by
//! [`LayoutService`](crate::services::LayoutService) on load.

use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

use crate::models::Layout;

/// Reads a `Layout` from a JSON file.
///
/// # Arguments
///
/// * `path` - Path to the `.json` layout file
///
/// # Errors
///
/// Returns an error if the file cannot be read or the JSON is malformed.
pub fn parse_json_layout(path: &Path) -> Result<Layout> {
    let content =
        fs::read_to_string(path).with_context(|| format!("Failed to read {}", path.display()))?;
    parse_json_layout_str(&content)
        .with_context(|| format!("Failed to parse JSON layout from {}", path.display()))
}

/// Parses a `Layout` from a JSON string.
pub fn parse_json_layout_str(content: &str) -> Result<Layout> {
    let mut layout: Layout = serde_json::from_str(content)?;

    // Auto-create missing tap dance definitions for any TD() references
    layout.auto_create_tap_dances();

    // Validate the parsed layout (matches markdown parser behavior)
    layout.validate()?;

    Ok(layout)
}

/// Writes a `Layout` to a JSON file using atomic write (temp + rename).
///
/// # Arguments
///
/// * `layout` - The layout to serialize
/// * `path` - Target path (should end in `.json`)
///
/// # Errors
///
/// Returns an error if serialization or file I/O fails.
pub fn save_json_layout(layout: &Layout, path: &Path) -> Result<()> {
    let content = serde_json::to_string_pretty(layout)
        .context("Failed to serialize layout to JSON")?;

    // Atomic write: write to .json.tmp, then rename to .json
    let temp_path = path.with_extension("json.tmp");
    fs::write(&temp_path, &content)
        .with_context(|| format!("Failed to write temporary file to {}", temp_path.display()))?;
    fs::rename(&temp_path, path)
        .with_context(|| format!("Failed to rename {} to {}", temp_path.display(), path.display()))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Category, RgbColor};
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_json_roundtrip_basic_layout() -> Result<()> {
        let mut layout = Layout::new("test_roundtrip")?;
        layout.metadata.author = "test".to_string();
        layout.metadata.description = "Roundtrip test".to_string();

        let json = serde_json::to_string_pretty(&layout)?;
        let back: Layout = serde_json::from_str(&json)?;

        assert_eq!(back.metadata.name, "test_roundtrip");
        assert_eq!(back.metadata.author, "test");
        assert!(back.layers.is_empty());
        assert!(back.tap_dances.is_empty());
        Ok(())
    }

    #[test]
    fn test_json_roundtrip_full_layout_from_example() -> Result<()> {
        let example_path = Path::new("examples/corne_choc_pro_layout.json");
        if !example_path.exists() {
            eprintln!("Skipping: example JSON file not found");
            return Ok(());
        }

        let layout = parse_json_layout(example_path)?;
        assert_eq!(layout.metadata.name, "corne_choc_pro_layout");
        assert_eq!(layout.layers.len(), 7);
        assert_eq!(layout.metadata.layout_variant.as_deref(), Some("LAYOUT_split_3x6_3_ex2"));
        assert_eq!(layout.categories.len(), 4);

        // Verify key details in Base layer
        let base = &layout.layers[0];
        assert_eq!(base.name, "Base");
        assert_eq!(base.keys.len(), 46);

        // Find first non-trns key
        let first_key = base.keys.iter().find(|k| k.keycode != "KC_TRNS" && k.keycode != "KC_NO");
        assert!(first_key.is_some(), "Expected at least one non-trns key");
        Ok(())
    }

    #[test]
    fn test_json_save_and_reload() -> Result<()> {
        let tmp = TempDir::new()?;
        let path = tmp.path().join("test.json");

        let mut layout = Layout::new("save_reload")?;
        layout.metadata.author = "author".to_string();
        layout.categories.push(Category::new(
            "fn",
            "Function",
            RgbColor::new(100, 150, 200),
        )?);
        layout.add_layer(crate::models::Layer::new(
            0,
            "Base",
            RgbColor::new(255, 255, 255),
        )?)?;

        save_json_layout(&layout, &path)?;
        assert!(path.exists());

        let loaded = parse_json_layout(&path)?;
        assert_eq!(loaded.metadata.name, "save_reload");
        assert_eq!(loaded.metadata.author, "author");
        assert_eq!(loaded.categories.len(), 1);
        assert_eq!(loaded.categories[0].id, "fn");

        Ok(())
    }

    #[test]
    fn test_json_parse_invalid_file() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("bad.json");
        fs::write(&path, "not valid json {").unwrap();

        let result = parse_json_layout(&path);
        assert!(result.is_err());
    }

    #[test]
    fn test_json_with_categories_and_tap_dances() -> Result<()> {
        let example_path = Path::new("examples/corne_choc_pro_layout.json");
        if !example_path.exists() {
            return Ok(());
        }
        let layout = parse_json_layout(example_path)?;
        
        // Verify categories
        assert!(!layout.categories.is_empty());
        let nav = layout.categories.iter().find(|c| c.id == "navigation");
        assert!(nav.is_some());
        assert_eq!(nav.unwrap().name, "Navigation");

        // Verify categories are properly referenced in keys
        let base = &layout.layers[0];
        let cat_keys: Vec<_> = base.keys.iter().filter(|k| k.category_id.is_some()).collect();
        assert!(!cat_keys.is_empty(), "Expected some keys with categories");
        Ok(())
    }
}
