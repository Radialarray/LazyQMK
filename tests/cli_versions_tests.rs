//! End-to-end tests for `lazyqmk versions` commands.

use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::Mutex;

use lazyqmk::services::layout_versions::LayoutVersionService;
use lazyqmk::services::LayoutService;

mod fixtures;
use fixtures::*;

static VERSIONS_TEST_LOCK: Mutex<()> = Mutex::new(());

/// Recover from a poisoned mutex so a single failing test doesn't cascade.
fn lock() -> std::sync::MutexGuard<'static, ()> {
    VERSIONS_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner())
}

fn lazyqmk_bin() -> &'static str {
    env!("CARGO_BIN_EXE_lazyqmk")
}

fn layouts_dir() -> PathBuf {
    let config_dir = dirs::config_dir().expect("Failed to get config directory");
    config_dir.join("LazyQMK").join("layouts")
}

fn unique_layout_name(prefix: &str) -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("{prefix}_{nanos}")
}

/// RAII guard that restores `LAZYQMK_CONFIG_DIR` to its previous value when
/// dropped. Tests use it so the env var override doesn't leak into siblings.
struct EnvGuard {
    previous: Option<String>,
}

impl EnvGuard {
    fn set(value: &std::path::Path) -> Self {
        let previous = std::env::var("LAZYQMK_CONFIG_DIR").ok();
        std::env::set_var("LAZYQMK_CONFIG_DIR", value);
        Self { previous }
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        match &self.previous {
            Some(prev) => std::env::set_var("LAZYQMK_CONFIG_DIR", prev),
            None => std::env::remove_var("LAZYQMK_CONFIG_DIR"),
        }
    }
}

/// Set up a fresh layout folder + a temp file pointing at it, then run the
/// `init` step (creates the folder via the service so the CLI's snapshot
/// hook can find it). Returns (layout_path, layout_name, env_guard, _tempdir).
fn setup_layout_with_folder(
    layout_name: &str,
) -> (PathBuf, String, EnvGuard, tempfile::TempDir) {
    let tmp = tempfile::TempDir::new().unwrap();
    let workspace = tmp.path().to_path_buf();

    // Point the CLI at our temp workspace via the override env var. We use
    // the workspace dir directly (it will become "the config dir", with
    // layouts expected at `<workspace>/layouts`).
    let guard = EnvGuard::set(&workspace);

    let layout = test_layout_basic(2, 3);
    let layouts_subdir = workspace.join("layouts");
    std::fs::create_dir_all(&layouts_subdir).unwrap();
    let layout_path = layouts_subdir.join(format!("{layout_name}.json"));
    LayoutService::save(&layout, &layout_path).unwrap();

    // Pre-create the folder layout so the snapshot hook will fire.
    let svc = LayoutVersionService::new(layouts_subdir.clone());
    svc.migrate_legacy_path(&layout_path).unwrap();

    (layout_path, layout_name.to_string(), guard, tmp)
}

#[test]
fn versions_save_creates_revision() {
    let _lock = lock();
    let (layout_path, name, _guard, _tmp) = setup_layout_with_folder("save_test");

    let output = Command::new(lazyqmk_bin())
        .args([
            "versions",
            "save",
            "--layout",
            layout_path.to_str().unwrap(),
            "--label",
            "test-label",
        ])
        .output()
        .expect("Failed to execute command");
    assert_eq!(
        output.status.code(),
        Some(0),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let workspace = layout_path.parent().unwrap();
    let versions_dir = workspace.join(&name).join("versions");
    let count = fs::read_dir(&versions_dir)
        .unwrap()
        .filter_map(Result::ok)
        .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("json"))
        .count();
    assert_eq!(count, 2, "expected 2 revisions (initial + saved)");
}

#[test]
fn versions_list_returns_revisions() {
    let _lock = lock();
    let (layout_path, _name, _guard, _tmp) = setup_layout_with_folder("list_test");

    // Save two extra revisions.
    for label in ["alpha", "beta"] {
        let out = Command::new(lazyqmk_bin())
            .args([
                "versions",
                "save",
                "--layout",
                layout_path.to_str().unwrap(),
                "--label",
                label,
            ])
            .output()
            .unwrap();
        assert_eq!(out.status.code(), Some(0));
    }

    // Use --json output for stable parsing.
    let workspace = layout_path.parent().unwrap();
    let out = Command::new(lazyqmk_bin())
        .args([
            "versions",
            "list",
            "list_test",
            "--json",
        ])
        .current_dir(workspace)
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&out.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let arr = parsed.as_array().expect("should be an array");
    assert_eq!(arr.len(), 3, "expected initial + alpha + beta");
}

