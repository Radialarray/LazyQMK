//! File-watcher event handlers for the TUI hot-reload flow.
//!
//! The TUI event loop calls [`drain_file_watcher`] on every iteration
//! alongside the existing `build_state.poll()` drain. When the watcher
//! reports a `FileEvent::Changed`, we either reload silently (if the
//! TUI has no unsaved edits) or open the conflict-resolution prompt.
//! When it reports a `Removed` we open a removal prompt instead.

use std::ffi::OsString;
use std::path::{Path, PathBuf};

use crate::services::file_watcher::FileEvent;
use crate::services::LayoutService;
use crate::tui::app_state::{AppState, ExternalPendingKind};
use crate::tui::external_change_prompt::{
    ExternalChangeEvent, ExternalChangePrompt,
};
use crate::tui::{ActiveComponent, PopupType};

/// Drains all pending events from the file watcher and applies them to
/// `state`. Returns `true` if any state-changing action was taken.
pub fn drain_file_watcher(state: &mut AppState) -> bool {
    let Some(handle) = state.file_watcher.as_ref() else {
        return false;
    };

    let events = handle.drain();
    if events.is_empty() {
        return false;
    }

    let mut changed = false;
    for ev in events {
        match ev {
            FileEvent::Changed { path, .. } => {
                if path_matches(&path, state.source_path.as_deref()) {
                    state.pending_external = ExternalPendingKind::Change;
                    changed = true;
                }
            }
            FileEvent::Removed { path } => {
                if path_matches(&path, state.source_path.as_deref()) {
                    state.pending_external = ExternalPendingKind::Removal;
                    changed = true;
                }
            }
        }
    }

    if changed {
        apply_pending_external_events(state);
    }
    changed
}

/// Reconciles the pending external-event flag against the current
/// popup state. Called both from [`drain_file_watcher`] and from the
/// input handler so the prompt appears at the right time.
pub fn apply_pending_external_events(state: &mut AppState) {
    match state.pending_external {
        ExternalPendingKind::None => {}
        ExternalPendingKind::Removal => {
            open_removal_prompt(state);
            state.pending_external = ExternalPendingKind::None;
        }
        ExternalPendingKind::Change => {
            if state.dirty {
                open_conflict_prompt(state);
            } else {
                // No local edits — safe to silently reload.
                state.reload_layout_from_disk();
            }
            state.pending_external = ExternalPendingKind::None;
        }
    }
}

fn open_conflict_prompt(state: &mut AppState) {
    if matches!(
        state.active_component,
        Some(ActiveComponent::ExternalChangePrompt(_))
    ) {
        return;
    }
    let file = state
        .source_path
        .as_ref()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| "<unsaved>".to_string());
    state.active_component = Some(ActiveComponent::ExternalChangePrompt(
        ExternalChangePrompt::new(file),
    ));
    state.active_popup = Some(PopupType::ExternalChangePrompt);
}

fn open_removal_prompt(state: &mut AppState) {
    if state.active_component.is_some() {
        // Don't clobber an active popup. The removal flag stays set
        // until the user dismisses the current popup; we reapply on
        // next drain.
        return;
    }
    let file = state
        .source_path
        .as_ref()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| "<unsaved>".to_string());
    state.active_component = Some(ActiveComponent::ExternalChangePrompt(
        ExternalChangePrompt::new(format!("{file} (file was removed on disk)")),
    ));
    state.active_popup = Some(PopupType::ExternalChangePrompt);
}

/// Resolves an [`ExternalChangeEvent`] returned by the prompt.
///
/// `was_removal` indicates whether the original prompt was opened
/// because the file disappeared (true) or because it was modified
/// while the editor had unsaved edits (false).
pub fn handle_external_change_event(state: &mut AppState, event: ExternalChangeEvent) {
    let was_removal = matches!(state.pending_external, ExternalPendingKind::Removal);
    state.pending_external = ExternalPendingKind::None;
    state.active_component = None;
    state.active_popup = None;

    match event {
        ExternalChangeEvent::Cancel => {
            if was_removal {
                state.set_status("File removed on disk — choose Save As to keep your work");
            } else {
                state.set_status("Reload dismissed");
            }
        }
        ExternalChangeEvent::Reload => {
            if was_removal {
                state.set_status("File is gone on disk — use Save As (Ctrl+Shift+S)");
            } else {
                state.reload_layout_from_disk();
            }
        }
        ExternalChangeEvent::KeepMine => {
            let Some(path) = state.source_path.clone() else {
                state.set_error("No source path to overwrite");
                return;
            };
            match LayoutService::save_with_epoch(
                &state.layout,
                &path,
                Some(&state.self_write_epoch),
            ) {
                Ok(()) => {
                    state.mark_clean();
                    state.set_status(format!("Overwrote {}", path.display()));
                }
                Err(e) => {
                    state.set_error(format!("Failed to overwrite: {e}"));
                }
            }
        }
        ExternalChangeEvent::SaveThenReload => {
            let Some(path) = state.source_path.clone() else {
                state.set_error("No source path to back up");
                return;
            };
            let sidecar = sidecar_path(&path);
            if let Err(e) = LayoutService::save_with_epoch(
                &state.layout,
                &sidecar,
                Some(&state.self_write_epoch),
            ) {
                state.set_error(format!("Failed to write sidecar: {e}"));
                return;
            }
            state.reload_layout_from_disk();
            state.set_status(format!(
                "Backed up local edits to {} and reloaded from disk",
                sidecar.display()
            ));
        }
    }
}

fn path_matches(event_path: &PathBuf, source: Option<&Path>) -> bool {
    let Some(src) = source else {
        return false;
    };
    let event_canon = event_path
        .canonicalize()
        .unwrap_or_else(|_| event_path.clone());
    let src_canon = src.canonicalize().unwrap_or_else(|_| src.to_path_buf());
    event_canon == src_canon
}

fn sidecar_path(path: &Path) -> PathBuf {
    let mut name = path.file_name().map_or_else(OsString::new, OsString::from);
    name.push(".local");
    path.with_file_name(name)
}
