//! Layout diff data structures.
//!
//! Diffs are computed between two `Layout` snapshots. The shape is stable
//! enough to round-trip via JSON, so both the TUI and the WebUI can render
//! the same diff.

use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::models::Layer;

/// Result of comparing two layouts revision-to-revision.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LayoutDiff {
    /// Revision id of the "before" side.
    pub from_revision: u32,
    /// Revision id of the "after" side.
    pub to_revision: u32,
    /// High-level change counts.
    pub summary: DiffSummary,
    /// Per-layer changes.
    pub layer_changes: Vec<LayerDiff>,
    /// Setting-level changes (curated set of paths).
    pub setting_changes: Vec<SettingDiff>,
}

/// High-level counts shown above the diff list.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[allow(clippy::struct_excessive_bools)] // Summary flags are intentionally flat for the UI/JSON output.
pub struct DiffSummary {
    /// Number of layers present only on the "to" side.
    pub layers_added: u32,
    /// Number of layers present only on the "from" side.
    pub layers_removed: u32,
    /// Total number of keycode changes across all layers.
    pub keys_changed: u32,
    /// True if any RGB-related setting differs.
    pub rgb_changed: bool,
    /// True if combo settings or combo count differs.
    pub combos_changed: bool,
    /// True if tap dance definitions differ.
    pub tap_dances_changed: bool,
    /// True if metadata (name, description, author, keyboard, keymap_name) differs.
    pub metadata_changed: bool,
}

/// Per-layer change.
///
/// `Added` / `Removed` are emitted when a layer is present in only one side.
/// `KeysChanged` is emitted when a layer exists on both sides but at least
/// one keycode differs. `Renamed` is emitted when the layer name changed but
/// no keycodes did.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum LayerDiff {
    /// Layer present only on the "to" side.
    Added {
        /// Index in the "to" layers array.
        index: usize,
        /// Full layer body.
        layer: Layer,
    },
    /// Layer present only on the "from" side.
    Removed {
        /// Index in the "from" layers array.
        index: usize,
        /// Layer name as it appeared on the "from" side.
        name: String,
    },
    /// Layer present on both sides with at least one keycode change.
    KeysChanged {
        /// Index in the "to" layers array.
        index: usize,
        /// Layer name (taken from the "to" side).
        name: String,
        /// Per-key changes.
        changes: Vec<KeyChange>,
    },
    /// Layer renamed but no keycodes changed.
    Renamed {
        /// Index in the "to" layers array.
        index: usize,
        /// Old name.
        from: String,
        /// New name.
        to: String,
    },
}

/// A single keycode change within a layer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyChange {
    /// Visual row (0-based).
    pub row: u8,
    /// Visual column (0-based).
    pub col: u8,
    /// Serialized keycode from the "from" layout.
    pub from: String,
    /// Serialized keycode from the "to" layout.
    pub to: String,
}

/// A non-layer setting change, addressed by dotted path.
///
/// Only emitted when the value differs between sides. Path examples:
/// - `"metadata.name"`
/// - `"rgb_enabled"`
/// - `"rgb_brightness"`
/// - `"combo_settings.enabled"`
/// - `"tap_dances_count"`
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SettingDiff {
    /// Dotted path to the setting (e.g., `"metadata.name"`).
    pub path: String,
    /// Serialized "before" value.
    pub from: String,
    /// Serialized "after" value.
    pub to: String,
}

