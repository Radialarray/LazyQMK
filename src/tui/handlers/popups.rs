//! Popup input handlers extracted from main TUI module.

use anyhow::Result;
use crossterm::event::{self, KeyCode, KeyModifiers};

use crate::services::LayoutService;
use crate::tui::{
    key_editor, keycode_picker, metadata_editor, onboarding_wizard, AppState,
    ParameterizedKeycodeType, PopupType,
};

/// Handle input for build log viewer
pub fn handle_build_log_input(state: &mut AppState, key: event::KeyEvent) -> Result<bool> {
    match key.code {
        KeyCode::Esc | KeyCode::Char('l') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            state.active_popup = None;
            state.build_log_state.visible = false;
            state.set_status("Build log closed");
            Ok(false)
        }
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            // Copy build log to clipboard
            if let Some(build_state) = &state.build_state {
                let log_text = build_state
                    .log_lines
                    .iter()
                    .map(|(_, message)| message.as_str())
                    .collect::<Vec<_>>()
                    .join("\n");

                match arboard::Clipboard::new()
                    .and_then(|mut clipboard| clipboard.set_text(log_text))
                {
                    Ok(()) => state.set_status("Build log copied to clipboard"),
                    Err(e) => state.set_error(format!("Failed to copy to clipboard: {e}")),
                }
            } else {
                state.set_error("No build log available");
            }
            Ok(false)
        }
        KeyCode::Up | KeyCode::Char('k') => {
            state.build_log_state.scroll_up();
            Ok(false)
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if let Some(build_state) = &state.build_state {
                let max_lines = build_state.log_lines.len();
                state.build_log_state.scroll_down(max_lines, 20); // Approximate visible lines
            }
            Ok(false)
        }
        KeyCode::Home => {
            state.build_log_state.scroll_to_top();
            Ok(false)
        }
        KeyCode::End => {
            if let Some(build_state) = &state.build_state {
                let max_lines = build_state.log_lines.len();
                state.build_log_state.scroll_to_bottom(max_lines, 20); // Approximate visible lines
            }
            Ok(false)
        }
        _ => Ok(false),
    }
}

/// Handle input for help overlay
pub fn handle_help_overlay_input(state: &mut AppState, key: event::KeyEvent) -> Result<bool> {
    match key.code {
        // Close help with Escape or '?'
        KeyCode::Esc | KeyCode::Char('?') => {
            state.active_popup = None;
            state.set_status("Press ? for help");
            Ok(false)
        }
        // Scroll up
        KeyCode::Up | KeyCode::Char('k') => {
            state.help_overlay_state.scroll_up();
            Ok(false)
        }
        // Scroll down
        KeyCode::Down | KeyCode::Char('j') => {
            state.help_overlay_state.scroll_down();
            Ok(false)
        }
        // Page up
        KeyCode::PageUp => {
            state.help_overlay_state.page_up(20); // Approximate visible height
            Ok(false)
        }
        // Page down
        KeyCode::PageDown => {
            state.help_overlay_state.page_down(20); // Approximate visible height
            Ok(false)
        }
        // Home - scroll to top
        KeyCode::Home => {
            state.help_overlay_state.scroll_to_top();
            Ok(false)
        }
        // End - scroll to bottom
        KeyCode::End => {
            state.help_overlay_state.scroll_to_bottom();
            Ok(false)
        }
        _ => Ok(false),
    }
}

/// Handle input for metadata editor
pub fn handle_metadata_editor_input(state: &mut AppState, key: event::KeyEvent) -> Result<bool> {
    let action =
        metadata_editor::handle_metadata_editor_input(&mut state.metadata_editor_state, key);

    match action {
        metadata_editor::MetadataEditorAction::Confirm => {
            // Check if name changed before applying (need to do this before apply mutates state)
            let name_changed = state.metadata_editor_state.name_changed();
            let new_name = state.metadata_editor_state.name.clone();

            // Validate and apply changes
            match state
                .metadata_editor_state
                .apply_to_layout(&mut state.layout)
            {
                Ok(()) => {
                    state.mark_dirty();

                    // If name changed and we have a source file, rename it
                    if name_changed {
                        if let Some(ref old_path) = state.source_path {
                            match LayoutService::rename_file_if_needed(old_path, &new_name) {
                                Ok(Some(new_path)) => {
                                    state.source_path = Some(new_path);
                                    state.set_status(format!("Layout renamed to '{new_name}'"));
                                }
                                Ok(None) => {
                                    state.set_status("Metadata updated");
                                }
                                Err(e) => {
                                    state.set_error(format!("Failed to rename file: {e}"));
                                }
                            }
                        } else {
                            state.set_status("Metadata updated");
                        }
                    } else {
                        state.set_status("Metadata updated");
                    }

                    state.active_popup = None;
                }
                Err(err) => {
                    state.set_error(format!("Validation failed: {err}"));
                }
            }
            Ok(false)
        }
        metadata_editor::MetadataEditorAction::Cancel => {
            state.active_popup = None;
            state.set_status("Metadata editing cancelled");
            Ok(false)
        }
        metadata_editor::MetadataEditorAction::Continue => Ok(false),
    }
}

