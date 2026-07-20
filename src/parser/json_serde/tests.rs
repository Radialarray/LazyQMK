//! Tests for parser::json_serde.
//!
//! Auto-extracted from src/parser/json_serde.rs.
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
    assert_eq!(
        layout.metadata.layout_variant.as_deref(),
        Some("LAYOUT_split_3x6_3_ex2")
    );
    assert_eq!(layout.categories.len(), 4);

    // Verify key details in Base layer
    let base = &layout.layers[0];
    assert_eq!(base.name, "Base");
    assert_eq!(base.keys.len(), 46);

    // Find first non-trns key
    let first_key = base
        .keys
        .iter()
        .find(|k| k.keycode != "KC_TRNS" && k.keycode != "KC_NO");
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
    let cat_keys: Vec<_> = base
        .keys
        .iter()
        .filter(|k| k.category_id.is_some())
        .collect();
    assert!(!cat_keys.is_empty(), "Expected some keys with categories");
    Ok(())
}
