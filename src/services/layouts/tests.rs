//! Tests for layouts.
//!
//! Auto-extracted from layouts.rs.

use super::*;

use super::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_sanitize_filename() {
    assert_eq!(sanitize_filename("My Layout"), "my_layout");
    assert_eq!(sanitize_filename("Layout/Name"), "layout_name");
    assert_eq!(sanitize_filename("Layout\\Name"), "layout_name");
    assert_eq!(sanitize_filename("Layout:Name"), "layout_name");
    assert_eq!(sanitize_filename("Layout Name Test"), "layout_name_test");
    assert_eq!(
        sanitize_filename("Complex/Layout\\Name:Test 123"),
        "complex_layout_name_test_123"
    );
}

#[test]
fn test_json_roundtrip_via_service() -> Result<()> {
    let tmp = TempDir::new()?;
    let path = tmp.path().join("test_roundtrip.json");

    let mut layout = Layout::new("roundtrip_test")?;
    layout.metadata.author = "tester".to_string();
    layout.add_layer(crate::models::Layer::new(
        0,
        "Base",
        crate::models::RgbColor::new(255, 255, 255),
    )?)?;

    LayoutService::save(&layout, &path)?;
    assert!(path.exists());

    let loaded = LayoutService::load(&path)?;
    assert_eq!(loaded.metadata.name, "roundtrip_test");
    assert_eq!(loaded.metadata.author, "tester");

    Ok(())
}

#[test]
fn test_load_without_extension_tries_json_first() -> Result<()> {
    let tmp = TempDir::new()?;
    let json_path = tmp.path().join("my_layout.json");
    let no_ext_path = tmp.path().join("my_layout");

    let mut layout = Layout::new("my_layout")?;
    layout.add_layer(crate::models::Layer::new(
        0,
        "Base",
        crate::models::RgbColor::new(255, 255, 255),
    )?)?;
    LayoutService::save(&layout, &json_path)?;

    // Load without extension → should find .json
    let loaded = LayoutService::load(&no_ext_path)?;
    assert_eq!(loaded.metadata.name, "my_layout");

    Ok(())
}

#[test]
fn test_load_without_extension_falls_back_to_md() -> Result<()> {
    let tmp = TempDir::new()?;
    let md_path = tmp.path().join("legacy.md");
    let no_ext_path = tmp.path().join("legacy");

    // Create a minimal .md layout file
    let md_content = r#"---
name: legacy_layout
description: ''
author: ''
created: 2025-01-01T00:00:00Z
modified: 2025-01-01T00:00:00Z
tags: []
is_template: false
version: '1.0'
---

# legacy_layout

## Layer 0: Base
**ID**: 00000000-0000-0000-0000-000000000000
**Color**: #808080

| C0 |
|-----|
| KC_A |
"#;
    fs::write(&md_path, md_content)?;

    // Load without extension → should find .md as fallback
    let loaded = LayoutService::load(&no_ext_path)?;
    assert_eq!(loaded.metadata.name, "legacy_layout");

    // After loading, the .json should exist (migration) and .md should be .md.bak
    assert!(
        tmp.path().join("legacy.json").exists(),
        "Expected migrated .json"
    );
    assert!(
        tmp.path().join("legacy.md.bak").exists(),
        "Expected .md.bak backup"
    );

    Ok(())
}

#[test]
fn test_save_always_uses_json_extension() -> Result<()> {
    let tmp = TempDir::new()?;

    // Save with .md path → should still write .json
    let md_path = tmp.path().join("test.md");
    let layout = Layout::new("test")?;
    LayoutService::save(&layout, &md_path)?;

    assert!(!md_path.exists(), "Should NOT write .md file");
    assert!(
        tmp.path().join("test.json").exists(),
        "Should write .json instead"
    );

    Ok(())
}

#[test]
fn test_rename_file_if_needed_no_file() -> Result<()> {
    let path = Path::new("/tmp/nonexistent_layout_test_12345.json");
    let result = LayoutService::rename_file_if_needed(path, "New Name");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), None);
    Ok(())
}

#[test]
fn test_rename_file_if_needed_same_name() -> Result<()> {
    let tmp = TempDir::new()?;
    let test_path = tmp.path().join("test_layout.json");
    fs::write(&test_path, "{}")?;

    let result = LayoutService::rename_file_if_needed(&test_path, "test_layout")?;
    assert_eq!(result, None);
    assert!(test_path.exists());

    Ok(())
}

#[test]
fn test_rename_file_if_needed_new_name() -> Result<()> {
    let tmp = TempDir::new()?;
    let old_path = tmp.path().join("old_layout.json");
    fs::write(&old_path, "{}")?;

    let result = LayoutService::rename_file_if_needed(&old_path, "new_layout")?;
    assert!(result.is_some());

    let new_path = result.unwrap();
    assert_eq!(new_path, tmp.path().join("new_layout.json"));
    assert!(!old_path.exists());
    assert!(new_path.exists());

    Ok(())
}

#[test]
fn test_migrate_md_to_json_from_file() -> Result<()> {
    let tmp = TempDir::new()?;
    let md_path = tmp.path().join("legacy.md");
    let json_path = tmp.path().join("legacy.json");
    let bak_path = tmp.path().join("legacy.md.bak");

    // Create minimal markdown layout
    let md_content = r#"---
name: migration_test
description: ''
author: tester
created: 2025-06-01T00:00:00Z
modified: 2025-06-01T00:00:00Z
tags: []
is_template: false
version: '1.0'
---

# migration_test

## Layer 0: Base
**ID**: 00000000-0000-0000-0000-000000000000
**Color**: #808080

| C0 |
|-----|
| KC_A |
"#;
    fs::write(&md_path, md_content)?;

    // Load via LayoutService (triggers migration)
    let layout = LayoutService::load(&md_path)?;
    assert_eq!(layout.metadata.name, "migration_test");
    assert_eq!(layout.metadata.author, "tester");

    // Verify migration artifacts
    assert!(json_path.exists(), "JSON file should exist after migration");
    assert!(bak_path.exists(), ".md.bak backup should exist");

    // Verify the .json is loadable as a proper layout
    let migrated = LayoutService::load(&json_path)?;
    assert_eq!(migrated.metadata.name, "migration_test");
    assert_eq!(migrated.metadata.author, "tester");

    Ok(())
}