/// Handle input for layout picker
pub fn handle_layout_picker_input(state: &mut AppState, key: event::KeyEvent) -> Result<bool> {
    use crate::tui::config_dialogs;
    
    if let Some(selected) =
        config_dialogs::handle_layout_picker_input(&mut state.layout_picker_state, key)
    {
        if selected.is_empty() {
            // User cancelled
            if state.return_to_settings_after_picker {
                state.return_to_settings_after_picker = false;
                state.active_popup = Some(PopupType::SettingsManager);
                state.set_status("Layout selection cancelled");
            } else {
                state.active_popup = None;
                state.set_status("Layout selection cancelled");
            }
            return Ok(false);
        }

        // User selected a layout - rebuild geometry and mapping
        match state.rebuild_geometry(&selected) {
            Ok(()) => {
                if state.return_to_settings_after_picker {
                    state.return_to_settings_after_picker = false;
                    state.active_popup = Some(PopupType::SettingsManager);
                    state.set_status(format!("Switched to layout: {selected}"));
                } else {
                    state.active_popup = None;
                    state.set_status(format!("Switched to layout: {selected}"));
                }
                state.mark_dirty(); // Config change requires save
            }
            Err(e) => {
                state.set_error(format!("Failed to switch layout: {e}"));
                if state.return_to_settings_after_picker {
                    state.return_to_settings_after_picker = false;
                    state.active_popup = Some(PopupType::SettingsManager);
                }
            }
        }
    }
    Ok(false)
}

/// Handle input for layer picker (for layer-switching keycodes)
pub fn handle_layer_picker_input(state: &mut AppState, key: event::KeyEvent) -> Result<bool> {
    match key.code {
        KeyCode::Esc => {
            // Check if we were editing a combo part
            if state.key_editor_state.combo_edit.is_some() {
                state.key_editor_state.combo_edit = None;
                state.active_popup = Some(PopupType::KeyEditor);
                state.layer_picker_state.reset();
                state.set_status("Cancelled - back to key editor");
                return Ok(false);
            }
            // Check if this was part of a parameterized keycode flow
            if state.pending_keycode.keycode_type.is_some() {
                state.pending_keycode.reset();
            }
            state.active_popup = None;
            state.layer_picker_state.reset();
            state.set_status("Layer selection cancelled");
            Ok(false)
        }
        KeyCode::Up | KeyCode::Char('k') => {
            let layer_count = state.layout.layers.len();
            state.layer_picker_state.select_previous(layer_count);
            Ok(false)
        }
        KeyCode::Down | KeyCode::Char('j') => {
            let layer_count = state.layout.layers.len();
            state.layer_picker_state.select_next(layer_count);
            Ok(false)
        }
        KeyCode::Enter => {
            // Get the selected layer
            let selected_idx = state.layer_picker_state.selected;
            if let Some(layer) = state.layout.layers.get(selected_idx) {
                let layer_ref = format!("@{}", layer.id);

                // Check if we're editing a combo keycode part
                if let Some((part, combo_type)) = state.key_editor_state.combo_edit.take() {
                    let new_combo = match part {
                        key_editor::ComboEditPart::Hold => combo_type.with_hold(&layer_ref),
                        key_editor::ComboEditPart::Tap => combo_type.with_tap(&layer_ref),
                    };
                    let new_keycode = new_combo.to_keycode();

                    if let Some(key) = state.get_selected_key_mut() {
                        key.keycode = new_keycode.clone();
                        state.mark_dirty();
                        state.set_status(format!("Updated: {new_keycode}"));
                    }

                    state.active_popup = Some(PopupType::KeyEditor);
                    state.layer_picker_state.reset();
                    return Ok(false);
                }

                // Check if we're in a parameterized keycode flow
                match &state.pending_keycode.keycode_type {
                    Some(ParameterizedKeycodeType::LayerTap) => {
                        // LT: Store layer, go to tap keycode picker
                        state.pending_keycode.param1 = Some(layer_ref);
                        state.active_popup = Some(PopupType::TapKeycodePicker);
                        state.keycode_picker_state = keycode_picker::KeycodePickerState::new();
                        state.layer_picker_state.reset();
                        state.set_status("Select tap keycode for LT");
                        return Ok(false);
                    }
                    Some(ParameterizedKeycodeType::LayerMod) => {
                        // LM: Store layer, go to modifier picker
                        state.pending_keycode.param1 = Some(layer_ref);
                        state.active_popup = Some(PopupType::ModifierPicker);
                        state.layer_picker_state.reset();
                        state.set_status("Select modifier(s) for LM");
                        return Ok(false);
                    }
                    _ => {
                        // Regular layer keycode (MO, TG, TO, etc.) - assign directly
                        let keycode = state.layer_picker_state.build_keycode(layer);

                        if let Some(key) = state.get_selected_key_mut() {
                            key.keycode = keycode.clone();
                            state.mark_dirty();
                            state.set_status(format!("Assigned: {keycode}"));
                        }
                    }
                }
            }

            state.active_popup = None;
            state.layer_picker_state.reset();
            Ok(false)
        }
        _ => Ok(false),
    }
}

