//! Layout file I/O service.
//!
//! This module centralizes all layout file operations, providing a consistent
//! interface for loading, saving, and managing layout files.
//!
//! # Format
//!
//! Since 0.22.0 layouts are stored in **JSON** format (`.json`). Legacy `.md`
//! files are automatically detected and migrated to `.json` on first load.
//!
//! # Migration
//!
//! When a `.md` file is loaded:
//! 1. The legacy Markdown parser reads the file
//! 2. The layout is immediately written as `.json`
//! 3. The `.md` file is renamed to `.md.bak`
//!
//! This ensures zero-touch migration for existing users.

use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

use crate::{models::Layout, parser};

/// Service for managing layout file I/O operations.
///
/// This service centralizes all layout file operations to ensure consistent
/// handling of file paths, error messages, and file system operations.
pub struct LayoutService;

impl LayoutService {
    /// Loads a layout from file, auto-detecting format.
    ///
    /// Supports both `.json` (current) and `.md` (legacy) files.
    /// When a `.md` file is loaded, it is automatically migrated to `.json`.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the layout file (`.json` or `.md`)
    ///
    /// # Returns
    ///
    /// * `Ok(Layout)` - Successfully parsed layout
    /// * `Err(...)` - File not found, parse error, or I/O error
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::path::Path;
    /// use lazyqmk::services::LayoutService;
    ///
    /// let layout = LayoutService::load(Path::new("my_layout.json"))?;
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn load(path: &Path) -> Result<Layout> {
        let ext = path.extension().and_then(|e| e.to_str());

        match ext {
            Some("json") => {
                parser::parse_json_layout(path)
                    .with_context(|| format!("Failed to load layout from {}", path.display()))
            }
            Some("md") => {
                // Legacy .md → load with markdown parser, then migrate to .json
                let layout = parser::parse_markdown_layout(path)
                    .with_context(|| format!("Failed to load legacy .md layout from {}", path.display()))?;

                // Auto-migrate: write .json, rename .md → .md.bak
                Self::migrate_md_to_json(path, &layout)?;

                Ok(layout)
            }
            _ => {
                // No recognized extension — try .json first, then .md as fallback
                let json_path = path.with_extension("json");
                if json_path.exists() {
                    return Self::load(&json_path);
                }

                let md_path = path.with_extension("md");
                if md_path.exists() {
                    return Self::load(&md_path);
                }

                Err(anyhow::anyhow!(
                    "Layout file not found: {} (tried .json and .md)",
                    path.display()
                ))
            }
        }
    }

    /// Saves a layout as JSON.
    ///
    /// If the path has a `.md` extension, it is automatically changed to `.json`.
    /// The write is atomic (temp file + rename).
    ///
    /// # Arguments
    ///
    /// * `layout` - The layout to save
    /// * `path` - Path where the layout should be saved
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Layout successfully saved
    /// * `Err(...)` - I/O error, permission error, or atomic rename failure
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::path::Path;
    /// use lazyqmk::{models::Layout, services::LayoutService};
    ///
    /// # fn example() -> anyhow::Result<()> {
    /// let layout = Layout::new("My Layout")?;
    /// LayoutService::save(&layout, Path::new("my_layout.json"))?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn save(layout: &Layout, path: &Path) -> Result<()> {
        // Always use .json extension
        let json_path = ensure_json_extension(path);
        parser::save_json_layout(layout, &json_path)
            .with_context(|| format!("Failed to save layout to {}", json_path.display()))
    }

    /// Migrates a legacy `.md` file to the current `.json` format.
    ///
    /// 1. Writes the layout as `.json`
    /// 2. Renames the `.md` file to `.md.bak`
    ///
    /// If both steps succeed, the migration is complete. If the JSON write
    /// fails, the `.md` file is left untouched.
    fn migrate_md_to_json(md_path: &Path, layout: &Layout) -> Result<()> {
        let json_path = md_path.with_extension("json");
        let bak_path = md_path.with_extension("md.bak");

        // Step 1: Write .json
        parser::save_json_layout(layout, &json_path)
            .with_context(|| format!("Migration failed: could not write {}", json_path.display()))?;

        // Step 2: Rename .md → .md.bak (silently skip if .md no longer exists)
        if md_path.exists() {
            fs::rename(md_path, &bak_path).with_context(|| {
                format!(
                    "Migration: layout saved as {} but could not rename {} to {}",
                    json_path.display(),
                    md_path.display(),
                    bak_path.display()
                )
            })?;
        }

        Ok(())
    }

    /// Renames a layout file if the layout name has changed.
    ///
    /// This is useful when a layout's name is changed through the metadata editor.
    /// The function sanitizes the new name for use as a filename.
    ///
    /// # Arguments
    ///
    /// * `old_path` - Current path to the layout file
    /// * `new_name` - New name for the layout (will be sanitized)
    ///
    /// # Returns
    ///
    /// * `Ok(Some(PathBuf))` - File was renamed, returns the new path
    /// * `Ok(None)` - No rename needed (same filename or file doesn't exist)
    /// * `Err(...)` - Failed to rename file
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::path::Path;
    /// use lazyqmk::services::LayoutService;
    ///
    /// let old_path = Path::new("old_layout.json");
    /// if let Some(new_path) = LayoutService::rename_file_if_needed(old_path, "New Layout Name")? {
    ///     println!("Layout renamed to: {}", new_path.display());
    /// }
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn rename_file_if_needed(old_path: &Path, new_name: &str) -> Result<Option<PathBuf>> {
        // Check if file exists
        if !old_path.exists() {
            return Ok(None);
        }

        // Get parent directory
        let parent = old_path
            .parent()
            .context("Layout file has no parent directory")?;

        // Sanitize the new name for use as a filename
        let sanitized_name = sanitize_filename(new_name);

        // Build new path with .json extension
        let new_path = parent.join(format!("{}.json", sanitized_name));

        // Check if rename is needed
        if new_path == old_path {
            return Ok(None);
        }

        // Perform the rename
        fs::rename(old_path, &new_path).with_context(|| {
            format!(
                "Failed to rename layout file from {} to {}",
                old_path.display(),
                new_path.display()
            )
        })?;

        Ok(Some(new_path))
    }
}

/// Ensures a path uses `.json` extension. If the path has `.md` or no
/// extension, it is replaced/appended with `.json`.
fn ensure_json_extension(path: &Path) -> PathBuf {
    let ext = path.extension().and_then(|e| e.to_str());
    match ext {
        Some("json") => path.to_path_buf(),
        _ => path.with_extension("json"),
    }
}

/// Sanitizes a layout name for use as a filename.
///
/// Replaces problematic characters with underscores and converts to lowercase.
///
/// # Arguments
///
/// * `name` - The layout name to sanitize
///
/// # Returns
///
/// A sanitized filename-safe string
///
/// # Examples
///
/// ```
/// # use lazyqmk::services::layouts::sanitize_filename;
/// assert_eq!(sanitize_filename("My Layout"), "my_layout");
/// assert_eq!(sanitize_filename("Layout/Name:Test"), "layout_name_test");
/// ```
pub fn sanitize_filename(name: &str) -> String {
    name.replace(['/', '\\', ':', ' '], "_").to_lowercase()
}

#[cfg(test)]
mod tests {
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
        assert!(tmp.path().join("legacy.json").exists(), "Expected migrated .json");
        assert!(tmp.path().join("legacy.md.bak").exists(), "Expected .md.bak backup");

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
        assert!(tmp.path().join("test.json").exists(), "Should write .json instead");

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
}
