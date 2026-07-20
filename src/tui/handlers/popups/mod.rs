//! Popup input dispatcher and shared utilities.
//!
//! `handle_popup_input` is the single entry point for popup key events.
//! It dispatches to the appropriate handler based on `state.active_popup`.
//!
//! This module is split into:
//! - `parameterized` — parameterized keycode flows + keycode/category/tap/modifier pickers
//! - `pickers` — color, layout, layer, layout-variant, tap-keycode, modifier pickers
//! - `dialogs` — build-log, help-overlay, metadata-editor, setup-wizard, tap-dance-form, export, unsaved-changes

pub mod dialogs;
pub mod parameterized;
pub mod pickers;

#[cfg(test)]
mod tests;

use anyhow::Result;
use crossterm::event::{self, KeyCode};

use crate::tui::component::ContextualComponent;
use crate::tui::editor::key_editor;
use crate::tui::{ActiveComponent, AppState, PopupType};

pub use dialogs::{
    handle_export_filename_dialog_input, handle_setup_wizard_input, handle_tap_dance_form_input,
    handle_unsaved_prompt_input,
};
#[cfg(test)]
pub use parameterized::extract_td_name;
pub use parameterized::{handle_category_picker_event, handle_keycode_picker_event};
#[cfg(test)]
pub use pickers::is_basic_or_layer_keycode;
pub use pickers::{
    handle_build_log_input, handle_color_picker_event, handle_help_overlay_input,
    handle_layer_picker_input, handle_layout_picker_input, handle_metadata_editor_input,
    handle_modifier_picker_input, handle_tap_keycode_picker_input,
};

/// Dispatch popup key events to the appropriate handler based on
/// `state.active_popup`. Each `PopupType` variant has a dedicated
/// handler; unknown variants close the popup on Escape.
pub fn handle_popup_input(state: &mut AppState, key: event::KeyEvent) -> Result<bool> {
    let popup_type = state.active_popup.clone();

    match popup_type {
        Some(PopupType::KeycodePicker) => {
            // Use ContextualComponent trait pattern
            if let Some(ActiveComponent::KeycodePicker(ref mut picker)) = state.active_component {
                if let Some(event) = picker.handle_input(key, &state.keycode_db) {
                    return handle_keycode_picker_event(state, event);
                }
            }
            Ok(false)
        }
        Some(PopupType::ColorPicker) => {
            // Use Component trait pattern
            handle_color_picker_event(state, key)
        }
        Some(PopupType::CategoryPicker) => {
            // Use ContextualComponent trait pattern
            if let Some(ActiveComponent::CategoryPicker(ref mut picker)) = state.active_component {
                if let Some(event) = picker.handle_input(key, &state.layout.categories) {
                    return handle_category_picker_event(state, event);
                }
            }
            Ok(false)
        }
        Some(PopupType::CategoryManager) => super::handle_category_manager_input(state, key),
        Some(PopupType::LayerManager) => super::handle_layer_manager_input(state, key),
        Some(PopupType::LayerPicker) => handle_layer_picker_input(state, key),
        Some(PopupType::TemplateBrowser) => super::handle_template_browser_input(state, key),
        Some(PopupType::TemplateSaveDialog) => super::handle_template_save_dialog_input(state, key),
        Some(PopupType::ExportFilenameDialog) => handle_export_filename_dialog_input(state, key),
        Some(PopupType::UnsavedChangesPrompt) => handle_unsaved_prompt_input(state, key),
        Some(PopupType::BuildLog) => handle_build_log_input(state, key),
        Some(PopupType::HelpOverlay) => handle_help_overlay_input(state, key),
        Some(PopupType::MetadataEditor) => handle_metadata_editor_input(state, key),
        Some(PopupType::LayoutPicker) => handle_layout_picker_input(state, key),
        Some(PopupType::SetupWizard) => handle_setup_wizard_input(state, key),
        Some(PopupType::SettingsManager) => super::handle_settings_manager_input(state, key),
        Some(PopupType::TapKeycodePicker) => handle_tap_keycode_picker_input(state, key),
        Some(PopupType::ModifierPicker) => handle_modifier_picker_input(state, key),
        Some(PopupType::KeyEditor) => key_editor::handle_input(state, key),
        Some(PopupType::TapDanceEditor) => super::handle_tap_dance_editor_input(state, key),
        Some(PopupType::TapDanceForm) => handle_tap_dance_form_input(state, key),
        _ => {
            // Escape closes any popup
            if key.code == KeyCode::Esc {
                state.active_popup = None;
                state.set_status("Cancelled");
            }
            Ok(false)
        }
    }
}
