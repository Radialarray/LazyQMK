//! Parameterized keycode flows and tap-dance / modifier / keycode picker handlers.
//!
//! Extracted from src/tui/handlers/popups.rs (1802 lines) to reduce file size.
//! All functions take `&mut AppState` and return `Result<bool>`.

use anyhow::Result;

use crate::keycode_db::{KeycodeDb, ParamType};
use crate::tui::editor::key_editor;
use crate::tui::handlers::popups::pickers::{is_basic_keycode, is_basic_or_layer_keycode};
use crate::tui::keycode_picker::{self, KeycodePickerEvent};
use crate::tui::{ActiveComponent, AppState, PopupType};

fn tap_dance_flow_label(target: crate::tui::tap_dance_form::FormRow) -> &'static str {
    match target {
        crate::tui::tap_dance_form::FormRow::Name => "Name",
        crate::tui::tap_dance_form::FormRow::Single => "Single tap",
        crate::tui::tap_dance_form::FormRow::Double => "Double tap",
        crate::tui::tap_dance_form::FormRow::Hold => "Hold",
    }
}

/// Open the keycode picker as a sub-flow of the tap-dance form.
/// Used by the tap-dance form to collect each tap-dance action.
pub fn open_tap_dance_picker_with_context(
    state: &mut AppState,
    target: crate::tui::tap_dance_form::FormRow,
    status: &str,
) {
    let mut picker = keycode_picker::KeycodePicker::with_language(
        state.config.ui.last_language.clone(),
        &state.keycode_db,
    );
    picker.set_flow_context(
        format!("Tap dance → {}", tap_dance_flow_label(target)),
        "Esc returns to tap dance form without losing draft. Enter applies chosen keycode.",
    );
    state.active_component = Some(ActiveComponent::KeycodePicker(picker));
    state.active_popup = Some(PopupType::KeycodePicker);
    state.set_status(status);
}

/// Extracts the tap dance name from a TD(name) keycode.
/// Returns None if the keycode is not a valid `TD()` pattern.
pub fn extract_td_name(keycode: &str) -> Option<String> {
    if let Some(stripped) = keycode.strip_prefix("TD(") {
        if let Some(name) = stripped.strip_suffix(')') {
            if !name.is_empty() {
                return Some(name.to_string());
            }
        }
    }
    None
}

/// Start a parameterized keycode flow using `KeycodeDb` param metadata (data-driven approach)
/// Returns true if the keycode was handled as parameterized.
fn start_parameterized_keycode_flow(state: &mut AppState, keycode: &str) -> bool {
    // Reset any previous state
    state.pending_keycode.reset();

    // Look up params from the keycode DB
    let _params = match state.keycode_db.get_params(keycode) {
        Some(p) if !p.is_empty() => p,
        _ => return false, // Not a parameterized keycode
    };

    // Store the template for later building
    state.pending_keycode.keycode_template = Some(keycode.to_string());

    // Open the first picker based on params[0].param_type
    open_picker_for_param_index(state, 0);

    true
}

