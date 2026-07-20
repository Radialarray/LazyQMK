//! Terminal user interface components and state management.
//!
//! This module contains the main TUI loop, `AppState`, event handling,
//! and all UI widgets using Ratatui.
//!
//! Sub-module layout:
//! - `app_state`, `event_loop`, `popup_type`, `terminal`, `component` — core
//! - `picker/` — popups for selecting keycodes, layers, layouts, colors, etc.
//! - `editor/` — key and metadata editing UI
//! - `manager/` — popups for managing layers, categories, builds, clipboard
//! - `dialog/` — dialogs, help overlay, status bar, theme, onboarding wizard
//! - `settings_manager/` — settings manager UI
//! - `handlers/` — top-level input dispatch
//! - `render/`, `input/` — internal rendering and key dispatch

#![allow(clippy::assigning_clones)]
#![allow(clippy::unnecessary_wraps)]
#![allow(clippy::trivially_copy_pass_by_ref)]

pub mod app_state;
pub mod component;
pub mod dialog;
pub mod editor;
pub mod event_loop;
pub mod handlers;
mod input;
pub mod manager;
pub mod picker;
pub mod popup_type;
mod render;
pub mod settings_manager;
pub mod terminal;

pub use app_state::{
    ActiveComponent, AppState, CategoryPickerContext, ExportFilenameDialogState, SelectionMode,
    TapDanceFormContext, TemplateSaveDialogState,
};
pub use event_loop::run_tui;
pub use popup_type::{popup_border_style, popup_title, PopupType};
pub use terminal::{restore_terminal, setup_terminal};

// Backward-compatible flat re-exports for the pre-reorg public API.
pub use component::Component;
pub use config_dialogs::LayoutPickerEvent as LayoutVariantPickerEvent;

pub use dialog::{
    config_dialogs, help_overlay, help_registry, onboarding_wizard, status_bar, theme,
};
pub use editor::{keyboard, metadata_editor};
pub use manager::{build_log, category_manager, clipboard, layer_manager};

pub use picker::{
    category_picker, color_picker, keycode_picker, layer_picker, layout_picker, modifier_picker,
    tap_dance_editor, tap_dance_form, template_browser,
};

pub use category_manager::CategoryManager;
pub use category_picker::{CategoryPicker, CategoryPickerEvent};
pub use layer_manager::LayerManagerEvent;
pub use theme::Theme;

#[cfg(test)]
mod tests;
