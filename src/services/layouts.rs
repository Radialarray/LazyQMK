//! Layout file I/O service.
//!
//! This module centralizes all layout file operations, providing a consistent
//! interface for loading, saving, and managing layout files.

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

use crate::{models::Layout, parser};

/// Service for managing layout file I/O operations.
///
/// This service centralizes all layout file operations to ensure consistent
/// handling of file paths, error messages, and file system operations.
pub struct LayoutService;

impl LayoutService {
    /// Loads a layout from a Markdown file.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the layout file to load
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
    /// use keyboard_configurator::services::LayoutService;
    ///
    /// let layout = LayoutService::load(Path::new("my_layout.md"))?;
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn load(path: &Path) -> Result<Layout> {
        parser::parse_markdown_layout(path)
            .with_context(|| format!("Failed to load layout from {}", path.display()))
    }

    /// Saves a layout to a Markdown file.
    ///
    /// This performs an atomic write using a temp file + rename pattern to ensure
    /// the file is never left in a corrupted state.
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
    /// use keyboard_configurator::{models::Layout, services::LayoutService};
    ///
    /// # fn example() -> anyhow::Result<()> {
    /// let layout = Layout::new("My Layout")?;
    /// LayoutService::save(&layout, Path::new("my_layout.md"))?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn save(layout: &Layout, path: &Path) -> Result<()> {
        parser::save_markdown_layout(layout, path)
            .with_context(|| format!("Failed to save layout to {}", path.display()))
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
    /// use keyboard_configurator::services::LayoutService;
    ///
    /// let old_path = Path::new("old_layout.md");
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
        // Replace characters that are problematic in filenames
        let sanitized_name = sanitize_filename(new_name);

        // Build new path with sanitized name
        let new_path = parent.join(format!("{}.md", sanitized_name));

        // Check if rename is needed
        if new_path == old_path {
            return Ok(None);
        }

        // Perform the rename
        std::fs::rename(old_path, &new_path)
            .with_context(|| {
                format!(
                    "Failed to rename layout file from {} to {}",
                    old_path.display(),
                    new_path.display()
                )
            })?;

        Ok(Some(new_path))
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
/// # use keyboard_configurator::services::layouts::sanitize_filename;
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
    fn test_rename_file_if_needed_no_file() {
        let path = Path::new("/tmp/nonexistent_layout_test_12345.md");
        let result = LayoutService::rename_file_if_needed(path, "New Name");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), None);
    }

    #[test]
    fn test_rename_file_if_needed_same_name() -> Result<()> {
        // Create a temporary file
        let temp_dir = std::env::temp_dir();
        let test_path = temp_dir.join("test_layout.md");
        fs::write(&test_path, "test content")?;

        // Try to rename with same name
        let result = LayoutService::rename_file_if_needed(&test_path, "test_layout")?;
        assert_eq!(result, None);

        // Verify file still exists at original location
        assert!(test_path.exists());

        // Clean up
        fs::remove_file(&test_path)?;
        Ok(())
    }

    #[test]
    fn test_rename_file_if_needed_new_name() -> Result<()> {
        // Create a temporary file
        let temp_dir = std::env::temp_dir();
        let old_path = temp_dir.join("old_layout_test_xyz.md");
        fs::write(&old_path, "test content")?;

        // Rename with new name
        let result = LayoutService::rename_file_if_needed(&old_path, "new_layout")?;
        assert!(result.is_some());

        let new_path = result.unwrap();
        assert_eq!(new_path, temp_dir.join("new_layout.md"));

        // Verify file moved
        assert!(!old_path.exists());
        assert!(new_path.exists());
        assert_eq!(fs::read_to_string(&new_path)?, "test content");

        // Clean up
        fs::remove_file(&new_path)?;
        Ok(())
    }

    #[test]
    fn test_rename_file_sanitizes_name() -> Result<()> {
        // Create a temporary file
        let temp_dir = std::env::temp_dir();
        let old_path = temp_dir.join("old_layout_test_abc.md");
        fs::write(&old_path, "test content")?;

        // Rename with name that needs sanitization
        let result = LayoutService::rename_file_if_needed(&old_path, "New/Layout:Name Test")?;
        assert!(result.is_some());

        let new_path = result.unwrap();
        assert_eq!(new_path, temp_dir.join("new_layout_name_test.md"));

        // Verify file moved
        assert!(!old_path.exists());
        assert!(new_path.exists());

        // Clean up
        fs::remove_file(&new_path)?;
        Ok(())
    }
}
