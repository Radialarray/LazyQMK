//! Tests for `LayoutVersionService`.

use super::*;
use crate::models::{KeyDefinition, Layer, Position, RgbColor};
use tempfile::TempDir;

fn make_layout(name: &str) -> Layout {
    let mut layout = Layout::new(name).unwrap();
    layout.metadata.author = "tester".to_string();
    let mut layer = Layer::new(0, "Base", RgbColor::new(0, 0, 0)).unwrap();
    layer
        .keys
        .push(KeyDefinition::new(Position::new(0, 0), "KC_A"));
    layout.add_layer(layer).unwrap();
    layout
}

fn service_in_tmp() -> (TempDir, LayoutVersionService) {
    let tmp = TempDir::new().unwrap();
    let svc = LayoutVersionService::new(tmp.path().to_path_buf());
    (tmp, svc)
}

#[test]
fn create_snapshot_writes_files_and_updates_manifest() {
    let (_tmp, svc) = service_in_tmp();
    let layout = make_layout("test");
    let summary = svc
        .create_snapshot("test", &layout, Some("first"), None, None)
        .unwrap();
    assert_eq!(summary.revision, 1);
    assert_eq!(summary.filename, "1-first.json");
    assert!(svc.versions_dir("test").join("1-first.json").exists());
    let manifest = svc.load_manifest("test").unwrap();
    assert_eq!(manifest.next_revision, 2);
    assert_eq!(manifest.revisions.len(), 1);
}

#[test]
fn list_returns_revisions() {
    let (_tmp, svc) = service_in_tmp();
    let layout = make_layout("test");
    svc.create_snapshot("test", &layout, None, None, None).unwrap();
    svc.create_snapshot("test", &layout, Some("second"), None, None)
        .unwrap();
    let listed = svc.list("test").unwrap();
    assert_eq!(listed.len(), 2);
    assert_eq!(listed[0].revision, 2);
    assert_eq!(listed[1].revision, 1);
}

#[test]
fn get_returns_full_snapshot() {
    let (_tmp, svc) = service_in_tmp();
    let layout = make_layout("test");
    svc.create_snapshot("test", &layout, Some("v1"), None, None)
        .unwrap();
    let snap = svc.get("test", 1).unwrap();
    assert_eq!(snap.revision, 1);
    assert_eq!(snap.label.as_deref(), Some("v1"));
    assert_eq!(snap.layout.metadata.name, "test");
}

#[test]
fn get_unknown_revision_errors() {
    let (_tmp, svc) = service_in_tmp();
    let layout = make_layout("test");
    svc.create_snapshot("test", &layout, None, None, None).unwrap();
    let err = svc.get("test", 99).unwrap_err();
    assert!(err.to_string().contains("not found"));
}

#[test]
fn delete_removes_revision() {
    let (_tmp, svc) = service_in_tmp();
    let layout = make_layout("test");
    svc.create_snapshot("test", &layout, None, None, None).unwrap();
    svc.create_snapshot("test", &layout, None, None, None).unwrap();
    svc.delete("test", 1).unwrap();
    assert_eq!(svc.list("test").unwrap().len(), 1);
    assert!(!svc.versions_dir("test").join("1.json").exists());
}

#[test]
fn delete_current_refused() {
    let (_tmp, svc) = service_in_tmp();
    let layout = make_layout("test");
    svc.create_snapshot("test", &layout, None, None, None).unwrap();
    let err = svc.delete("test", 1).unwrap_err();
    assert!(err.to_string().contains("active"));
}

#[test]
fn restore_writes_current_and_auto_snapshots_old() {
    let (_tmp, svc) = service_in_tmp();
    let mut layout = make_layout("test");
    svc.create_snapshot("test", &layout, Some("v1"), None, None)
        .unwrap();

    // Mutate the layout and save it as current.
    layout.layers[0].keys[0].keycode = "KC_Z".to_string();
    svc.save_current("test", &layout, None).unwrap();

    // Restore v1 — should create a pre-restore snapshot first, then restore.
    svc.restore("test", 1, None).unwrap();

    let restored = svc.load_current("test").unwrap();
    assert_eq!(restored.layers[0].keys[0].keycode, "KC_A");
    let listed = svc.list("test").unwrap();
    // We now have: v1 (1), the mutated snapshot implicitly via pre-restore (2),
    // and restore moved us back to v1.
    assert!(listed.iter().any(|r| r.revision == 2));
    let pre = svc.get("test", 2).unwrap();
    assert!(pre.label.as_deref().unwrap_or("").starts_with("pre-restore-"));
}

