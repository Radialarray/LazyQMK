//! Layout revision service — manages per-layout snapshot folders.
//!
//! # Storage layout
//!
//! Each layout lives in its own folder:
//!
//! ```text
//! <layouts>/my_keymap/
//!   current.json                  # the active revision
//!   manifest.json                 # index + summaries
//!   versions/
//!     1.json
//!     2.json
//!     3-pre-rgb-overhaul.json
//! ```
//!
//! # Recovery
//!
//! If `manifest.json` is missing or corrupted, [`LayoutVersionService::rebuild_manifest_from_disk`]
//! derives a fresh manifest by scanning the `versions/` directory. If
//! `current.json` is missing, the latest revision is promoted to `current.json`
//! (via [`LayoutVersionService::load_or_recover`]).
//!
//! # Atomicity
//!
//! Every write uses temp-file + rename so partial failures don't leave the
//! folder in a half-written state.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use chrono::Utc;
use uuid::Uuid;

use crate::models::{
    auto_label, compute_diff, revision_filename, Layout, LayoutDiff, LayoutManifest,
    LayoutRevision, RevisionSummary,
};
use crate::parser;
use crate::services::file_watcher::{mark_self_write, SelfWriteEpoch};

/// Optional callback invoked after a revision mutation. The web layer
/// passes a closure that publishes an SSE event.
pub type RevisionEventHook = std::sync::Arc<dyn Fn(&str, u32, RevisionEventKind) + Send + Sync>;

/// What kind of revision event just happened.
#[derive(Debug, Clone, Copy)]
pub enum RevisionEventKind {
    /// A new revision was created.
    Created,
    /// A revision was deleted.
    Deleted,
    /// A revision's label or note was edited.
    Renamed,
    /// The current layout was restored from a revision.
    Restored,
}

impl RevisionEventKind {
    /// String identifier used in the SSE payload.
    #[allow(dead_code)] // Reserved for web SSE consumer (see src/web/routes/versions.rs).
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Deleted => "deleted",
            Self::Renamed => "renamed",
            Self::Restored => "restored",
        }
    }
}

/// Default name of the active layout snapshot.
pub const CURRENT_FILE: &str = "current.json";
/// Default name of the manifest.
pub const MANIFEST_FILE: &str = "manifest.json";
/// Subdirectory holding version snapshots.
pub const VERSIONS_DIR: &str = "versions";

/// Construct a "not found" error for a layout.
pub fn not_found(name: &str) -> anyhow::Error {
    anyhow::anyhow!("Layout '{name}' not found")
}

/// Construct a "revision not found" error.
pub fn revision_not_found(revision: u32, name: &str) -> anyhow::Error {
    anyhow::anyhow!("Revision {revision} not found for layout '{name}'")
}

/// Construct a "cannot delete current" error.
pub fn cannot_delete_current(revision: u32, name: &str) -> anyhow::Error {
    anyhow::anyhow!("Cannot delete revision {revision}: it is the active revision for layout '{name}'")
}

/// Per-layout snapshot manager.
#[derive(Clone, Default)]
pub struct LayoutVersionService {
    layouts_dir: PathBuf,
    /// Optional event hook called on every mutation. Used by the web
    /// backend to publish SSE events; unused in CLI/TUI.
    event_hook: Option<RevisionEventHook>,
}

impl std::fmt::Debug for LayoutVersionService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LayoutVersionService")
            .field("layouts_dir", &self.layouts_dir)
            .field("event_hook", &self.event_hook.as_ref().map(|_| "<hook>"))
            .finish()
    }
}

impl LayoutVersionService {
    /// Create a new service rooted at the given layouts directory.
    #[must_use]
    pub fn new(layouts_dir: PathBuf) -> Self {
        Self {
            layouts_dir,
            event_hook: None,
        }
    }

    /// Attach an event hook for SSE publishing (web backend).
    #[allow(dead_code)] // Wired by the web backend in src/web/routes/versions.rs.
    #[must_use]
    pub fn with_event_hook(mut self, hook: RevisionEventHook) -> Self {
        self.event_hook = Some(hook);
        self
    }

    fn emit(&self, layout_name: &str, revision: u32, kind: RevisionEventKind) {
        if let Some(hook) = &self.event_hook {
            hook(layout_name, revision, kind);
        }
    }

    /// Returns the layouts directory this service was configured with.
    #[allow(dead_code)] // Public API; reserved for TUI/Web callers adding directory-level introspection.
    #[must_use]
    pub fn layouts_dir(&self) -> &Path {
        &self.layouts_dir
    }

