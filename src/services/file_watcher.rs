//! Shared file-watcher service for hot-reload in TUI and WebUI.
//!
//! Both the TUI editor and the WebUI editor (served by the `web` binary) need
//! to know when the on-disk layout `.json` file is modified by an external
//! process — for example, a background AI agent running
//! `lazyqmk tap-dance add --layout foo.json`, a sibling TUI session, or a
//! hand-edit with `jq`/`vim`.
//!
//! This module wraps the [`notify_debouncer_full`] crate with a small,
//! process-local API that both the TUI (synchronous `mpsc` drain) and the
//! web backend (Tokio `broadcast`) can consume without depending on each
//! other.
//!
//! # Self-write suppression
//!
//! `notify` fires events for every `write`/`rename`, including the ones
//! we triggered ourselves. To avoid the "save → watcher → reload → save"
//! echo loop, callers bump [`SelfWriteEpoch`] immediately before performing
//! an atomic write, and the debouncer consults the epoch when deciding
//! whether to forward an event. See [`mark_self_write`] and
//! [`should_ignore`].

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{channel, Receiver};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use notify_debouncer_full::{new_debouncer, DebounceEventResult, Debouncer, FileIdMap};

/// Debounce window for the underlying `notify` watcher.
///
/// The TUI polls every 100 ms and the web UI serves events as soon as they
/// arrive, so a 250 ms debounce is enough to coalesce the burst of events
/// that an atomic temp-rename write produces without adding meaningful
/// latency for human-perceived "live" updates.
pub const DEBOUNCE_WINDOW: Duration = Duration::from_millis(250);

/// How long after a self-write mark we still consider an event "ours".
///
/// This is intentionally larger than `DEBOUNCE_WINDOW` to absorb clock
/// drift and filesystem timestamp granularity (some filesystems only
/// record mtime to the nearest second).
const SELF_WRITE_TOLERANCE: Duration = Duration::from_millis(500);

/// File-system event surfaced to consumers (TUI / web backend).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileEvent {
    /// File content changed on disk.
    Changed {
        /// The file that changed (always inside the watched scope).
        path: PathBuf,
        /// Best-effort mtime of the file at event time.
        mtime: SystemTime,
    },
    /// File was removed.
    Removed {
        /// The file that was removed.
        path: PathBuf,
    },
}

/// Monotonic counter used to suppress watcher events caused by our own
/// atomic writes.
///
/// `LayoutService::save` (and any other writer) calls
/// [`mark_self_write`] before the temp+rename, and the debouncer
/// inspects the counter when deciding whether to forward a `Changed`
/// event to subscribers.
pub type SelfWriteEpoch = std::sync::Arc<AtomicU64>;

/// Creates a fresh epoch counter.
#[must_use]
pub fn new_epoch() -> SelfWriteEpoch {
    std::sync::Arc::new(AtomicU64::new(0))
}

/// Records that the current process is about to write to a watched file.
///
/// Callers should invoke this **before** the atomic write so the watcher
/// sees a timestamp strictly less than (or equal to) the recorded value.
pub fn mark_self_write(epoch: &SelfWriteEpoch) {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0);
    epoch.store(nanos, Ordering::Release);
}

/// Returns `true` if an event with the given `mtime` should be ignored
/// because it was almost certainly caused by our own self-write mark.
///
/// The check tolerates up to [`SELF_WRITE_TOLERANCE`] of clock skew
/// between `mark_self_write` and the filesystem's reported mtime.
#[must_use]
pub fn should_ignore(epoch: &SelfWriteEpoch, mtime: SystemTime) -> bool {
    let marked = epoch.load(Ordering::Acquire);
    if marked == 0 {
        return false;
    }
    let event_nanos = mtime
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0);
    if event_nanos == 0 || event_nanos < marked {
        return true;
    }
    let delta = Duration::from_nanos(event_nanos - marked);
    delta <= SELF_WRITE_TOLERANCE
}

/// Owning handle for a live `notify` debouncer plus the receive side of
/// its `mpsc` event stream.
///
/// Dropping the handle stops the watcher. The [`Self::try_recv`] method
/// is non-blocking and intended to be called from the TUI's 100 ms
/// event loop.
pub struct FileWatcherHandle {
    /// Keeps the background watcher thread alive.
    _debouncer: Debouncer<RecommendedWatcher, FileIdMap>,
    /// Receive end of the debouncer's event channel.
    rx: Receiver<FileEvent>,
    /// Path the watcher is rooted on (file or directory).
    watched_path: PathBuf,
    /// Shared epoch so the debouncer can suppress self-writes.
    self_write_epoch: SelfWriteEpoch,
}