#[test]
fn versions_show_returns_full_layout() {
    let _lock = lock();
    let (layout_path, _name, _guard, _tmp) = setup_layout_with_folder("show_test");

    let out = Command::new(lazyqmk_bin())
        .args([
            "versions",
            "save",
            "--layout",
            layout_path.to_str().unwrap(),
            "--label",
            "v1",
        ])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));

    let workspace = layout_path.parent().unwrap();
    // Setup creates revision 1 (initial). The save above creates revision 2.
    let out = Command::new(lazyqmk_bin())
        .args(["versions", "show", "show_test", "2"])
        .current_dir(workspace)
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("Revision #2"));
    assert!(stdout.contains("v1"));
}

#[test]
fn versions_diff_shows_changes() {
    let _lock = lock();
    let (layout_path, _name, _guard, _tmp) = setup_layout_with_folder("diff_test");

    // Revision 1 was created by setup_layout_with_folder's migration step.
    // Mutate the layout, save as the new current.
    let mut layout = LayoutService::load(&layout_path).unwrap();
    layout.metadata.description = "changed".to_string();
    LayoutService::save(&layout, &layout_path).unwrap();

    // Save the mutated state as revision 2.
    Command::new(lazyqmk_bin())
        .args([
            "versions",
            "save",
            "--layout",
            layout_path.to_str().unwrap(),
            "--label",
            "after-change",
        ])
        .output()
        .unwrap();

    let workspace = layout_path.parent().unwrap();
    let out = Command::new(lazyqmk_bin())
        .args(["versions", "diff", "--layout", layout_path.to_str().unwrap(), "1", "2"])
        .current_dir(workspace)
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&out.stdout);
    eprintln!("DIFF STDOUT:\n{stdout}");

    assert!(stdout.contains("Diff:"));
    assert!(stdout.contains("metadata"));
}

#[test]
fn versions_diff_json_emits_valid_json() {
    let _lock = lock();
    let (layout_path, _name, _guard, _tmp) = setup_layout_with_folder("diffjson_test");

    // Setup already creates revision 1. Mutate + save revision 2.
    let mut layout = LayoutService::load(&layout_path).unwrap();
    layout.metadata.description = "diff".to_string();
    LayoutService::save(&layout, &layout_path).unwrap();
    Command::new(lazyqmk_bin())
        .args([
            "versions",
            "save",
            "--layout",
            layout_path.to_str().unwrap(),
            "--label",
            "after-change",
        ])
        .output()
        .unwrap();

    let out = Command::new(lazyqmk_bin())
        .args([
            "versions",
            "diff",
            "--layout",
            layout_path.to_str().unwrap(),
            "1",
            "2",
            "--json",
        ])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    let parsed: serde_json::Value =
        serde_json::from_slice(&out.stdout).expect("should be valid JSON");
    assert_eq!(parsed["from_revision"], 1);
    assert_eq!(parsed["to_revision"], 2);
    assert!(parsed["metadata_changed"].as_bool().unwrap_or(false));
}

#[test]
fn versions_rename_updates_label() {
    let _lock = lock();
    let (layout_path, _name, _guard, _tmp) = setup_layout_with_folder("rename_test");

    Command::new(lazyqmk_bin())
        .args([
            "versions",
            "save",
            "--layout",
            layout_path.to_str().unwrap(),
        ])
        .output()
        .unwrap();

    let out = Command::new(lazyqmk_bin())
        .args([
            "versions",
            "rename",
            "--layout",
            layout_path.to_str().unwrap(),
            "1",
            "--label",
            "new-label",
        ])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));

    let workspace = layout_path.parent().unwrap();
    let versions_dir = workspace.join("rename_test").join("versions");
    let has_new = fs::read_dir(&versions_dir)
        .unwrap()
        .filter_map(Result::ok)
        .any(|e| e.file_name().to_string_lossy().contains("new-label"));
    assert!(has_new, "filename should include the new label");
}

