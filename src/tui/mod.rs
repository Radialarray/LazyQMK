//! Terminal user interface components and state management.
//!
//! This module contains the main TUI loop, `AppState`, event handling,
//! and all UI widgets using Ratatui.
//!
//! The module is split across several sub-modules:
//! - `app_state` — `AppState`, `ActiveComponent`, `SelectionMode`, and supporting types
//! - `popup_type` — `PopupType`, `PopupVisualKind`, and popup chrome helpers
//! - `terminal` — terminal setup / teardown
//! - `event_loop` — `run_tui` the main event loop
//! - `render` — rendering functions (title bar, main content, dialogs)
//! - `input` — keyboard event dispatch

// Allow clone assignment patterns - common in UI state management
#![allow(clippy::assigning_clones)]
// Input handlers use Result<bool> for consistency even when they never fail
#![allow(clippy::unnecessary_wraps)]
// Allow small types passed by reference for API consistency
#![allow(clippy::trivially_copy_pass_by_ref)]

// Existing sub-modules (alphabetical order)
pub mod build_log;
pub mod category_manager;
pub mod category_picker;
pub mod clipboard;
pub mod color_picker;
pub mod component;
pub mod config_dialogs;
/// Editor components (key property editing UI).
pub mod editor;
pub mod handlers;
pub mod help_overlay;
pub mod help_registry;
pub mod keyboard;
pub mod keycode_picker;
pub mod layer_manager;
pub mod layer_picker;
pub mod layout_picker;
pub mod metadata_editor;
pub mod modifier_picker;
pub mod onboarding_wizard;
pub mod settings_manager;
pub mod status_bar;
pub mod tap_dance_editor;
pub mod tap_dance_form;
pub mod template_browser;
pub mod theme;

// New split sub-modules
pub mod app_state;
pub mod event_loop;
mod input;
pub mod popup_type;
mod render;
pub mod terminal;

// Re-export items needed by the crate's code and test code.
//
// NOTE: `render` and `input` modules are private so `crate::tui::render`
// and `crate::tui::handle_key_event` would conflict with the module names
// if both were public.  The functions are selectively re-exported below.

// Existing component re-exports used via `crate::tui::*`
pub use category_manager::CategoryManager;
pub use category_picker::{CategoryPicker, CategoryPickerEvent};
pub use component::Component;
pub use config_dialogs::LayoutPickerEvent as LayoutVariantPickerEvent;
pub use layer_manager::LayerManagerEvent;
pub use theme::Theme;

// Re-export from split modules
pub use app_state::{
    ActiveComponent, AppState, CategoryPickerContext, ExportFilenameDialogState, SelectionMode,
    TapDanceFormContext, TemplateSaveDialogState,
};
pub use event_loop::run_tui;
pub use popup_type::{popup_border_style, popup_title, PopupType};
pub use terminal::{restore_terminal, setup_terminal};

#[cfg(test)]
mod tests;
