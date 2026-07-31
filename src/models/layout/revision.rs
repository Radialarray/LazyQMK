//! Layout revision/snapshot data structures.
//!
//! A `LayoutRevision` is a stored snapshot of a `Layout` at a point in time.
//! Revisions are created manually by the user or automatically before firmware
//! generation. The active (current) revision is mirrored in `current.json` so
//! the editor can load it without parsing every snapshot.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::models::Layout;

/// A single stored snapshot of a layout.
///
/// On disk this struct is serialized to `<layouts>/<name>/versions/<rev>.json`.
/// The body always carries the full `Layout` so revisions can be inspected,
/// diffed, and restored without needing the active `current.json`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LayoutRevision {
    /// Monotonic per-layout revision id (1-based).
    pub revision: u32,
    /// When the snapshot was created (ISO 8601, UTC).
    pub created: DateTime<Utc>,
    /// Optional short user-supplied label (e.g., "pre-rgb-overhaul").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// Optional longer free-form note.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    /// Author copied from `LayoutMetadata::author` at snapshot time.
    pub author: String,
    /// Full layout body at the moment of snapshot.
    pub layout: Layout,
}

/// Lightweight summary shown in list views.
///
/// Storing summaries (not full layouts) in the manifest keeps list views fast
/// and avoids re-parsing every snapshot just to render a row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RevisionSummary {
    /// Revision id.
    pub revision: u32,
    /// When the snapshot was created.
    pub created: DateTime<Utc>,
    /// Optional short user-supplied label.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// Optional longer free-form note.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    /// Author of the snapshot.
    pub author: String,
    /// File name relative to the `versions/` directory (e.g., `"3.json"`).
    pub filename: String,
}

/// Per-layout index of revisions + pointer to the active one.
///
/// Written to `<layouts>/<name>/manifest.json`. Always rebuildable from disk
/// if missing or corrupted (recovery path in the version service).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct LayoutManifest {
    /// Layout name this manifest belongs to.
    pub layout_name: String,
    /// Next revision id to allocate (monotonic, 1-based).
    pub next_revision: u32,
    /// Which revision id is currently the active `current.json`.
    pub current_revision: u32,
    /// All known revisions, ordered as recorded (oldest first).
    pub revisions: Vec<RevisionSummary>,
}

impl LayoutManifest {
    /// Find a summary by revision id.
    #[must_use]
    pub fn find(&self, revision: u32) -> Option<&RevisionSummary> {
        self.revisions.iter().find(|r| r.revision == revision)
    }

    /// Find a summary by revision id (mutable).
    pub fn find_mut(&mut self, revision: u32) -> Option<&mut RevisionSummary> {
        self.revisions.iter_mut().find(|r| r.revision == revision)
    }
}

/// Default label used when the system auto-snapshots before compile.
#[must_use]
pub fn auto_label(ts: DateTime<Utc>) -> String {
    format!("pre-compile {}", ts.format("%Y-%m-%dT%H:%M:%SZ"))
}

/// Sanitize a user-supplied label for safe use in filenames.
///
/// - lowercase
/// - hyphens for whitespace
/// - keeps alnum + hyphens
/// - truncates to 40 chars
/// - returns `"untitled"` if empty after sanitization
#[must_use]
pub fn sanitize_label(label: &str) -> String {
    let mut out = String::with_capacity(label.len());
    let mut prev_dash = false;
    for c in label.chars() {
        let mapped = match c {
            'a'..='z' | '0'..='9' => Some(c),
            'A'..='Z' => Some(c.to_ascii_lowercase()),
            ' ' | '_' | '-' => Some('-'),
            _ => None,
        };
        if let Some(ch) = mapped {
            if ch == '-' {
                if !prev_dash && !out.is_empty() {
                    out.push('-');
                    prev_dash = true;
                }
            } else {
                out.push(ch);
                prev_dash = false;
            }
        }
        if out.len() >= 40 {
            break;
        }
    }
    let trimmed = out.trim_matches('-').to_string();
    if trimmed.is_empty() {
        "untitled".to_string()
    } else {
        trimmed
    }
}

/// Build a snapshot filename like `"3.json"` or `"3-pre-rgb-overhaul.json"`.
///
/// Returns just the file name (no directory component).
#[must_use]
pub fn revision_filename(revision: u32, label: Option<&str>) -> String {
    match label {
        Some(l) => {
            let slug = sanitize_label(l);
            format!("{revision}-{slug}.json")
        }
        None => format!("{revision}.json"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_label_lowercases_and_hyphenates() {
        assert_eq!(sanitize_label("Pre RGB Overhaul"), "pre-rgb-overhaul");
        assert_eq!(sanitize_label("hello_world"), "hello-world");
        assert_eq!(sanitize_label("MIXED case"), "mixed-case");
    }

    #[test]
    fn sanitize_label_strips_punctuation() {
        assert_eq!(sanitize_label("v1.0 (final)"), "v10-final");
        assert_eq!(sanitize_label("--weird--"), "weird");
        assert_eq!(sanitize_label("!!!"), "untitled");
        assert_eq!(sanitize_label(""), "untitled");
    }

    #[test]
    fn sanitize_label_truncates_to_40() {
        let long = "a".repeat(100);
        assert_eq!(sanitize_label(&long).len(), 40);
    }

    #[test]
    fn sanitize_label_collapses_runs_of_dashes() {
        assert_eq!(sanitize_label("foo   bar"), "foo-bar");
        assert_eq!(sanitize_label("foo___bar"), "foo-bar");
        assert_eq!(sanitize_label("a - b - c"), "a-b-c");
    }

    #[test]
    fn revision_filename_no_label() {
        assert_eq!(revision_filename(3, None), "3.json");
        assert_eq!(revision_filename(42, None), "42.json");
    }

    #[test]
    fn revision_filename_with_label() {
        assert_eq!(
            revision_filename(3, Some("Pre RGB Overhaul")),
            "3-pre-rgb-overhaul.json"
        );
        assert_eq!(
            revision_filename(7, Some("v1.0!!")),
            "7-v10.json"
        );
    }

    #[test]
    fn auto_label_is_human_readable() {
        let ts = DateTime::parse_from_rfc3339("2026-07-31T14:30:00Z")
            .unwrap()
            .with_timezone(&Utc);
        assert_eq!(auto_label(ts), "pre-compile 2026-07-31T14:30:00Z");
    }

    #[test]
    fn manifest_find_returns_summary() {
        let mut m = LayoutManifest::default();
        m.revisions.push(RevisionSummary {
            revision: 1,
            created: Utc::now(),
            label: None,
            note: None,
            author: "tester".to_string(),
            filename: "1.json".to_string(),
        });
        assert!(m.find(1).is_some());
        assert!(m.find(2).is_none());
    }

    #[test]
    fn manifest_roundtrip() {
        let m = LayoutManifest {
            layout_name: "demo".to_string(),
            next_revision: 4,
            current_revision: 3,
            revisions: vec![RevisionSummary {
                revision: 3,
                created: Utc::now(),
                label: Some("demo".to_string()),
                note: None,
                author: "a".to_string(),
                filename: "3-demo.json".to_string(),
            }],
        };
        let json = serde_json::to_string(&m).unwrap();
        let back: LayoutManifest = serde_json::from_str(&json).unwrap();
        assert_eq!(m, back);
    }
}