    /// Returns the folder for a given layout name.
    #[must_use]
    pub fn layout_dir(&self, name: &str) -> PathBuf {
        self.layouts_dir.join(name)
    }

    /// Returns the path to `current.json` for a layout.
    #[must_use]
    pub fn current_path(&self, name: &str) -> PathBuf {
        self.layout_dir(name).join(CURRENT_FILE)
    }

    /// Returns the path to `manifest.json` for a layout.
    #[must_use]
    pub fn manifest_path(&self, name: &str) -> PathBuf {
        self.layout_dir(name).join(MANIFEST_FILE)
    }

    /// Returns the `versions/` directory for a layout.
    #[must_use]
    pub fn versions_dir(&self, name: &str) -> PathBuf {
        self.layout_dir(name).join(VERSIONS_DIR)
    }

    /// Check whether a layout exists in the new folder layout.
    #[allow(dead_code)] // Public API; intended for WebUI list views.
    #[must_use]
    pub fn layout_exists(&self, name: &str) -> bool {
        self.layout_dir(name).join(CURRENT_FILE).exists()
    }

    /// Load the current (active) layout for a given name.
    ///
    /// If `current.json` is missing but versions exist, the most recent
    /// revision is promoted to `current.json` first.
    ///
    /// # Errors
    ///
    /// Returns `not_found` error (see [`crate::services::layout_versions::not_found`]) when neither the folder nor a
    /// legacy flat file exists. Returns IO/parse errors otherwise.
    pub fn load_current(&self, name: &str) -> Result<Layout> {
        let current = self.current_path(name);
        if current.exists() {
            let content = fs::read_to_string(&current)
                .with_context(|| format!("Failed to read {}", current.display()))?;
            return parser::parse_json_layout_str(&content)
                .with_context(|| format!("Failed to parse {}", current.display()));
        }

        // current.json missing — try to recover from the latest revision.
        let manifest = self.load_manifest(name)?;
        if manifest.revisions.is_empty() {
            bail!(not_found(name));
        }
        let latest = manifest
            .revisions
            .iter()
            .max_by_key(|r| r.revision)
            .ok_or_else(|| not_found(name))?;

        let snapshot = self.load_revision_snapshot(name, latest.revision)?;
        atomic_write_json(&current, &snapshot.layout, None)?;
        Ok(snapshot.layout)
    }

    /// Save the active layout to `current.json`. Does not create a revision;
    /// call [`Self::create_snapshot`] explicitly when the user wants one.
    ///
    /// # Errors
    ///
    /// Returns IO errors if the write fails.
    pub fn save_current(
        &self,
        name: &str,
        layout: &Layout,
        epoch: Option<&SelfWriteEpoch>,
    ) -> Result<()> {
        let current = self.current_path(name);
        atomic_write_json(&current, layout, epoch)
    }

    /// Load the manifest, rebuilding it from disk if missing or corrupted.
    ///
    /// # Errors
    ///
    /// Returns `not_found` error (see [`crate::services::layout_versions::not_found`]) when neither the folder nor any
    /// `versions/*.json` files exist. Returns IO/parse errors otherwise.
    pub fn load_manifest(&self, name: &str) -> Result<LayoutManifest> {
        let manifest_path = self.manifest_path(name);
        if manifest_path.exists() {
            match read_manifest(&manifest_path) {
                Ok(m) if m.layout_name == name => return Ok(m),
                Ok(_) | Err(_) => {
                    // Wrong-name or corrupt: rebuild.
                }
            }
        }
        self.rebuild_manifest_from_disk(name)
    }

    /// Force a rebuild of the manifest by scanning `versions/*.json`.
    ///
    /// Sorts files by revision id (numeric prefix). If `current.json` is
    /// present, `current_revision` is taken from a sibling `<n>.current.json`
    /// marker or — if absent — defaults to the highest revision id.
    ///
    /// Returns an empty manifest when called on a fresh layout folder that
    /// has no snapshots yet (bootstrap path).
    pub fn rebuild_manifest_from_disk(&self, name: &str) -> Result<LayoutManifest> {
        let versions_dir = self.versions_dir(name);
        let layout_dir = self.layout_dir(name);

        let mut summaries: Vec<RevisionSummary> = Vec::new();
        let mut max_rev: u32 = 0;

        if versions_dir.exists() {
            for entry in fs::read_dir(&versions_dir)
                .with_context(|| format!("Failed to read {}", versions_dir.display()))?
            {
                let entry = entry?;
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) != Some("json") {
                    continue;
                }
                let Some(rev) = parse_revision_number(path.file_name().and_then(|f| f.to_str()))
                else {
                    continue;
                };
                // Be lenient: skip unparseable files instead of bailing.
                let Ok(snapshot) = read_revision(&path) else {
                    continue;
                };
                summaries.push(RevisionSummary {
                    revision: rev,
                    created: snapshot.created,
                    label: snapshot.label,
                    note: snapshot.note,
                    author: snapshot.author,
                    filename: path
                        .file_name()
                        .and_then(|f| f.to_str())
                        .unwrap_or("")
                        .to_string(),
                });
                if rev > max_rev {
                    max_rev = rev;
                }
            }
        }
        summaries.sort_by_key(|r| r.revision);