impl FileWatcherHandle {
    /// Returns the path or directory the watcher is rooted on.
    #[must_use]
    #[allow(dead_code)] // public API; useful for diagnostics and future tool use
    pub fn watched_path(&self) -> &Path {
        &self.watched_path
    }

    /// Returns a clone of the shared self-write epoch.
    #[must_use]
    #[allow(dead_code)] // public API; consumers can pass it to LayoutService::save_with_epoch
    pub fn self_write_epoch(&self) -> SelfWriteEpoch {
        std::sync::Arc::clone(&self.self_write_epoch)
    }

    /// Drains all currently buffered events, returning them in order.
    ///
    /// Used by the TUI event loop to catch up after a longer pause
    /// (e.g. blocking on a key press).
    pub fn drain(&self) -> Vec<FileEvent> {
        let mut out = Vec::new();
        while let Ok(ev) = self.rx.try_recv() {
            out.push(ev);
        }
        out
    }
}

/// Starts watching `path` for external modifications.
///
/// If `path` is a file, the watcher is rooted on its parent directory
/// and events are filtered to that file. If `path` is a directory, all
/// `.json` events inside it are forwarded.
///
/// # Errors
///
/// Returns an error if the path does not exist, the parent directory
/// cannot be resolved, or the underlying `notify` watcher fails to
/// register.
pub fn watch(path: &Path, self_write_epoch: SelfWriteEpoch) -> Result<FileWatcherHandle> {
    let (tx, rx) = channel::<FileEvent>();

    let watched_path = path.to_path_buf();
    let watch_target: PathBuf = if path.is_file() {
        path.parent()
            .context("Cannot watch a file with no parent directory")?
            .to_path_buf()
    } else {
        path.to_path_buf()
    };
    let watch_filter: Option<PathBuf> = if path.is_file() {
        Some(path.canonicalize().unwrap_or_else(|_| path.to_path_buf()))
    } else {
        None
    };
    let closure_epoch = std::sync::Arc::clone(&self_write_epoch);

    let mut debouncer = new_debouncer(
        DEBOUNCE_WINDOW,
        None,
        move |res: DebounceEventResult| match res {
            Ok(events) => {
                for ev in events {
                    for path in &ev.event.paths {
                        if let Some(filter) = &watch_filter {
                            if path != filter.as_path() {
                                continue;
                            }
                        } else if !is_layout_file(path) {
                            continue;
                        }
                        if let Some(event) = classify_event(path, ev.event.kind, &closure_epoch)
                        {
                            let _ = tx.send(event);
                        }
                    }
                }
            }
            Err(errors) => {
                for err in errors {
                    eprintln!("[file_watcher] notify error: {err:?}");
                }
            }
        },
    )
    .context("Failed to construct notify debouncer")?;

    debouncer
        .watcher()
        .watch(&watch_target, RecursiveMode::NonRecursive)
        .with_context(|| format!("Failed to watch {}", watch_target.display()))?;

    // `Debouncer` registers tick events on the cache even before the
    // first filesystem change; drop the initial DebouncedTick so callers
    // don't see a spurious event at startup.
    while rx.try_recv().is_ok() {}

    Ok(FileWatcherHandle {
        _debouncer: debouncer,
        rx,
        watched_path,
        self_write_epoch,
    })
}


/// Returns `true` if `path` looks like a layout file (`.json`).
///
/// Public so the web-side `watcher.rs` can reuse the same filter
/// without duplicating the extension check.
pub fn is_layout_file(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| e.eq_ignore_ascii_case("json"))
        .unwrap_or(false)
}

fn classify_event(
    path: &Path,
    kind: notify::EventKind,
    epoch: &SelfWriteEpoch,
) -> Option<FileEvent> {
    use notify::EventKind;

    let path = path.to_path_buf();
    match kind {
        EventKind::Remove(_) => Some(FileEvent::Removed { path }),
        EventKind::Modify(_) | EventKind::Create(_) | EventKind::Any | EventKind::Other => {
            let mtime = std::fs::metadata(&path)
                .and_then(|m| m.modified())
                .unwrap_or_else(|_| SystemTime::now());
            if should_ignore(epoch, mtime) {
                None
            } else {
                Some(FileEvent::Changed { path, mtime })
            }
        }
        EventKind::Access(_) => None,
    }
}


#[cfg(test)]
mod tests;
