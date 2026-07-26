//! Workspace-level file watcher for the web backend.
//!
//! Spins up a background `notify` watcher on the workspace root and
//! forwards every `.json` modification as a [`LayoutEvent`] to the
//! supplied Tokio broadcast sender. Self-writes (i.e. our own
//! `LayoutService::save_with_epoch` calls) are suppressed via the
//! shared [`SelfWriteEpoch`].

use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, RecvTimeoutError};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use notify::{EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use notify_debouncer_full::{new_debouncer, DebounceEventResult, Debouncer, FileIdMap};
use tokio::sync::broadcast;

use crate::services::file_watcher::{self, is_layout_file, SelfWriteEpoch, DEBOUNCE_WINDOW};
use crate::web::events::LayoutEvent;

/// Worker poll interval while idle. Low enough to make the
/// cancellation flag responsive, high enough to keep CPU usage
/// negligible.
const WORKER_POLL_INTERVAL: Duration = Duration::from_millis(200);

/// Owning handle for the workspace watcher.
///
/// The watcher background thread stays alive as long as this struct
/// exists, and is dropped when the AppState that owns it goes away.
/// The worker publishes to a cloned broadcast sender, so it works
/// with any number of subscribers; when the channel has no senders,
/// the worker exits naturally.
pub struct WorkspaceWatcher {
    /// The notify debouncer — its drop stops the underlying watcher.
    _debouncer: Debouncer<RecommendedWatcher, FileIdMap>,
    /// Aborts the worker thread when the watcher is dropped.
    cancel: Arc<AtomicBool>,
}

impl Drop for WorkspaceWatcher {
    fn drop(&mut self) {
        self.cancel.store(true, Ordering::Release);
    }
}

/// Starts a background watcher for the workspace and pushes
/// [`LayoutEvent`] messages into `event_tx` whenever a layout file
/// changes.
///
/// Returns a [`WorkspaceWatcher`] which keeps the underlying notify
/// watcher alive. The caller is expected to attach the handle to
/// long-lived state (e.g. `AppState`) so it gets dropped at
/// shutdown. The companion worker thread exits when the broadcast
/// sender has no more receivers OR when the watcher is dropped.
///
/// # Errors
///
/// Returns an error if the underlying `notify` watcher cannot be
/// constructed.
pub fn spawn_workspace_watcher(
    workspace: &Path,
    self_write_epoch: SelfWriteEpoch,
    event_tx: broadcast::Sender<LayoutEvent>,
) -> Result<WorkspaceWatcher> {
    let (tx, rx) = channel::<notify_debouncer_full::DebouncedEvent>();

    let mut debouncer = new_debouncer(
        DEBOUNCE_WINDOW,
        None,
        move |res: DebounceEventResult| {
            if let Ok(events) = res {
                for ev in events {
                    let _ = tx.send(ev);
                }
            }
        },
    )
    .context("Failed to construct directory watcher")?;

    debouncer
        .watcher()
        .watch(workspace, RecursiveMode::NonRecursive)
        .with_context(|| format!("Failed to watch directory {}", workspace.display()))?;

    // Drain any startup noise.
    while rx.try_recv().is_ok() {}

    let cancel = Arc::new(AtomicBool::new(false));
    let cancel_thread = Arc::clone(&cancel);

    // Background thread that translates notify events to broadcast
    // events. When the cancellation flag is set (via the watcher's
    // Drop) or the broadcast channel has no senders, the loop exits.
    thread::Builder::new()
        .name("lazyqmk-watcher".into())
        .spawn(move || loop {
            if cancel_thread.load(Ordering::Acquire) {
                break;
            }
            match rx.recv_timeout(WORKER_POLL_INTERVAL) {
                Ok(ev) => {
                    let layout_paths: Vec<_> =
                        ev.event.paths.iter().filter(|p| is_layout_file(p)).collect();
                    for path in layout_paths {
                        let filename = match path.file_name().and_then(|s| s.to_str()) {
                            Some(f) => f.to_string(),
                            None => continue,
                        };
                        let event = match ev.event.kind {
                            EventKind::Remove(_) => LayoutEvent::Removed { filename },
                            _ => {
                                let mtime = std::fs::metadata(path)
                                    .and_then(|m| m.modified())
                                    .unwrap_or_else(|_| SystemTime::now());
                                if file_watcher::should_ignore(&self_write_epoch, mtime) {
                                    continue;
                                }
                                let mtime_secs = mtime
                                    .duration_since(UNIX_EPOCH)
                                    .map(|d| d.as_secs())
                                    .unwrap_or(0);
                                LayoutEvent::Changed {
                                    filename,
                                    mtime: mtime_secs,
                                }
                            }
                        };
                        if event_tx.send(event).is_err() {
                            // No subscribers — exit.
                            return;
                        }
                    }
                }
                Err(RecvTimeoutError::Timeout) => {}
                Err(RecvTimeoutError::Disconnected) => break,
            }
        })
        .context("Failed to spawn watcher thread")?;

    Ok(WorkspaceWatcher {
        _debouncer: debouncer,
        cancel,
    })
}
