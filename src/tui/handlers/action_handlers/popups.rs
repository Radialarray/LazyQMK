// Popup management action handlers

use crate::models::KeyDefinition;
use crate::tui::{key_editor, onboarding_wizard, ActiveComponent, AppState, PopupType};
use anyhow::Result;

/// Handle open layer manager action
pub fn handle_open_layer_manager(state: &mut AppState) -> Result<bool> {
    state.open_layer_manager();
    state.set_status("Layer Manager - n: new, d: delete, r: rename");
    Ok(false)
}

/// Handle open category manager action
pub fn handle_open_category_manager(state: &mut AppState) -> Result<bool> {
    state.open_category_manager();
    state.set_status("Category Manager - n: new, r: rename, c: color, d: delete");
    Ok(false)
}

/// Handle open settings action
pub fn handle_open_settings(state: &mut AppState) -> Result<bool> {
    state.open_settings_manager();
    state.set_status("Settings Manager - Enter: edit, Esc: close");
    Ok(false)
}

/// Handle edit metadata action
pub fn handle_edit_metadata(state: &mut AppState) -> Result<bool> {
    state.open_metadata_editor();
    state.set_status("Edit Metadata - Tab: next field, Enter: save");
    Ok(false)
}

/// Handle browse templates action
pub fn handle_browse_templates(state: &mut AppState) -> Result<bool> {
    state.open_template_browser();
    state.set_status("Template Browser - Enter: load, /: search");
    Ok(false)
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
        Some((key, true)) => {
            // Key is assigned - open key editor
            open_key_editor(state, &key);
        }
        Some((_, false)) | None => {
            // Key is empty or no key selected - open keycode picker
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
    state.set_status("Key editor - Enter: Reassign, D: Description, C: Color");
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
    state.open_tap_dance_editor();
    state.set_status("Tap Dance Editor - n: new, d: delete, Enter: select, Esc: cancel");
    Ok(false)
}