/// Open the appropriate picker for the parameter at the given index in the current flow
fn open_picker_for_param_index(state: &mut AppState, param_idx: usize) {
    // Clone template to avoid borrow conflicts
    let template = match &state.pending_keycode.keycode_template {
        Some(t) => t.clone(),
        None => {
            state.set_error("No keycode template in pending state");
            return;
        }
    };

    let params = match state.keycode_db.get_params(&template) {
        Some(p) => p,
        None => {
            state.set_error("Failed to get params for keycode template");
            return;
        }
    };

    if param_idx >= params.len() {
        state.set_error("Parameter index out of bounds");
        return;
    }

    // Clone the parameter data we need before calling state methods
    let param = &params[param_idx];
    let param_type = param.param_type;
    let prefix = KeycodeDb::get_prefix(&template)
        .unwrap_or(&template)
        .to_string();

    // Build status message from param description or generic fallback
    let message = param
        .description
        .as_ref()
        .map(|d| {
            // Capitalize first letter and clean up description
            let desc = d.trim().to_lowercase();
            format!("Select {desc}")
        })
        .unwrap_or_else(|| format!("Select {} for {prefix}", param.name));

    // Open the appropriate picker based on parameter type
    match param_type {
        ParamType::Layer => {
            state.open_layer_picker(&prefix);
            state.set_status(&message);
        }
        ParamType::Modifier => {
            state.open_modifier_picker();
            state.set_status(&message);
        }
        ParamType::Keycode => {
            // Use component-based picker (no last_language since tap keycodes are typically basic)
            let picker = keycode_picker::KeycodePicker::new();
            state.active_component = Some(ActiveComponent::KeycodePicker(picker));
            state.active_popup = Some(PopupType::TapKeycodePicker);
            state.set_status(&message);
        }
        ParamType::TapDance => {
            // Launch the tap dance form (Option B) directly
            let existing_names = state
                .layout
                .tap_dances
                .iter()
                .map(|td| td.name.clone())
                .collect::<Vec<_>>();

            let form = crate::tui::tap_dance_form::TapDanceForm::new_create(existing_names);
            state.tap_dance_form_context = Some(crate::tui::TapDanceFormContext::FromKeycodePicker);
            state.active_component = Some(ActiveComponent::TapDanceForm(form));
            state.active_popup = Some(PopupType::TapDanceForm);
            state.pending_keycode.reset(); // stop further param flow
            state.set_status("Tap Dance: fill name/single/double (hold optional)");
        }
    }
}

/// Handle a parameter being selected - add it to the collected params and open next picker or finish
pub fn handle_parameter_collected(state: &mut AppState, param_value: String) {
    // Add to collected params
    state.pending_keycode.params.push(param_value);

    let template = match &state.pending_keycode.keycode_template {
        Some(t) => t.clone(),
        None => {
            state.set_error("No keycode template in pending state");
            state.pending_keycode.reset();
            return;
        }
    };

    let params_meta = match state.keycode_db.get_params(&template) {
        Some(p) => p,
        None => {
            state.set_error("Failed to get params for keycode template");
            state.pending_keycode.reset();
            return;
        }
    };

    let current_param_count = state.pending_keycode.params.len();

    if current_param_count < params_meta.len() {
        // More params needed - open next picker
        open_picker_for_param_index(state, current_param_count);
    } else {
        // All params collected - build final keycode
        if let Some(final_keycode) = state.pending_keycode.build_keycode() {
            if let Some(key) = state.get_selected_key_mut() {
                key.keycode = final_keycode.clone();
                state.mark_dirty();
                state.refresh_layer_refs(); // Update layer reference index
                state.set_status(format!("Assigned: {final_keycode}"));
            }
        } else {
            state.set_error("Failed to build final keycode");
        }

        state.pending_keycode.reset();
        state.close_component();
    }
}

