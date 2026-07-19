// Popup management action handlers

use crate::models::KeyDefinition;
use crate::tui::editor::key_editor;
use crate::tui::{onboarding_wizard, ActiveComponent, AppState, PopupType};
use anyhow::Result;

/// Open a popup by invoking `opener` on the state and set a status message.
///
/// Consolidates the repeated `state.foo(); state.set_status(...); Ok(false)`
/// pattern across the simple popup-open handlers.
fn open_popup_with_status<F>(state: &mut AppState, opener: F, status: &str) -> Result<bool>
where
    F: FnOnce(&mut AppState),
{
    opener(state);
    state.set_status(status);
    Ok(false)
}

/// Handle open layer manager action
pub fn handle_open_layer_manager(state: &mut AppState) -> Result<bool> {
    open_popup_with_status(
        state,
        crate::tui::AppState::open_layer_manager,
        "Layers: add, rename, reorder, copy, or delete layers",
    )
}

/// Handle open category manager action
pub fn handle_open_category_manager(state: &mut AppState) -> Result<bool> {
    open_popup_with_status(
        state,
        crate::tui::AppState::open_category_manager,
        "Key categories: add, rename, recolor, or delete categories",
    )
}

/// Handle open settings action
pub fn handle_open_settings(state: &mut AppState) -> Result<bool> {
    open_popup_with_status(
        state,
        crate::tui::AppState::open_settings_manager,
        "Settings: choose task, Enter edits, Esc closes",
    )
}

/// Handle edit metadata action
pub fn handle_edit_metadata(state: &mut AppState) -> Result<bool> {
    open_popup_with_status(
        state,
        crate::tui::AppState::open_metadata_editor,
        "Metadata: Tab moves between fields, Enter saves",
    )
}

/// Handle browse templates action
pub fn handle_browse_templates(state: &mut AppState) -> Result<bool> {
    open_popup_with_status(
        state,
        crate::tui::AppState::open_template_browser,
        "Templates: Enter loads, / searches",
    )
}

/// Handle toggle help action
pub fn handle_toggle_help(state: &mut AppState) -> Result<bool> {
    if state.active_popup == Some(PopupType::HelpOverlay) {
        state.close_component();
    } else {
        state.open_help_overlay();
    }
    Ok(false)
}

/// Handle view build log action
pub fn handle_view_build_log(state: &mut AppState) -> Result<bool> {
    if state.build_state.is_some() {
        if matches!(state.active_component, Some(ActiveComponent::BuildLog(_))) {
            state.close_component();
        } else {
            state.open_build_log();
        }
        state.set_status("Build log toggled");
    } else {
        state.set_error("No build active");
    }
    Ok(false)
}

/// Handle open keycode picker action
pub fn handle_open_keycode_picker(state: &mut AppState) -> Result<bool> {
    match get_selected_key_info(state) {
        Some((key, _)) => {
            open_key_editor(state, &key);
        }
        None => {
            open_keycode_picker_for_empty_key(state);
        }
    }
    Ok(false)
}

/// Get selected key and whether it's assigned
fn get_selected_key_info(state: &AppState) -> Option<(KeyDefinition, bool)> {
    state
        .get_selected_key()
        .map(|key| (key.clone(), key_editor::is_key_assigned(&key.keycode)))
}

/// Open key editor for an assigned key
fn open_key_editor(state: &mut AppState, key: &KeyDefinition) {
    state
        .key_editor_state
        .init_for_key(key, state.current_layer);
    state.active_popup = Some(PopupType::KeyEditor);
    state.set_status("Key details - review current key, Enter to assign keycode");
}

/// Open keycode picker for an empty/unassigned key
fn open_keycode_picker_for_empty_key(state: &mut AppState) {
    state.open_keycode_picker();
    state.set_status("Select keycode - Type to search, Enter to apply");
}

/// Handle setup wizard action
pub fn handle_setup_wizard(state: &mut AppState) -> Result<bool> {
    let qmk_path = match &state.config.paths.qmk_firmware {
        Some(path) => path.clone(),
        None => {
            state.set_error("QMK firmware path not configured");
            return Ok(false);
        }
    };

    match onboarding_wizard::OnboardingWizardState::new_for_keyboard_selection(&qmk_path) {
        Ok(wizard_state) => {
            state.wizard_state = wizard_state;
            state.active_popup = Some(PopupType::SetupWizard);
            state.set_status("Setup Wizard - Follow prompts to configure");
        }
        Err(e) => {
            state.set_error(format!("Failed to start wizard: {e}"));
        }
    }
    Ok(false)
}

/// Handle open tap dance editor action
pub fn handle_open_tap_dance_editor(state: &mut AppState) -> Result<bool> {
    open_popup_with_status(
        state,
        crate::tui::AppState::open_tap_dance_editor,
        "Tap Dance Editor - visible actions shown in status bar and help",
    )
}