/// Handle input for tap keycode picker (second stage of `LT/MT/SH_T`)
pub fn handle_tap_keycode_picker_input(state: &mut AppState, key: event::KeyEvent) -> Result<bool> {
    match key.code {
        KeyCode::Esc => {
            // Cancel the whole parameterized keycode flow
            state.pending_keycode.reset();
            state.active_popup = None;
            state.keycode_picker_state = keycode_picker::KeycodePickerState::new();
            state.set_status("Cancelled");
            Ok(false)
        }
        KeyCode::Enter => {
            // Get the selected keycode
            let keycodes = keycode_picker::get_filtered_keycodes(state);
            if let Some(kc) = keycodes.get(state.keycode_picker_state.selected) {
                // Only allow basic keycodes for tap action (no parameterized keycodes)
                if is_basic_keycode(&kc.code) {
                    // Store as param2 (tap keycode) for LT/MT, or param1 for SH_T
                    match &state.pending_keycode.keycode_type {
                        Some(ParameterizedKeycodeType::SwapHandsTap) => {
                            state.pending_keycode.param1 = Some(kc.code.clone());
                        }
                        _ => {
                            state.pending_keycode.param2 = Some(kc.code.clone());
                        }
                    }

                    // Build and assign the final keycode
                    if let Some(final_keycode) = state.pending_keycode.build_keycode() {
                        if let Some(key) = state.get_selected_key_mut() {
                            key.keycode = final_keycode.clone();
                            state.mark_dirty();
                            state.set_status(format!("Assigned: {final_keycode}"));
                        }
                    }

                    // Reset and close
                    state.pending_keycode.reset();
                    state.active_popup = None;
                    state.keycode_picker_state = keycode_picker::KeycodePickerState::new();
                } else {
                    state.set_error("Only basic keycodes allowed for tap action");
                }
            }
            Ok(false)
        }
        // Delegate navigation to standard keycode picker
        _ => keycode_picker::handle_navigation(state, key),
    }
}