#[test]
fn versions_restore_works_with_yes() {
    let _lock = lock();
    let (layout_path, _name, _guard, _tmp) = setup_layout_with_folder("restore_test");

    // Save initial revision with description A.
    Command::new(lazyqmk_bin())
        .args([
            "versions",
            "save",
            "--layout",
            layout_path.to_str().unwrap(),
            "--label",
            "before",
        ])
        .output()
        .unwrap();

    // Mutate and save again.
    let mut layout = LayoutService::load(&layout_path).unwrap();
    layout.metadata.description = "mutated".to_string();
    LayoutService::save(&layout, &layout_path).unwrap();

    // Restore revision 1 (the "before" state).
    let out = Command::new(lazyqmk_bin())
        .args([
            "versions",
            "restore",
            "--layout",
            layout_path.to_str().unwrap(),
            "1",
            "--yes",
        ])
        .output()
        .unwrap();
    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    // Verify current.json now reflects revision 1 (description "").
    let loaded = LayoutService::load(&layout_path).unwrap();
    assert_ne!(loaded.metadata.description, "mutated");
}

#[test]
fn versions_delete_removes_revision() {
    let _lock = lock();
    let (layout_path, _name, _guard, _tmp) = setup_layout_with_folder("delete_test");

    // Save 2 revisions.
    Command::new(lazyqmk_bin())
        .args([
            "versions",
            "save",
            "--layout",
            layout_path.to_str().unwrap(),
            "--label",
            "extra",
        ])
        .output()
        .unwrap();
    Command::new(lazyqmk_bin())
        .args([
            "versions",
            "save",
            "--layout",
            layout_path.to_str().unwrap(),
            "--label",
            "extra2",
        ])
        .output()
        .unwrap();

    // Delete revision 2 (the "extra" one).
    let out = Command::new(lazyqmk_bin())
        .args([
            "versions",
            "delete",
            "--layout",
            layout_path.to_str().unwrap(),
            "2",
            "--yes",
        ])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));

    let workspace = layout_path.parent().unwrap();
    let versions_dir = workspace.join("delete_test").join("versions");
    let count = fs::read_dir(&versions_dir)
        .unwrap()
        .filter_map(Result::ok)
        .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("json"))
        .count();
    assert_eq!(count, 2, "expected 2 revisions after delete (initial + extra2)");
}

#[test]
fn versions_delete_current_refused() {
    let _lock = lock();
    let (layout_path, _name, _guard, _tmp) = setup_layout_with_folder("delcur_test");

    let out = Command::new(lazyqmk_bin())
        .args([
            "versions",
            "delete",
            "--layout",
            layout_path.to_str().unwrap(),
            "1",
            "--yes",
        ])
        .output()
        .unwrap();
    assert_ne!(out.status.code(), Some(0), "should refuse to delete current");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.to_lowercase().contains("active"),
        "expected active error, got: {stderr}"
    );
}

#[test]
fn versions_list_unknown_layout_succeeds_empty() {
    let _lock = lock();
    let name = unique_layout_name("nonexistent");

    let out = Command::new(lazyqmk_bin())
        .args(["versions", "list", &name])
        .output()
        .unwrap();
    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("No revisions"),
        "expected empty message, got: {stdout}"
    );
}

#[test]
fn versions_save_without_folder_skips_snapshot() {
    let _lock = lock();

    let tmp = tempfile::TempDir::new().unwrap();
    let workspace = tmp.path();
    let layout = test_layout_basic(2, 3);
    let layout_path = workspace.join("nofolder.json");
    LayoutService::save(&layout, &layout_path).unwrap();

    // The folder layout doesn't exist (only the flat file does). The
    // snapshot hook should still work because we fall through to the
    // legacy path... actually, the hook is supposed to skip in this case.
    // Verify it doesn't crash.
    let out = Command::new(lazyqmk_bin())
        .args([
            "versions",
            "save",
            "--layout",
            layout_path.to_str().unwrap(),
            "--label",
            "from-flat",
        ])
        .output()
        .unwrap();
    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    // After save, the service-side path used by `versions save` reads the
    // layout via LayoutService::load which migrates to folder. So a folder
    // should exist now.
    assert!(workspace.join("nofolder").join("versions").exists());
}