        // Determine current_revision.
        let current_path = layout_dir.join(CURRENT_FILE);
        let current_revision = if current_path.exists() {
            max_rev
        } else if !summaries.is_empty() {
            max_rev
        } else {
            // Bootstrap: no folder contents yet. Return empty manifest.
            return Ok(LayoutManifest {
                layout_name: name.to_string(),
                next_revision: 1,
                current_revision: 0,
                revisions: Vec::new(),
            });
        };

        let next_revision = max_rev.saturating_add(1).max(1);

        let manifest = LayoutManifest {
            layout_name: name.to_string(),
            next_revision,
            current_revision,
            revisions: summaries,
        };
        // Persist the rebuilt manifest (best-effort).
        if layout_dir.exists() {
            write_manifest(&self.manifest_path(name), &manifest).ok();
        }
        Ok(manifest)
    }

    /// List all known revisions for a layout, newest first.
    ///
    /// # Errors
    ///
    /// Returns `not_found` error (see [`crate::services::layout_versions::not_found`]) if the layout does not exist.
    pub fn list(&self, name: &str) -> Result<Vec<RevisionSummary>> {
        let mut manifest = self.load_manifest(name)?;
        manifest.revisions.sort_by_key(|r| std::cmp::Reverse(r.revision));
        Ok(manifest.revisions)
    }

    /// Load a single revision snapshot (full body).
    ///
    /// # Errors
    ///
    /// Returns `revision_not_found` error (see [`crate::services::layout_versions::revision_not_found`]) if the revision id is unknown.
    pub fn get(&self, name: &str, revision: u32) -> Result<LayoutRevision> {
        let manifest = self.load_manifest(name)?;
        if manifest.find(revision).is_none() {
            bail!(revision_not_found(revision, name));
        }
        self.load_revision_snapshot(name, revision)
    }

    fn load_revision_snapshot(&self, name: &str, revision: u32) -> Result<LayoutRevision> {
        let path = self.versions_dir(name).join(format!("{revision}.json"));
        if path.exists() {
            return read_revision(&path);
        }
        // Filename may include a label slug, e.g. "3-pre-rgb.json".
        let prefix = format!("{revision}-");
        let dir = self.versions_dir(name);
        if dir.exists() {
            for entry in fs::read_dir(&dir)? {
                let entry = entry?;
                let fname = entry.file_name();
                if let Some(name_os) = fname.to_str() {
                    if name_os.starts_with(&prefix)
                        && std::path::Path::new(name_os)
                            .extension()
                            .is_some_and(|ext| ext.eq_ignore_ascii_case("json"))
                    {
                        return read_revision(&entry.path());
                    }
                }
            }
        }
        bail!(revision_not_found(revision, name));
    }

    /// Create a new revision from the given layout.
    ///
    /// Allocates the next revision id, writes the snapshot atomically, and
    /// appends a summary to the manifest. Returns the created summary.
    ///
    /// On bootstrap (when `current.json` does not exist yet), the new
    /// revision becomes the active one (`current_revision` is updated).
    ///
    /// # Errors
    ///
    /// Returns IO/parse errors on failure.
    pub fn create_snapshot(
        &self,
        name: &str,
        layout: &Layout,
        label: Option<&str>,
        note: Option<&str>,
        epoch: Option<&SelfWriteEpoch>,
    ) -> Result<RevisionSummary> {
        fs::create_dir_all(self.versions_dir(name))
            .with_context(|| format!("Failed to create {}", self.versions_dir(name).display()))?;

        let mut manifest = self.load_manifest(name)?;
        let revision = if manifest.next_revision == 0 {
            1
        } else {
            manifest.next_revision
        };
        let filename = revision_filename(revision, label);

        // Resolve filename collisions by appending a short random suffix.
        let final_filename = unique_filename(&self.versions_dir(name), &filename);
        let path = self.versions_dir(name).join(&final_filename);

        let snapshot = LayoutRevision {
            revision,
            created: Utc::now(),
            label: label.map(String::from),
            note: note.map(String::from),
            author: layout.metadata.author.clone(),
            layout: layout.clone(),
        };
        atomic_write_revision(&path, &snapshot, epoch)?;

        let summary = RevisionSummary {
            revision,
            created: snapshot.created,
            label: snapshot.label,
            note: snapshot.note,
            author: snapshot.author,
            filename: final_filename,
        };
        manifest.revisions.push(summary.clone());
        manifest.next_revision = revision.saturating_add(1);

        // Bootstrap case: no current.json yet — promote this snapshot to active.
        if !self.current_path(name).exists() {
            manifest.current_revision = revision;
        }

        write_manifest(&self.manifest_path(name), &manifest)?;
        self.emit(name, revision, RevisionEventKind::Created);
        Ok(summary)
    }

    /// Create a snapshot using the default pre-compile label.
    ///
    /// Convenience wrapper around [`Self::create_snapshot`].
    pub fn create_auto_snapshot(
        &self,
        name: &str,
        layout: &Layout,
        epoch: Option<&SelfWriteEpoch>,
    ) -> Result<RevisionSummary> {
        let label = auto_label(Utc::now());
        self.create_snapshot(name, layout, Some(&label), None, epoch)
    }

    /// Restore `current.json` to the contents of the given revision.
    ///
    /// As a safety net, the current layout is auto-snapshotted first (using
    /// a `pre-restore-<ts>` label) so the user can always undo the restore.
    ///
    /// # Errors
    ///
    /// Returns `revision_not_found` error (see [`crate::services::layout_versions::revision_not_found`]) if the revision id is unknown.
    pub fn restore(
        &self,
        name: &str,
        revision: u32,
        epoch: Option<&SelfWriteEpoch>,
    ) -> Result<()> {
        let snapshot = self.get(name, revision)?;
        // Auto-snapshot the current layout before overwriting it.
        let current = self.load_current(name)?;
        let label = format!("pre-restore-{}", Utc::now().format("%Y-%m-%dT%H:%M:%SZ"));
        self.create_snapshot(name, &current, Some(&label), None, epoch)?;

        // Write current.json with the restored body.
        self.save_current(name, &snapshot.layout, epoch)?;

        // Update current_revision in the manifest.
        let mut manifest = self.load_manifest(name)?;
        manifest.current_revision = revision;
        write_manifest(&self.manifest_path(name), &manifest)?;
        self.emit(name, revision, RevisionEventKind::Restored);
        Ok(())
    }

    /// Delete a revision. Refuses to delete the currently-active revision.
    ///
    /// # Errors
    ///
    /// Returns `cannot_delete_current` error (see [`crate::services::layout_versions::cannot_delete_current`]) when the target is the
    /// active revision. Returns `revision_not_found` error (see [`crate::services::layout_versions::revision_not_found`]) otherwise.
    pub fn delete(&self, name: &str, revision: u32) -> Result<()> {
        let mut manifest = self.load_manifest(name)?;
        if manifest.find(revision).is_none() {
            bail!(revision_not_found(revision, name));
        }
        if manifest.current_revision == revision {
            bail!(cannot_delete_current(revision, name));
        }
        let path = self.versions_dir(name).join(format!("{revision}.json"));
        if path.exists() {
            fs::remove_file(&path)
                .with_context(|| format!("Failed to delete {}", path.display()))?;
        } else {
            // Try with label suffix.
            let prefix = format!("{revision}-");
            let dir = self.versions_dir(name);
            if dir.exists() {
                for entry in fs::read_dir(&dir)? {
                    let entry = entry?;
                    let fname = entry.file_name();
                    if let Some(name_os) = fname.to_str() {
                        if name_os.starts_with(&prefix)
                            && std::path::Path::new(name_os)
                                .extension()
                                .is_some_and(|ext| ext.eq_ignore_ascii_case("json"))
                        {
                            fs::remove_file(entry.path())?;
                            break;
                        }
                    }
                }
            }
        }
        manifest.revisions.retain(|r| r.revision != revision);
        write_manifest(&self.manifest_path(name), &manifest)?;
        self.emit(name, revision, RevisionEventKind::Deleted);
        Ok(())
    }

    /// Update the label and/or note on a revision.
    ///
    /// # Errors
    ///
    /// Returns `revision_not_found` error (see [`crate::services::layout_versions::revision_not_found`]) if the revision id is unknown.
    #[allow(clippy::too_many_lines)]
    pub fn rename(
        &self,
        name: &str,
        revision: u32,
        label: Option<&str>,
        note: Option<&str>,
    ) -> Result<RevisionSummary> {
        let mut manifest = self.load_manifest(name)?;
        if manifest.find(revision).is_none() {
            bail!(revision_not_found(revision, name));
        }
        // Drop the immutable borrow so we can mutably borrow below.
        let old_filename = manifest
            .find(revision)
            .map(|r| r.filename.clone())
            .unwrap_or_default();

        let mut snapshot = self.load_revision_snapshot(name, revision)?;

        snapshot.label = label.map(String::from);
        snapshot.note = note.map(String::from);

        // If the label changed and the filename includes a slug, rename the file.
        let new_filename = revision_filename(revision, label);
        let final_filename = if new_filename != old_filename {
            let versions_dir = self.versions_dir(name);
            let final_new = unique_filename(&versions_dir, &new_filename);
            let final_new_path = versions_dir.join(&final_new);
            let old_path = versions_dir.join(&old_filename);
            if old_path.exists() {
                atomic_write_revision(&final_new_path, &snapshot, None)?;
                fs::remove_file(&old_path)?;
            } else {
                atomic_write_revision(&final_new_path, &snapshot, None)?;
            }
            final_new
        } else {
            // No rename needed but still rewrite the snapshot so label/note changes persist.
            let path = self.versions_dir(name).join(&old_filename);
            atomic_write_revision(&path, &snapshot, None)?;
            old_filename
        };

        // Update manifest in place.
        {
            let summary = manifest
                .find_mut(revision)
                .expect("checked above");
            summary.label.clone_from(&snapshot.label);
            summary.note.clone_from(&snapshot.note);
            summary.filename.clone_from(&final_filename);
        }
        write_manifest(&self.manifest_path(name), &manifest)?;
        let summary = manifest.find(revision).expect("present");
        let result = RevisionSummary {
            revision,
            created: summary.created,
            label: summary.label.clone(),
            note: summary.note.clone(),
            author: summary.author.clone(),
            filename: final_filename,
        };
        self.emit(name, revision, RevisionEventKind::Renamed);
        Ok(result)
    }

    /// Compute a diff between two revisions.
    pub fn diff(&self, name: &str, from_rev: u32, to_rev: u32) -> Result<LayoutDiff> {
        let from = self.get(name, from_rev)?;
        let to = self.get(name, to_rev)?;
        Ok(compute_diff(&from.layout, &to.layout, from_rev, to_rev))
    }

    /// Delete the entire layout folder (current + manifest + all revisions).
    ///
    /// Used when the user deletes a layout via the editor.
    #[allow(dead_code)] // Wired into the layout-delete UI in a follow-up bead.
    pub fn delete_layout_folder(&self, name: &str) -> Result<()> {
        let dir = self.layout_dir(name);
        if dir.exists() {
            fs::remove_dir_all(&dir)
                .with_context(|| format!("Failed to delete layout folder {}", dir.display()))?;
        }
        Ok(())
    }

    /// Migrate a legacy flat file at `legacy_path` into the new folder layout.
    ///
    /// The legacy file is moved to `<layouts>/<name>/current.json`, an initial
    /// revision `1.json` is created, and a manifest is written. Idempotent:
    /// if the folder layout already exists, returns `false` without changes.
    ///
    /// # Errors
    ///
    /// Returns parse/IO errors on failure.
    pub fn migrate_legacy_path(&self, legacy_path: &Path) -> Result<bool> {
        let stem = legacy_path
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| {
                anyhow::anyhow!("Cannot derive layout name from {}", legacy_path.display())
            })?;

        let dir = self.layout_dir(stem);
        if dir.exists() && dir.join(CURRENT_FILE).exists() {
            return Ok(false);
        }

        let layout = crate::parser::parse_json_layout(legacy_path)
            .with_context(|| format!("Failed to parse legacy layout {}", legacy_path.display()))?;

        fs::create_dir_all(&dir)
            .with_context(|| format!("Failed to create layout dir {}", dir.display()))?;
        fs::create_dir_all(self.versions_dir(stem))
            .with_context(|| format!("Failed to create versions dir for '{stem}'"))?;

        // Move legacy -> current.json (atomic write + remove legacy).
        atomic_write_json(&dir.join(CURRENT_FILE), &layout, None)?;
        let _ = fs::remove_file(legacy_path);

        // Create revision 1.
        let snapshot = LayoutRevision {
            revision: 1,
            created: Utc::now(),
            label: Some("initial".to_string()),
            note: Some("Migrated from legacy flat-file layout.".to_string()),
            author: layout.metadata.author.clone(),
            layout,
        };
        atomic_write_revision(&self.versions_dir(stem).join("1.json"), &snapshot, None)?;

        let manifest = LayoutManifest {
            layout_name: stem.to_string(),
            next_revision: 2,
            current_revision: 1,
            revisions: vec![RevisionSummary {
                revision: 1,
                created: snapshot.created,
                label: snapshot.label,
                note: snapshot.note,
                author: snapshot.author,
                filename: "1.json".to_string(),
            }],
        };
        write_manifest(&self.manifest_path(stem), &manifest)?;
        Ok(true)
    }

    /// Returns a service rooted at the parent directory of the given path.
    ///
    /// Convenience for callers that already have a layout file path and want
    /// to run version operations on its containing layouts directory.
    #[must_use]
    pub fn from_layout_path(layout_path: &Path) -> Option<Self> {
        let parent = layout_path.parent()?;
        Some(Self::new(parent.to_path_buf()))
    }
}