/// Compute a diff between two layouts.
///
/// Layer matching uses the layer `id` field (stable across renames/reorders).
/// Settings are compared field-by-field with a curated set of paths.
#[must_use]
pub fn compute_diff(
    from: &crate::models::Layout,
    to: &crate::models::Layout,
    from_revision: u32,
    to_revision: u32,
) -> LayoutDiff {
    let mut summary = DiffSummary::default();
    let mut layer_changes = Vec::new();

    // Index layers by stable id.
    let from_layers: HashMap<&str, &Layer> = from.layers.iter().map(|l| (l.id.as_str(), l)).collect();

    // Walk the "to" side first so additions come first in the output.
    let mut seen_ids: HashSet<&str> = HashSet::new();
    for (i, layer) in to.layers.iter().enumerate() {
        seen_ids.insert(layer.id.as_str());
        match from_layers.get(layer.id.as_str()) {
            None => {
                summary.layers_added += 1;
                layer_changes.push(LayerDiff::Added {
                    index: i,
                    layer: layer.clone(),
                });
            }
            Some(from_layer) => {
                if from_layer.name != layer.name {
                    layer_changes.push(LayerDiff::Renamed {
                        index: i,
                        from: from_layer.name.clone(),
                        to: layer.name.clone(),
                    });
                }
                let key_changes = diff_keys(from_layer, layer);
                if !key_changes.is_empty() {
                    summary.keys_changed += key_changes.len() as u32;
                    layer_changes.push(LayerDiff::KeysChanged {
                        index: i,
                        name: layer.name.clone(),
                        changes: key_changes,
                    });
                }
            }
        }
    }

    // Removals: from-side layers not present in "to".
    for (i, layer) in from.layers.iter().enumerate() {
        if !seen_ids.contains(layer.id.as_str()) {
            summary.layers_removed += 1;
            layer_changes.push(LayerDiff::Removed {
                index: i,
                name: layer.name.clone(),
            });
        }
    }

    // Settings: walk a curated set of paths.
    let mut setting_changes = Vec::new();
    add_if_changed(&mut setting_changes, "metadata.name", &from.metadata.name, &to.metadata.name);
    add_if_changed(
        &mut setting_changes,
        "metadata.description",
        &from.metadata.description,
        &to.metadata.description,
    );
    add_if_changed(
        &mut setting_changes,
        "metadata.author",
        &from.metadata.author,
        &to.metadata.author,
    );
    add_if_changed(
        &mut setting_changes,
        "metadata.keyboard",
        from.metadata.keyboard.as_deref().unwrap_or(""),
        to.metadata.keyboard.as_deref().unwrap_or(""),
    );
    add_if_changed(
        &mut setting_changes,
        "metadata.keymap_name",
        from.metadata.keymap_name.as_deref().unwrap_or(""),
        to.metadata.keymap_name.as_deref().unwrap_or(""),
    );
    summary.metadata_changed = setting_changes
        .iter()
        .any(|d| d.path.starts_with("metadata.") && d.from != d.to);

    add_if_changed(
        &mut setting_changes,
        "rgb_enabled",
        &from.rgb_enabled.to_string(),
        &to.rgb_enabled.to_string(),
    );
    add_if_changed(
        &mut setting_changes,
        "rgb_brightness",
        &from.rgb_brightness.as_percent().to_string(),
        &to.rgb_brightness.as_percent().to_string(),
    );
    add_if_changed(
        &mut setting_changes,
        "rgb_saturation",
        &from.rgb_saturation.as_percent().to_string(),
        &to.rgb_saturation.as_percent().to_string(),
    );
    add_if_changed(
        &mut setting_changes,
        "rgb_matrix_default_speed",
        &from.rgb_matrix_default_speed.to_string(),
        &to.rgb_matrix_default_speed.to_string(),
    );

    add_if_changed(
        &mut setting_changes,
        "combo_settings.enabled",
        &from.combo_settings.enabled.to_string(),
        &to.combo_settings.enabled.to_string(),
    );
    if from.combo_settings != to.combo_settings {
        summary.combos_changed = true;
    }
    add_if_changed(
        &mut setting_changes,
        "combo_settings.combos_count",
        &from.combo_settings.combos.len().to_string(),
        &to.combo_settings.combos.len().to_string(),
    );

    add_if_changed(
        &mut setting_changes,
        "tap_dances_count",
        &from.tap_dances.len().to_string(),
        &to.tap_dances.len().to_string(),
    );
    if from.tap_dances != to.tap_dances {
        summary.tap_dances_changed = true;
    }

    summary.rgb_changed = setting_changes
        .iter()
        .any(|d| d.path.starts_with("rgb_") && d.from != d.to);

    LayoutDiff {
        from_revision,
        to_revision,
        summary,
        layer_changes,
        setting_changes,
    }
}