#[test]
fn rename_updates_label_and_filename() {
    let (_tmp, svc) = service_in_tmp();
    let layout = make_layout("test");
    svc.create_snapshot("test", &layout, None, None, None).unwrap();
    let updated = svc
        .rename("test", 1, Some("pre-rgb-overhaul"), None)
        .unwrap();
    assert_eq!(updated.filename, "1-pre-rgb-overhaul.json");
    assert!(svc.versions_dir("test").join("1-pre-rgb-overhaul.json").exists());
    assert!(!svc.versions_dir("test").join("1.json").exists());
}

#[test]
fn diff_returns_changes() {
    let (_tmp, svc) = service_in_tmp();
    let mut a = make_layout("test");
    svc.create_snapshot("test", &a, None, None, None).unwrap();
    a.layers[0].keys[0].keycode = "KC_Q".to_string();
    svc.create_snapshot("test", &a, None, None, None).unwrap();
    let diff = svc.diff("test", 1, 2).unwrap();
    assert_eq!(diff.summary.keys_changed, 1);
}

#[test]
fn migrate_legacy_creates_folder_layout() {
    let (_tmp, svc) = service_in_tmp();
    let layout = make_layout("legacy");
    let legacy_path = svc.layouts_dir.join("legacy.json");
    crate::parser::save_json_layout(&layout, &legacy_path).unwrap();

    let migrated = svc.migrate_legacy_path(&legacy_path).unwrap();
    assert!(migrated);
    assert!(svc.layout_dir("legacy").join(CURRENT_FILE).exists());
    assert!(svc.versions_dir("legacy").join("1.json").exists());
    assert!(svc.manifest_path("legacy").exists());
    assert!(!legacy_path.exists(), "legacy file should be moved");

    // Second call should be a no-op.
    let again = svc.migrate_legacy_path(&legacy_path).unwrap();
    assert!(!again);
}

#[test]
fn load_current_recovers_from_missing_current() {
    let (_tmp, svc) = service_in_tmp();
    let layout = make_layout("test");
    // Manually create a layout folder with only versions/, no current.json.
    fs::create_dir_all(svc.layout_dir("test")).unwrap();
    fs::create_dir_all(svc.versions_dir("test")).unwrap();
    svc.create_snapshot("test", &layout, None, None, None).unwrap();
    // Now delete current.json to simulate corruption.
    let _ = fs::remove_file(svc.current_path("test"));
    let restored = svc.load_current("test").unwrap();
    assert_eq!(restored.metadata.name, "test");
    assert!(svc.current_path("test").exists(), "current.json should be regenerated");
}

#[test]
fn rebuild_manifest_from_disk_handles_missing_manifest() {
    let (_tmp, svc) = service_in_tmp();
    let layout = make_layout("test");
    svc.create_snapshot("test", &layout, None, None, None).unwrap();
    // Delete manifest.
    fs::remove_file(svc.manifest_path("test")).unwrap();
    let rebuilt = svc.rebuild_manifest_from_disk("test").unwrap();
    assert_eq!(rebuilt.revisions.len(), 1);
    assert_eq!(rebuilt.next_revision, 2);
}

#[test]
fn delete_layout_folder_removes_everything() {
    let (_tmp, svc) = service_in_tmp();
    let layout = make_layout("test");
    svc.create_snapshot("test", &layout, None, None, None).unwrap();
    assert!(svc.layout_dir("test").exists());
    svc.delete_layout_folder("test").unwrap();
    assert!(!svc.layout_dir("test").exists());
}

#[test]
fn create_auto_snapshot_uses_precompile_label() {
    let (_tmp, svc) = service_in_tmp();
    let layout = make_layout("test");
    let summary = svc
        .create_auto_snapshot("test", &layout, None)
        .unwrap();
    assert!(summary
        .label
        .as_deref()
        .unwrap_or("")
        .starts_with("pre-compile "));
    assert!(summary.filename.starts_with("1-pre-compile-"));
}

#[test]
fn create_snapshot_filename_collision_resolved() {
    let (_tmp, svc) = service_in_tmp();
    let layout = make_layout("test");
    // Manually create a file with the same name as the next snapshot would use.
    fs::create_dir_all(svc.versions_dir("test")).unwrap();
    fs::write(svc.versions_dir("test").join("1-first.json"), "{}").unwrap();
    let summary = svc
        .create_snapshot("test", &layout, Some("first"), None, None)
        .unwrap();
    assert_ne!(summary.filename, "1-first.json");
    assert!(svc.versions_dir("test").join(&summary.filename).exists());
}