/// Handle input for modifier picker (for MT/LM keycodes)
#[allow(clippy::too_many_lines)]
pub fn handle_modifier_picker_input(state: &mut AppState, key: event::KeyEvent) -> Result<bool> {
    match key.code {
        KeyCode::Esc => {
            // Check if we were editing a combo part
            if state.key_editor_state.combo_edit.is_some() {
                state.key_editor_state.combo_edit = None;
                state.active_popup = Some(PopupType::KeyEditor);
                state.modifier_picker_state.reset();
                state.set_status("Cancelled - back to key editor");
                return Ok(false);
            }
            state.pending_keycode.reset();
            state.modifier_picker_state.reset();
            state.active_popup = None;
            state.set_status("Cancelled");
            Ok(false)
        }
        KeyCode::Up | KeyCode::Char('k') => {
            state.modifier_picker_state.focus_up();
            Ok(false)
        }
        KeyCode::Down | KeyCode::Char('j') => {
            state.modifier_picker_state.focus_down();
            Ok(false)
        }
        KeyCode::Left | KeyCode::Char('h') => {
            state.modifier_picker_state.focus_left();
            Ok(false)
        }
        KeyCode::Right | KeyCode::Char('l') => {
            state.modifier_picker_state.focus_right();
            Ok(false)
        }
        KeyCode::Char(' ') => {
            // Toggle the focused modifier
            state.modifier_picker_state.toggle_focused();
            Ok(false)
        }
        KeyCode::Enter => {
            // Confirm selection - need at least one modifier
            if !state.modifier_picker_state.has_selection() {
                state.set_error("Select at least one modifier");
                return Ok(false);
            }

            let mod_string = state.modifier_picker_state.to_mod_string();

            // Check if we're editing a combo keycode part
            if let Some((part, combo_type)) = state.key_editor_state.combo_edit.take() {
                let new_combo = match part {
                    key_editor::ComboEditPart::Hold => combo_type.with_hold(&mod_string),
                    key_editor::ComboEditPart::Tap => combo_type.with_tap(&mod_string),
                };
                let new_keycode = new_combo.to_keycode();

                if let Some(key) = state.get_selected_key_mut() {
                    key.keycode = new_keycode.clone();
                    state.mark_dirty();
                    state.set_status(format!("Updated: {new_keycode}"));
                }

                state.active_popup = Some(PopupType::KeyEditor);
                state.modifier_picker_state.reset();
                return Ok(false);
            }

            match &state.pending_keycode.keycode_type {
                Some(ParameterizedKeycodeType::ModTap) => {
                    // MT: Store modifier as param1, go to tap keycode picker
                    state.pending_keycode.param1 = Some(mod_string);
                    state.active_popup = Some(PopupType::TapKeycodePicker);
                    state.keycode_picker_state = keycode_picker::KeycodePickerState::new();
                    state.modifier_picker_state.reset();
                    state.set_status("Select tap keycode for MT");
                }
                Some(ParameterizedKeycodeType::LayerMod) => {
                    // LM: Store modifier as param2, build and assign
                    state.pending_keycode.param2 = Some(mod_string);

                    if let Some(final_keycode) = state.pending_keycode.build_keycode() {
                        if let Some(key) = state.get_selected_key_mut() {
                            key.keycode = final_keycode.clone();
                            state.mark_dirty();
                            state.set_status(format!("Assigned: {final_keycode}"));
                        }
                    }

                    state.pending_keycode.reset();
                    state.modifier_picker_state.reset();
                    state.active_popup = None;
                }
                Some(ParameterizedKeycodeType::SingleMod) => {
                    // OSM and similar: Store modifier as param2, build and assign
                    state.pending_keycode.param2 = Some(mod_string);

                    if let Some(final_keycode) = state.pending_keycode.build_keycode() {
                        if let Some(key) = state.get_selected_key_mut() {
                            key.keycode = final_keycode.clone();
                            state.mark_dirty();
                            state.set_status(format!("Assigned: {final_keycode}"));
                        }
                    }

                    state.pending_keycode.reset();
                    state.modifier_picker_state.reset();
                    state.active_popup = None;
                }
                _ => {
                    // Unexpected state - cancel
                    state.pending_keycode.reset();
                    state.modifier_picker_state.reset();
                    state.active_popup = None;
                    state.set_error("Unexpected state");
                }
            }
            Ok(false)
        }
        _ => Ok(false),
    }
}

/// Check if a keycode is a basic keycode (not parameterized)
fn is_basic_keycode(code: &str) -> bool {
    // Basic keycodes: KC_A-Z, KC_0-9, KC_F1-24, navigation, symbols, etc.
    // Exclude: layer keycodes, mod-taps, parameterized keycodes
    !code.contains('(') && !code.contains('@')
}