/// Write JSON atomically: write to `<path>.tmp` then rename to `<path>`.
///
/// If `epoch` is provided, the file's mtime is bumped *before* the write so
/// the hot-reload watcher can recognize the change as a self-write.
pub fn atomic_write_json(
    path: &Path,
    layout: &Layout,
    epoch: Option<&SelfWriteEpoch>,
) -> Result<()> {
    if let Some(e) = epoch {
        mark_self_write(e);
    }
    let content = serde_json::to_string_pretty(layout)
        .with_context(|| format!("Failed to serialize layout to JSON for {}", path.display()))?;
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, &content)
        .with_context(|| format!("Failed to write temp file {}", tmp.display()))?;
    fs::rename(&tmp, path)
        .with_context(|| format!("Failed to rename {} to {}", tmp.display(), path.display()))?;
    Ok(())
}

fn atomic_write_revision(
    path: &Path,
    revision: &LayoutRevision,
    epoch: Option<&SelfWriteEpoch>,
) -> Result<()> {
    if let Some(e) = epoch {
        mark_self_write(e);
    }
    let content = serde_json::to_string_pretty(revision)
        .with_context(|| format!("Failed to serialize revision for {}", path.display()))?;
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, &content)
        .with_context(|| format!("Failed to write temp file {}", tmp.display()))?;
    fs::rename(&tmp, path).with_context(|| {
        format!(
            "Failed to rename {} to {}",
            tmp.display(),
            path.display()
        )
    })?;
    Ok(())
}