/// Handle keycode picker events
pub fn handle_keycode_picker_event(state: &mut AppState, event: KeycodePickerEvent) -> Result<bool> {
    // Save last-used language to config if changed
    // Get the selected language from the active component (not the legacy state)
    let selected_language =
        if let Some(ActiveComponent::KeycodePicker(ref picker)) = state.active_component {
            picker.state().selected_language.clone()
        } else {
            None
        };

    if selected_language != state.config.ui.last_language {
        state.config.ui.last_language = selected_language;
        // Save config to persist the language preference
        let _ = state.config.save();
    }

    match event {
        KeycodePickerEvent::KeycodeSelected(keycode) => {
            // If user selected TD() directly from the picker, launch the tap dance form
            if keycode == "TD()" || keycode.starts_with("TD(") {
                // Existing names for duplicate validation
                let existing_names: Vec<String> = state
                    .layout
                    .tap_dances
                    .iter()
                    .map(|td| td.name.clone())
                    .collect();

                // Check if the current key already has a TD() keycode - if so, edit that tap dance
                let form = if let Some(current_key) = state.get_selected_key() {
                    let current_keycode = &current_key.keycode;

                    // Parse TD(name) pattern to extract the name
                    if let Some(td_name) = extract_td_name(current_keycode) {
                        // Find the tap dance definition
                        if let Some((index, tap_dance)) = state
                            .layout
                            .tap_dances
                            .iter()
                            .enumerate()
                            .find(|(_, td)| td.name == td_name)
                        {
                            // Edit existing tap dance
                            crate::tui::tap_dance_form::TapDanceForm::new_edit(
                                tap_dance.clone(),
                                index,
                                existing_names,
                            )
                        } else {
                            // TD reference exists but no definition found (auto-created placeholder)
                            // Create new tap dance with that name
                            crate::tui::tap_dance_form::TapDanceForm::new_create(existing_names)
                        }
                    } else {
                        // Current key doesn't have TD(), create new
                        crate::tui::tap_dance_form::TapDanceForm::new_create(existing_names)
                    }
                } else {
                    // No selected key, create new
                    crate::tui::tap_dance_form::TapDanceForm::new_create(existing_names)
                };

                state.tap_dance_form_context =
                    Some(crate::tui::TapDanceFormContext::FromKeycodePicker);
                state.active_component = Some(ActiveComponent::TapDanceForm(form));
                state.active_popup = Some(PopupType::TapDanceForm);
                state.pending_keycode.reset();
                state.set_status("Tap Dance: fill name/single/double (hold optional)");
                return Ok(false);
            }

            // Check if we're in a tap dance form picker flow
            if let Some(mut form) = state.tap_dance_form_cache.take() {
                // Check if this is a parameterized keycode that needs more input
                // (e.g., MO() needs a layer number)
                if start_parameterized_keycode_flow(state, &keycode) {
                    // Parameterized flow started - keep form cached and pick target for later
                    state.tap_dance_form_cache = Some(form);
                    // Pick target remains set for when the parameterized keycode is completed
                    return Ok(false);
                }

                // Apply keycode to the appropriate field based on pick target
                if let Some(target) = state.tap_dance_form_pick_target.take() {
                    use crate::tui::tap_dance_form::FormRow;
                    match target {
                        FormRow::Single | FormRow::Double => {
                            // Single/Double taps must be basic keycodes (no parameterized)
                            if !is_basic_keycode(&keycode) {
                                state
                                    .set_error("Only basic keycodes allowed for single/double tap");
                                state.tap_dance_form_cache = Some(form);
                                state.tap_dance_form_pick_target = Some(target);
                                return Ok(false);
                            }

                            if target == FormRow::Single {
                                form.set_single_tap(keycode.clone());
                                state.set_status(format!("Single tap set to: {keycode}"));
                            } else {
                                form.set_double_tap(keycode.clone());
                                state.set_status(format!("Double tap set to: {keycode}"));
                            }
                        }
                        FormRow::Hold => {
                            // Hold action can be any keycode, including layer keycodes (MO, TG, etc.)
                            // But still reject complex parameterized keycodes like LT, MT
                            if !is_basic_or_layer_keycode(&keycode) {
                                state.set_error("Hold action: use basic keycodes or layer keycodes (MO, TG, TO, etc.)");
                                state.tap_dance_form_cache = Some(form);
                                state.tap_dance_form_pick_target = Some(target);
                                return Ok(false);
                            }

                            form.set_hold(keycode.clone());
                            state.set_status(format!("Hold set to: {keycode}"));
                        }
                        FormRow::Name => {
                            // Should never happen
                            state.set_error("Invalid state: picker opened for name field");
                        }
                    }
                }

                // Restore form and close picker
                state.active_component = Some(ActiveComponent::TapDanceForm(form));
                state.active_popup = Some(PopupType::TapDanceForm);
                return Ok(false);
            }

            // Check if we're editing a combo keycode part
            if let Some((part, combo_type)) = state.key_editor_state.combo_edit.take() {
                // Validate that the new keycode is basic
                if !is_basic_keycode(&keycode) {
                    state.set_error("Only basic keycodes allowed inside a combo");
                    // Restore the combo edit state
                    state.key_editor_state.combo_edit = Some((part, combo_type));
                    return Ok(false);
                }

                let new_combo = match part {
                    key_editor::ComboEditPart::Hold => combo_type.with_hold(&keycode),
                    key_editor::ComboEditPart::Tap => combo_type.with_tap(&keycode),
                };
                let new_keycode = new_combo.to_keycode();

                if let Some(key) = state.get_selected_key_mut() {
                    key.keycode = new_keycode.clone();
                    state.mark_dirty();
                    state.refresh_layer_refs(); // Update layer reference index
                    state.set_status(format!("Updated: {new_keycode}"));
                }

                state.active_popup = Some(PopupType::KeyEditor);
                state.close_component();
                return Ok(false);
            }

            // Parameterized keycodes (LT/MT/LM/SH_T/OSM/XXX_T)
            // Parameterized keycodes (LT/MT/LM/SH_T/OSM/XXX_T)
            if start_parameterized_keycode_flow(state, &keycode) {
                return Ok(false);
            }

            // Normal keycode assignment
            // Check for transparency conflicts before assigning (need to do this before mut borrow)
            use crate::services::layer_refs::check_transparency_conflict;
            let warning_msg = check_transparency_conflict(
                state.current_layer,
                state.selected_position,
                &keycode,
                &state.layer_refs,
            );

            if let Some(key) = state.get_selected_key_mut() {
                key.keycode = keycode.clone();
                state.mark_dirty();
                state.refresh_layer_refs(); // Update layer reference index

                // Show appropriate status message
                if let Some(warning) = warning_msg {
                    state.set_status_with_style(
                        format!("Assigned: {} - {}", keycode, warning),
                        state.theme.error,
                    );
                } else {
                    state.set_status(format!("Assigned: {keycode}"));
                }
            }

            state.close_component();
        }
        KeycodePickerEvent::Cancelled => {
            // Check if we're in a tap dance form picker flow
            if let Some(form) = state.tap_dance_form_cache.take() {
                // Clear pick target
                state.tap_dance_form_pick_target = None;

                // Restore form without changes
                state.active_component = Some(ActiveComponent::TapDanceForm(form));
                state.active_popup = Some(PopupType::TapDanceForm);
                state.set_status("Picker cancelled");
                return Ok(false);
            }

            state.close_component();
            state.set_status("Cancelled");
        }
    }
    Ok(false)
}