/// Handle input for setup wizard
pub fn handle_setup_wizard_input(state: &mut AppState, key: event::KeyEvent) -> Result<bool> {
    // Delegate to wizard's handle_input function
    match onboarding_wizard::handle_input(&mut state.wizard_state, key) {
        Ok(should_exit) => {
            if should_exit {
                // Wizard completed or cancelled
                if state.wizard_state.is_complete {
                    // Check if this was a keyboard-only change
                    if state.wizard_state.keyboard_change_only {
                        // Only update keyboard and layout fields in layout metadata
                        if let Some(keyboard) = state.wizard_state.inputs.get("keyboard") {
                            state.layout.metadata.keyboard = Some(keyboard.clone());
                        }
                        if let Some(layout_variant) = state.wizard_state.inputs.get("layout") {
                            state.layout.metadata.layout_variant = Some(layout_variant.clone());
                        }

                        // Mark layout as modified
                        state.layout.metadata.touch();

                        // Rebuild geometry for new keyboard/layout
                        if let Some(layout_name) = state.layout.metadata.layout_variant.clone() {
                            match state.rebuild_geometry(&layout_name) {
                                Ok(()) => {
                                    let keyboard = state
                                        .layout
                                        .metadata
                                        .keyboard
                                        .as_deref()
                                        .unwrap_or("unknown");
                                    state.set_status(format!("Keyboard changed to: {keyboard}"));
                                }
                                Err(e) => {
                                    state.set_error(format!("Failed to rebuild geometry: {e}"));
                                }
                            }
                        }

                        // Return to settings manager
                        state.active_popup = Some(PopupType::SettingsManager);
                    } else {
                        // Full wizard - build and save the new config
                        match state.wizard_state.build_config() {
                            Ok(new_config) => {
                                // Update the app config
                                state.config = new_config;

                                // Save the config
                                if let Err(e) = state.config.save() {
                                    state.set_error(format!("Failed to save configuration: {e}"));
                                } else {
                                    state.set_status("Configuration saved successfully");
                                }
                            }
                            Err(e) => {
                                state.set_error(format!("Failed to build configuration: {e}"));
                            }
                        }
                        state.active_popup = None;
                    }
                } else {
                    // Wizard cancelled
                    if state.wizard_state.keyboard_change_only {
                        // Return to settings manager
                        state.active_popup = Some(PopupType::SettingsManager);
                        state.set_status("Keyboard selection cancelled");
                    } else {
                        state.active_popup = None;
                        state.set_status("Setup wizard cancelled");
                    }
                }

                // Reset wizard state for next time
                state.wizard_state = onboarding_wizard::OnboardingWizardState::new();
            }
            Ok(false)
        }
        Err(e) => {
            state.set_error(format!("Wizard error: {e}"));
            Ok(false)
        }
    }
}

/// Handle input for unsaved changes prompt
pub fn handle_unsaved_prompt_input(state: &mut AppState, key: event::KeyEvent) -> Result<bool> {
    match key.code {
        KeyCode::Char('s' | 'S') => {
            // Save and quit
            if let Some(path) = &state.source_path.clone() {
                LayoutService::save(&state.layout, path)?;
                state.mark_clean();
                state.set_status("Saved");
            }
            state.should_quit = true;
            Ok(true)
        }
        KeyCode::Char('q' | 'Q') => {
            // Quit without saving
            state.should_quit = true;
            Ok(true)
        }
        KeyCode::Esc => {
            // Cancel
            state.active_popup = None;
            state.set_status("Cancelled");
            Ok(false)
        }
        _ => Ok(false),
    }
}

/// Handle input when popup is active (dispatcher)
pub fn handle_popup_input(state: &mut AppState, key: event::KeyEvent) -> Result<bool> {
    use crate::tui::{category_picker, color_picker};
    
    let popup_type = state.active_popup.clone();

    match popup_type {
        Some(PopupType::KeycodePicker) => keycode_picker::handle_input(state, key),
        Some(PopupType::ColorPicker) => color_picker::handle_input(state, key),
        Some(PopupType::CategoryPicker) => category_picker::handle_input(state, key),
        Some(PopupType::CategoryManager) => super::handle_category_manager_input(state, key),
        Some(PopupType::LayerManager) => super::handle_layer_manager_input(state, key),
        Some(PopupType::LayerPicker) => handle_layer_picker_input(state, key),
        Some(PopupType::TemplateBrowser) => super::handle_template_browser_input(state, key),
        Some(PopupType::TemplateSaveDialog) => super::handle_template_save_dialog_input(state, key),
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