fn write_manifest(path: &Path, manifest: &LayoutManifest) -> Result<()> {
    let content = serde_json::to_string_pretty(manifest)
        .with_context(|| format!("Failed to serialize manifest for {}", path.display()))?;
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, &content)?;
    fs::rename(&tmp, path)?;
    Ok(())
}

fn read_manifest(path: &Path) -> Result<LayoutManifest> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read manifest {}", path.display()))?;
    serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse manifest {}", path.display()))
}

fn read_revision(path: &Path) -> Result<LayoutRevision> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read snapshot {}", path.display()))?;
    serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse snapshot {}", path.display()))
}

/// Resolve filename collisions by appending a short random suffix.
///
/// Example: `3.json` already exists -> `3-a1b2.json`.
fn unique_filename(dir: &Path, desired: &str) -> String {
    let candidate = dir.join(desired);
    if !candidate.exists() {
        return desired.to_string();
    }
    let stem = desired.trim_end_matches(".json");
    let suffix = Uuid::new_v4().simple().to_string()[..4].to_string();
    format!("{stem}-{suffix}.json")
}

/// Parse the leading numeric revision id from a snapshot filename.
///
/// Returns `None` for non-conforming names.
fn parse_revision_number(filename: Option<&str>) -> Option<u32> {
    let name = filename?;
    let trimmed = name.strip_suffix(".json")?;
    let numeric: String = trimmed.chars().take_while(char::is_ascii_digit).collect();
    if numeric.is_empty() {
        None
    } else {
        numeric.parse().ok()
    }
}

#[cfg(test)]
mod tests;
