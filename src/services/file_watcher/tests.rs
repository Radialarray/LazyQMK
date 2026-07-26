//! Unit tests for the shared file watcher service.

use super::*;
use std::fs;
use std::thread;
use std::time::Duration;
use tempfile::TempDir;

#[test]
fn test_self_write_epoch_round_trip() {
    let epoch = new_epoch();
    assert!(!should_ignore(&epoch, SystemTime::now()));
    mark_self_write(&epoch);
    // Immediately after marking, the epoch should suppress the
    // event with the same mtime as "now".
    let now = SystemTime::now();
    assert!(should_ignore(&epoch, now));
}

#[test]
fn test_self_write_tolerance_window() {
    let epoch = new_epoch();
    mark_self_write(&epoch);
    // Event well outside the tolerance window must NOT be ignored.
    let far_future = SystemTime::now() + Duration::from_secs(60);
    assert!(!should_ignore(&epoch, far_future));
}

#[test]
fn test_zero_epoch_never_ignores() {
    let epoch = new_epoch();
    // Fresh epoch has value 0 -> no suppression.
    assert!(!should_ignore(&epoch, SystemTime::now()));
    assert!(!should_ignore(&epoch, UNIX_EPOCH));
}

#[test]
fn test_watch_fires_on_external_write() {
    let dir = TempDir::new().expect("create temp dir");
    let file = dir.path().join("test.json");
    fs::write(&file, "{}").expect("seed file");

    let epoch = new_epoch();
    let handle = watch(&file, std::sync::Arc::clone(&epoch)).expect("watch file");

    // Write from another process/thread (this thread).
    thread::sleep(Duration::from_millis(50));
    fs::write(&file, r#"{"changed": true}"#).expect("rewrite file");

    // The debouncer coalesces events; poll for up to 2 seconds.
    let mut got_change = false;
    for _ in 0..20 {
        thread::sleep(Duration::from_millis(100));
        for ev in handle.drain() {
            if matches!(ev, FileEvent::Changed { .. }) {
                got_change = true;
            }
        }
        if got_change {
            break;
        }
    }
    assert!(got_change, "expected at least one Changed event");
}

#[test]
fn test_watch_ignores_self_write() {
    let dir = TempDir::new().expect("create temp dir");
    let file = dir.path().join("self.json");
    fs::write(&file, "{}").expect("seed file");

    let epoch = new_epoch();
    let handle = watch(&file, std::sync::Arc::clone(&epoch)).expect("watch file");

    // Mark as self-write and then write within the tolerance.
    mark_self_write(&epoch);
    fs::write(&file, r#"{"ours": true}"#).expect("self-write");

    // Wait longer than the tolerance window + debounce, then verify
    // no Changed event was forwarded.
    thread::sleep(Duration::from_millis(900));
    let drained = handle.drain();
    assert!(
        drained.is_empty(),
        "self-write echo leaked: {drained:?}"
    );
}

#[test]
fn test_is_layout_file() {
    assert!(is_layout_file(Path::new("foo.json")));
    assert!(is_layout_file(Path::new("FOO.JSON")));
    assert!(!is_layout_file(Path::new("foo.md")));
    assert!(!is_layout_file(Path::new("foo")));
}