fn diff_keys(from: &Layer, to: &Layer) -> Vec<KeyChange> {
    let mut changes = Vec::new();
    // Build a (row,col) -> keycode lookup for "from".
    let from_keys: HashMap<(u8, u8), &str> = from
        .keys
        .iter()
        .map(|k| ((k.position.row, k.position.col), k.keycode.as_str()))
        .collect();
    let mut seen = HashSet::new();
    for key in &to.keys {
        let pos = (key.position.row, key.position.col);
        seen.insert(pos);
        let from_code = from_keys.get(&pos).copied().unwrap_or("");
        if from_code != key.keycode.as_str() {
            changes.push(KeyChange {
                row: key.position.row,
                col: key.position.col,
                from: from_code.to_string(),
                to: key.keycode.clone(),
            });
        }
    }
    // Removals on "from" side.
    for key in &from.keys {
        let pos = (key.position.row, key.position.col);
        if !seen.contains(&pos) {
            changes.push(KeyChange {
                row: key.position.row,
                col: key.position.col,
                from: key.keycode.clone(),
                to: String::new(),
            });
        }
    }
    changes
}

fn add_if_changed(changes: &mut Vec<SettingDiff>, path: &str, from: &str, to: &str) {
    if from != to {
        changes.push(SettingDiff {
            path: path.to_string(),
            from: from.to_string(),
            to: to.to_string(),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{KeyDefinition, Layer, Layout, Position, RgbColor};

    fn make_layout() -> Layout {
        let mut layout = Layout::new("diff_test").unwrap();
        let mut layer = Layer::new(0, "Base", RgbColor::new(0, 0, 0)).unwrap();
        layer.keys.push(KeyDefinition::new(Position::new(0, 0), "KC_A"));
        layer.keys.push(KeyDefinition::new(Position::new(0, 1), "KC_B"));
        layout.add_layer(layer).unwrap();
        layout
    }

    #[test]
    fn identical_layouts_yield_empty_diff() {
        let a = make_layout();
        let b = a.clone();
        let diff = compute_diff(&a, &b, 1, 2);
        assert_eq!(diff.summary.layers_added, 0);
        assert_eq!(diff.summary.layers_removed, 0);
        assert_eq!(diff.summary.keys_changed, 0);
        assert!(diff.layer_changes.is_empty());
        assert!(diff.setting_changes.is_empty());
    }

    #[test]
    fn keycode_change_detected() {
        let a = make_layout();
        let mut b = a.clone();
        b.layers[0].keys[0].keycode = "KC_C".to_string();
        let diff = compute_diff(&a, &b, 1, 2);
        assert_eq!(diff.summary.keys_changed, 1);
        assert!(matches!(
            &diff.layer_changes[0],
            LayerDiff::KeysChanged { changes, .. } if changes[0].from == "KC_A" && changes[0].to == "KC_C"
        ));
    }

    #[test]
    fn layer_added_detected() {
        let a = make_layout();
        let mut b = a.clone();
        let new_layer = Layer::new(1, "Lower", RgbColor::new(0, 0, 0)).unwrap();
        b.add_layer(new_layer).unwrap();
        let diff = compute_diff(&a, &b, 1, 2);
        assert_eq!(diff.summary.layers_added, 1);
        assert!(matches!(&diff.layer_changes[0], LayerDiff::Added { .. }));
    }

    #[test]
    fn layer_removed_detected() {
        let a = make_layout();
        let mut b = a.clone();
        b.layers.remove(0);
        let diff = compute_diff(&a, &b, 1, 2);
        assert_eq!(diff.summary.layers_removed, 1);
        assert!(matches!(&diff.layer_changes[0], LayerDiff::Removed { .. }));
    }

    #[test]
    fn metadata_change_detected() {
        let a = make_layout();
        let mut b = a.clone();
        b.metadata.description = "changed".to_string();
        let diff = compute_diff(&a, &b, 1, 2);
        assert!(diff.summary.metadata_changed);
        assert!(diff
            .setting_changes
            .iter()
            .any(|d| d.path == "metadata.description"));
    }

    #[test]
    fn rgb_change_detected() {
        let a = make_layout();
        let mut b = a.clone();
        b.rgb_enabled = false;
        let diff = compute_diff(&a, &b, 1, 2);
        assert!(diff.summary.rgb_changed);
        assert!(diff
            .setting_changes
            .iter()
            .any(|d| d.path == "rgb_enabled"));
    }

    #[test]
    fn diff_roundtrips_json() {
        let a = make_layout();
        let mut b = a.clone();
        b.layers[0].keys[0].keycode = "KC_X".to_string();
        b.metadata.description = "diff".to_string();
        let diff = compute_diff(&a, &b, 1, 2);
        let json = serde_json::to_string(&diff).unwrap();
        let back: LayoutDiff = serde_json::from_str(&json).unwrap();
        assert_eq!(diff, back);
    }
}