/// Handle category picker events
pub fn handle_category_picker_event(
    state: &mut AppState,
    event: crate::tui::CategoryPickerEvent,
) -> Result<bool> {
    match event {
        crate::tui::CategoryPickerEvent::CategorySelected(category_id) => {
            // Apply category based on context
            match state.category_picker_context {
                Some(crate::tui::CategoryPickerContext::IndividualKey) => {
                    if let Some(key) = state.get_selected_key_mut() {
                        key.category_id.clone_from(&category_id);
                        state.mark_dirty();

                        if let Some(id) = category_id {
                            state.set_status(format!("Assigned key category '{id}'"));
                        } else {
                            state.set_status("Removed key category");
                        }
                    }
                }
                Some(crate::tui::CategoryPickerContext::Layer) => {
                    if let Some(layer) = state.layout.layers.get_mut(state.current_layer) {
                        layer.category_id.clone_from(&category_id);
                        state.mark_dirty();

                        if let Some(id) = category_id {
                            state.set_status(format!("Assigned layer category '{id}'"));
                        } else {
                            state.set_status("Removed layer category");
                        }
                    }
                }
                Some(crate::tui::CategoryPickerContext::MultiKeySelection) => {
                    // Apply category to all selected keys on current layer
                    if let Some(layer) = state.layout.layers.get_mut(state.current_layer) {
                        let mut count = 0;
                        for pos in &state.selected_keys {
                            if let Some(key) = layer.keys.iter_mut().find(|k| k.position == *pos) {
                                key.category_id.clone_from(&category_id);
                                count += 1;
                            }
                        }

                        if count > 0 {
                            state.mark_dirty();
                            if let Some(id) = &category_id {
                                state
                                    .set_status(format!("Applied category '{id}' to {count} keys"));
                            } else {
                                state.set_status(format!("Removed category from {count} keys"));
                            }
                        }
                    }
                }
                None => {
                    state.set_error("No category context set");
                }
            }

            state.close_component();
            state.category_picker_context = None;
        }
        crate::tui::CategoryPickerEvent::Cancelled => {
            state.close_component();
            state.category_picker_context = None;
            state.set_status("Cancelled");
        }
    }
    Ok(false)
}
