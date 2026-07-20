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
    let content =
        serde_json::to_string_pretty(layout).context("Failed to serialize layout to JSON")?;

    // Atomic write: write to .json.tmp, then rename to .json
    let temp_path = path.with_extension("json.tmp");
    fs::write(&temp_path, &content)
        .with_context(|| format!("Failed to write temporary file to {}", temp_path.display()))?;
    fs::rename(&temp_path, path).with_context(|| {
        format!(
            "Failed to rename {} to {}",
            temp_path.display(),
            path.display()
        )
    })?;

    Ok(())
}

#[cfg(test)]
mod tests;

