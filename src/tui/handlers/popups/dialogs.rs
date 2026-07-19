//! Dialog and form handlers for build-log, help-overlay, metadata-editor,
//! setup-wizard, tap-dance-form, export-filename, and unsaved-changes popups.
//!
//! Extracted from src/tui/handlers/popups.rs to reduce file size. Each
//! `handle_*_input` function dispatches to its popup-specific logic.

use anyhow::Result;
use crossterm::event::{self, KeyCode, KeyModifiers};

use crate::services::LayoutService;
use crate::tui::component::Component;
use crate::tui::handlers::popups::parameterized::open_tap_dance_picker_with_context;
use crate::tui::keycode_picker;
use crate::tui::onboarding_wizard;
use crate::tui::{AppState, PopupType};

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
                        state.open_settings_manager();
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
                        state.open_settings_manager();
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

/// Handle input for tap dance form dialog
pub fn handle_tap_dance_form_input(state: &mut AppState, key: event::KeyEvent) -> Result<bool> {
    use crate::tui::tap_dance_form::TapDanceFormEvent;
    use crate::tui::ActiveComponent;

    // Extract the component from active_component
    let mut component = match state.active_component.take() {
        Some(ActiveComponent::TapDanceForm(form)) => form,
        _ => {
            // Component not found - close popup
            state.active_popup = None;
            return Ok(false);
        }
    };

    // Handle input and get event
    if let Some(event) = component.handle_input(key) {
        match event {
            TapDanceFormEvent::PickSingle => {
                // Cache the form and set pick target
                state.tap_dance_form_cache = Some(component);
                let target = crate::tui::tap_dance_form::FormRow::Single;
                state.tap_dance_form_pick_target = Some(target);
                open_tap_dance_picker_with_context(
                    state,
                    target,
                    "Select single tap keycode (required)",
                );
                return Ok(false);
            }
            TapDanceFormEvent::PickDouble => {
                // Cache the form and set pick target
                state.tap_dance_form_cache = Some(component);
                let target = crate::tui::tap_dance_form::FormRow::Double;
                state.tap_dance_form_pick_target = Some(target);
                open_tap_dance_picker_with_context(
                    state,
                    target,
                    "Select double tap keycode (required)",
                );
                return Ok(false);
            }
            TapDanceFormEvent::PickHold => {
                // Cache the form and set pick target
                state.tap_dance_form_cache = Some(component);
                let target = crate::tui::tap_dance_form::FormRow::Hold;
                state.tap_dance_form_pick_target = Some(target);
                open_tap_dance_picker_with_context(
                    state,
                    target,
                    "Select hold keycode (optional, Esc to skip)",
                );
                return Ok(false);
            }
            TapDanceFormEvent::Save(tap_dance, editing_index) => {
                let context = state
                    .tap_dance_form_context
                    .unwrap_or(crate::tui::TapDanceFormContext::FromEditor);

                // Validate the tap dance
                if let Err(e) = tap_dance.validate() {
                    state.set_error(format!("Validation failed: {e}"));
                    // Restore component
                    state.active_component = Some(ActiveComponent::TapDanceForm(component));
                    return Ok(false);
                }

                let name = tap_dance.name.clone();
                let status_message = if let Some(index) = editing_index {
                    // Update existing tap dance
                    if let Some(existing) = state.layout.tap_dances.get_mut(index) {
                        *existing = tap_dance;
                        state.mark_dirty();
                        format!("Tap dance '{name}' updated")
                    } else {
                        state.set_error(format!("Tap dance at index {index} not found"));
                        state.active_component = Some(ActiveComponent::TapDanceForm(component));
                        return Ok(false);
                    }
                } else {
                    // Add new tap dance
                    if let Err(e) = state.layout.add_tap_dance(tap_dance) {
                        state.set_error(format!("Failed to add tap dance: {e}"));
                        state.active_component = Some(ActiveComponent::TapDanceForm(component));
                        return Ok(false);
                    }
                    state.mark_dirty();
                    format!("Tap dance '{name}' created")
                };

                // Clear cached form state
                state.tap_dance_form_cache = None;
                state.tap_dance_form_pick_target = None;
                state.tap_dance_form_context = None;

                match context {
                    crate::tui::TapDanceFormContext::FromEditor => {
                        state.set_status(status_message);
                        state.active_popup = None;
                        state.active_component = None;
                        state.open_tap_dance_editor();
                    }
                    crate::tui::TapDanceFormContext::FromKeycodePicker => {
                        state.set_status(status_message);
                        if let Some(key) = state.get_selected_key_mut() {
                            let td_keycode = format!("TD({name})");
                            key.keycode = td_keycode.clone();
                            state.mark_dirty();
                            state.set_status(format!("Applied: {td_keycode}"));
                        } else {
                            state.set_error("No key selected");
                        }

                        state.active_popup = None;
                        state.active_component = None;
                    }
                }

                return Ok(false);
            }
            TapDanceFormEvent::Cancel => {
                let context = state
                    .tap_dance_form_context
                    .unwrap_or(crate::tui::TapDanceFormContext::FromEditor);

                state.tap_dance_form_cache = None;
                state.tap_dance_form_pick_target = None;
                state.tap_dance_form_context = None;
                state.active_popup = None;
                state.active_component = None;

                match context {
                    crate::tui::TapDanceFormContext::FromEditor => {
                        state.open_tap_dance_editor();
                        state.set_status("Tap dance form cancelled");
                    }
                    crate::tui::TapDanceFormContext::FromKeycodePicker => {
                        let mut picker = keycode_picker::KeycodePicker::with_language(
                            state.config.ui.last_language.clone(),
                            &state.keycode_db,
                        );
                        picker.set_flow_context(
                            "Tap dance setup".to_string(),
                            "Esc closes picker. Choose TD() again to reopen tap dance form.",
                        );
                        state.active_component = Some(ActiveComponent::KeycodePicker(picker));
                        state.active_popup = Some(PopupType::TapKeycodePicker);
                        state.set_status("Tap dance form cancelled");
                    }
                }
                return Ok(false);
            }
        }
    } else {
        // No event - restore component and continue
        state.active_component = Some(ActiveComponent::TapDanceForm(component));
    }

    Ok(false)
}

