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

use crate::services::file_watcher::{mark_self_write, SelfWriteEpoch};
use crate::services::layout_versions::LayoutVersionService;
use crate::{models::Layout, parser};

/// Service for managing layout file I/O operations.
///
/// This service centralizes all layout file operations to ensure consistent
/// handling of file paths, error messages, and file system operations.
pub struct LayoutService;

impl LayoutService {
    /// Loads a layout from file, auto-detecting format.
    ///
    /// Supports the new per-layout folder layout (`<layouts>/<name>/current.json`)
    /// as well as legacy `.json` and `.md` flat files. When a legacy file is
    /// loaded, it is automatically migrated to the folder layout (moved into
    /// `<layouts>/<name>/current.json` with an initial revision).
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the layout file or folder (`.json`, `.md`, or
    ///   stem-only that resolves to one of those).
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
        // 1. If path points to a folder-style layout (`<stem>/current.json`), use it.
        if let Some(parent) = path.parent() {
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                let current = parent.join(stem).join("current.json");
                if current.exists() {
                    return parser::parse_json_layout(&current).with_context(|| {
                        format!("Failed to load layout from {}", current.display())
                    });
                }
            }
        }

        let ext = path.extension().and_then(|e| e.to_str());

        match ext {
            Some("json") => {
                if path.exists() {
                    let layout = parser::parse_json_layout(path)
                        .with_context(|| format!("Failed to load layout from {}", path.display()))?;

                    // Migrate legacy flat .json -> per-layout folder.
                    if let Some(svc) = LayoutVersionService::from_layout_path(path) {
                        let _ = svc.migrate_legacy_path(path);
                    }

                    Ok(layout)
                } else {
                    // Flat file no longer exists (e.g., already migrated).
                    // Try the folder layout as a fallback.
                    if let Some(parent) = path.parent() {
                        if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                            let current = parent.join(stem).join("current.json");
                            if current.exists() {
                                return Self::load(&current);
                            }
                        }
                    }
                    Err(anyhow::anyhow!("Layout file not found: {}", path.display()))
                }
            }
            Some("md") => {
                let layout = parser::parse_markdown_layout(path).with_context(|| {
                    format!("Failed to load legacy .md layout from {}", path.display())
                })?;

                // Auto-migrate .md -> .json (in-place)
                Self::migrate_md_to_json(path, &layout)?;
                // Then migrate the .json -> folder layout
                let json_path = path.with_extension("json");
                if let Some(svc) = LayoutVersionService::from_layout_path(path) {
                    let _ = svc.migrate_legacy_path(&json_path);
                }
                Ok(layout)
            }
            _ => {
                // No recognized extension — try folder, .json, then .md.
                if let Some(parent) = path.parent() {
                    if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                        let current = parent.join(stem).join("current.json");
                        if current.exists() {
                            return Self::load(&current);
                        }
                    }
                }

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
        Self::save_with_epoch(layout, path, None)
    }

    /// Saves a layout as JSON, optionally marking a self-write epoch so
    /// the hot-reload watcher can suppress the resulting file change
    /// event.
    ///
    /// The TUI and the web backend pass a shared `SelfWriteEpoch` so
    /// the watcher they registered does not echo their own writes back
    /// as "external" changes. CLI commands (which never spawn a
    /// watcher) can simply call [`Self::save`].
    ///
    /// The mark is performed *before* the atomic write so the watcher
    /// sees a timestamp at or after the mark when the rename lands.
    pub fn save_with_epoch(
        layout: &Layout,
        path: &Path,
        self_write_epoch: Option<&SelfWriteEpoch>,
    ) -> Result<()> {
        if let Some(epoch) = self_write_epoch {
            mark_self_write(epoch);
        }
        // Always use .json extension
        let json_path = ensure_json_extension(path);

        // Primary write target.
        let folder_current = json_path
            .parent()
            .and_then(|p| json_path.file_stem().and_then(|s| s.to_str()).map(|stem| p.join(stem).join("current.json")))
            .filter(|p| p.exists());

        // If the layout folder already exists, the folder layout is the
        // source of truth. Write there first, then mirror to the flat file
        // so external tools/scripts that still point at the original path
        // observe the latest state.
        if let Some(folder_path) = folder_current {
            parser::save_json_layout(layout, &folder_path)
                .with_context(|| format!("Failed to save layout to {}", folder_path.display()))?;
            // Mirror to the flat file (best-effort).
            if let Err(e) = parser::save_json_layout(layout, &json_path) {
                eprintln!(
                    "Warning: failed to mirror layout to {}: {e}",
                    json_path.display()
                );
            }
            Ok(())
        } else {
            parser::save_json_layout(layout, &json_path)
                .with_context(|| format!("Failed to save layout to {}", json_path.display()))
        }
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
        parser::save_json_layout(layout, &json_path).with_context(|| {
            format!("Migration failed: could not write {}", json_path.display())
        })?;

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

/// Returns true when a layout exists at the given path — either as a legacy
/// flat file or as a per-layout folder layout (`<stem>/current.json`).
#[allow(dead_code)] // Public API; also referenced by tests/web for future folder-aware checks.
#[must_use]
pub fn layout_exists_at(path: &Path) -> bool {
    if path.exists() {
        return true;
    }
    path.parent()
        .and_then(|p| path.file_stem().and_then(|s| s.to_str()).map(|stem| p.join(stem).join("current.json")))
        .is_some_and(|p| p.exists())
}

#[cfg(test)]
mod tests;