/// Handle keyboard picker input using Component trait pattern
/// Handle input for export filename dialog
pub fn handle_export_filename_dialog_input(
    state: &mut AppState,
    key: event::KeyEvent,
) -> Result<bool> {
    use crate::tui::handlers::action_handlers::file_ops;

    match key.code {
        KeyCode::Char(c) => {
            // Add character to filename
            state.export_filename_dialog_state.filename.push(c);
            Ok(false)
        }
        KeyCode::Backspace => {
            // Remove last character
            state.export_filename_dialog_state.filename.pop();
            Ok(false)
        }
        KeyCode::Enter => {
            // Perform export
            let filename = state.export_filename_dialog_state.filename.clone();
            state.active_popup = None;

            if filename.trim().is_empty() {
                state.set_error("Filename cannot be empty");
                return Ok(false);
            }

            // Ensure filename ends with .json (case-insensitive check)
            let filename = if std::path::Path::new(&filename)
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("json"))
            {
                filename
            } else {
                format!("{filename}.json")
            };

            match file_ops::perform_export(state, &filename) {
                Ok(()) => {
                    // Success message already set by perform_export
                }
                Err(e) => {
                    state.set_error(format!("Export failed: {e}"));
                }
            }
            Ok(false)
        }
        KeyCode::Esc => {
            // Cancel export
            state.active_popup = None;
            state.set_status("Export cancelled");
            Ok(false)
        }
        _ => Ok(false),
    }
}

/// Handle input for unsaved changes prompt
pub fn handle_unsaved_prompt_input(state: &mut AppState, key: event::KeyEvent) -> Result<bool> {
    match key.code {
        KeyCode::Char('s' | 'S')
            if key.modifiers.is_empty() || key.modifiers == KeyModifiers::CONTROL =>
        {
            // Save and quit
            if let Some(path) = &state.source_path.clone() {
                LayoutService::save(&state.layout, path)?;
                state.mark_clean();
                state.set_status("Saved");
            }
            state.should_quit = true;
            Ok(true)
        }
        KeyCode::Char('q' | 'Q')
            if key.modifiers.is_empty() || key.modifiers == KeyModifiers::CONTROL =>
        {
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
