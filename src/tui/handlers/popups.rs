//! Popup input handlers extracted from main TUI module.

use anyhow::Result;
use crossterm::event::{self, KeyCode, KeyModifiers};

use crate::keycode_db::{KeycodeDb, ParamType};
use crate::services::LayoutService;
use crate::tui::{
    build_log::BuildLogEvent,
    color_picker::ColorPickerEvent,
    component::{Component, ContextualComponent},
    key_editor, keycode_picker,
    keycode_picker::KeycodePickerEvent,
    metadata_editor, onboarding_wizard, ActiveComponent, AppState, LayoutVariantPickerEvent,
    PopupType,
};

/// Extracts the tap dance name from a TD(name) keycode.
/// Returns None if the keycode is not a valid TD() pattern.
fn extract_td_name(keycode: &str) -> Option<String> {
    if let Some(stripped) = keycode.strip_prefix("TD(") {
        if let Some(name) = stripped.strip_suffix(')') {
            if !name.is_empty() {
                return Some(name.to_string());
            }
        }
    }
    None
}

/// Start a parameterized keycode flow using KeycodeDb param metadata (data-driven approach)
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
fn handle_parameter_collected(state: &mut AppState, param_value: String) {
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
fn handle_keycode_picker_event(state: &mut AppState, event: KeycodePickerEvent) -> Result<bool> {
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
fn handle_category_picker_event(
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
                                state.set_status(format!("Applied category '{id}' to {count} keys"));
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

/// Handle color picker input using Component trait pattern
fn handle_color_picker_event(state: &mut AppState, key: event::KeyEvent) -> Result<bool> {
    // Get mutable reference to the ColorPicker component
    if let Some(ActiveComponent::ColorPicker(ref mut picker)) = state.active_component {
        // Handle input and get event
        if let Some(event) = picker.handle_input(key) {
            // Process the event
            match event {
                ColorPickerEvent::ColorSelected(color) => {
                    // Get context before closing component
                    let context = picker.get_context();

                    // Apply color based on context
                    match context {
                        crate::tui::component::ColorPickerContext::IndividualKey => {
                            if let Some(key) = state.get_selected_key_mut() {
                                key.color_override = Some(color);
                                state.mark_dirty();
                                state.set_status(format!("Set key color to {}", color.to_hex()));
                            }
                        }
                        crate::tui::component::ColorPickerContext::LayerDefault => {
                            if let Some(layer) = state.layout.layers.get_mut(state.current_layer) {
                                layer.default_color = color;
                                state.mark_dirty();
                                state.set_status(format!(
                                    "Set layer default color to {}",
                                    color.to_hex()
                                ));
                            }
                        }
                        crate::tui::component::ColorPickerContext::Category => {
                            use crate::tui::category_manager::ManagerMode;
                            use crate::tui::CategoryManager;

                            match &state.category_manager_state.mode {
                                ManagerMode::CreatingColor { name } => {
                                    let name = name.clone();
                                    let id = name.to_lowercase().replace(' ', "-");

                                    if let Ok(category) =
                                        crate::models::Category::new(&id, &name, color)
                                    {
                                        state.layout.categories.push(category);
                                        state.mark_dirty();
                                        state.set_status(format!("Created category '{name}'"));
                                    } else {
                                        state.set_error("Failed to create category");
                                    }

                                    state.category_manager_state.cancel();
                                }
                                ManagerMode::Browsing => {
                                    let selected_idx = state.category_manager_state.selected;
                                    if let Some(category) =
                                        state.layout.categories.get_mut(selected_idx)
                                    {
                                        let name = category.name.clone();
                                        category.set_color(color);
                                        state.mark_dirty();
                                        state.set_status(format!("Updated color for '{name}'"));
                                    }
                                }
                                _ => {
                                    state.set_error("Invalid category manager state");
                                }
                            }

                            // Recreate CategoryManager component with updated categories and synced state
                            let mut manager = CategoryManager::new(state.layout.categories.clone());
                            *manager.state_mut() = state.category_manager_state.clone();
                            state.active_component =
                                Some(ActiveComponent::CategoryManager(manager));
                            state.active_popup = Some(PopupType::CategoryManager);
                        }
                        crate::tui::component::ColorPickerContext::MultiKeySelection => {
                            // Apply color to all selected keys on current layer
                            if let Some(layer) = state.layout.layers.get_mut(state.current_layer) {
                                let mut count = 0;
                                for pos in &state.selected_keys {
                                    if let Some(key) = layer.keys.iter_mut().find(|k| k.position == *pos) {
                                        key.color_override = Some(color);
                                        count += 1;
                                    }
                                }
                                
                                if count > 0 {
                                    state.mark_dirty();
                                    state.set_status(format!("Set color to {} for {count} keys", color.to_hex()));
                                }
                            }
                        }
                    }

                    // Close the color picker
                    state.close_component();
                }
                ColorPickerEvent::ColorCleared => {
                    // Get context before closing component
                    let context = picker.get_context();

                    match context {
                        crate::tui::component::ColorPickerContext::IndividualKey => {
                            if let Some(key) = state.get_selected_key_mut() {
                                key.color_override = None;
                                state.mark_dirty();
                                state.set_status("Cleared key color (using layer default)");
                            }
                        }
                        crate::tui::component::ColorPickerContext::LayerDefault => {
                            let default_color = crate::models::RgbColor::new(255, 255, 255);
                            if let Some(layer) = state.layout.layers.get_mut(state.current_layer) {
                                layer.default_color = default_color;
                                state.mark_dirty();
                                state.set_status("Reset layer color to white");
                            }
                        }
                        crate::tui::component::ColorPickerContext::Category => {
                            state.set_error("Categories must have a color");
                            return Ok(false); // Don't close
                        }
                        crate::tui::component::ColorPickerContext::MultiKeySelection => {
                            // Clear color override for all selected keys on current layer
                            if let Some(layer) = state.layout.layers.get_mut(state.current_layer) {
                                let mut count = 0;
                                for pos in &state.selected_keys {
                                    if let Some(key) = layer.keys.iter_mut().find(|k| k.position == *pos) {
                                        key.color_override = None;
                                        count += 1;
                                    }
                                }
                                
                                if count > 0 {
                                    state.mark_dirty();
                                    state.set_status(format!("Cleared color for {count} keys (using layer default)"));
                                }
                            }
                        }
                    }

                    // Close the color picker
                    state.close_component();
                }
                ColorPickerEvent::Cancelled => {
                    state.close_component();
                    state.set_status("Cancelled");
                }
            }
        }
    }

    Ok(false)
}

/// Handle input for build log viewer
pub fn handle_build_log_input(state: &mut AppState, key: event::KeyEvent) -> Result<bool> {
    // Handle clipboard copy separately as it's not part of the component
    if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
        if let Some(build_state) = &state.build_state {
            let log_text = build_state
                .log_lines
                .iter()
                .map(|(_, message)| message.as_str())
                .collect::<Vec<_>>()
                .join("\n");

            match arboard::Clipboard::new().and_then(|mut clipboard| clipboard.set_text(log_text)) {
                Ok(()) => state.set_status("Build log copied to clipboard"),
                Err(e) => state.set_error(format!("Failed to copy to clipboard: {e}")),
            }
        } else {
            state.set_error("No build log available");
        }
        return Ok(false);
    }

    // Use ContextualComponent trait pattern
    if let Some(ActiveComponent::BuildLog(ref mut log)) = state.active_component {
        if let Some(ref build_state) = state.build_state {
            if let Some(event) = log.handle_input(key, build_state) {
                return handle_build_log_event(state, event);
            }
        }
    }

    Ok(false)
}

/// Handle events from BuildLog component
fn handle_build_log_event(state: &mut AppState, event: BuildLogEvent) -> Result<bool> {
    match event {
        BuildLogEvent::Closed => {
            state.close_component();
            state.set_status("Build log closed");
        }
    }

    Ok(false)
}

/// Handle input for help overlay
pub fn handle_help_overlay_input(state: &mut AppState, key: event::KeyEvent) -> Result<bool> {
    // Use Component trait pattern
    if let Some(ActiveComponent::HelpOverlay(ref mut help)) = state.active_component {
        if let Some(event) = help.handle_input(key) {
            return handle_help_overlay_event(state, event);
        }
    }
    Ok(false)
}

/// Handle help overlay events
fn handle_help_overlay_event(
    state: &mut AppState,
    event: crate::tui::help_overlay::HelpOverlayEvent,
) -> Result<bool> {
    use crate::tui::help_overlay::HelpOverlayEvent;

    match event {
        HelpOverlayEvent::Closed => {
            state.close_component();
            state.set_status("Press ? for help");
        }
    }
    Ok(false)
}

/// Handle input for metadata editor
pub fn handle_metadata_editor_input(state: &mut AppState, key: event::KeyEvent) -> Result<bool> {
    // Use Component trait pattern
    if let Some(ActiveComponent::MetadataEditor(ref mut editor)) = state.active_component {
        if let Some(event) = editor.handle_input(key) {
            return handle_metadata_editor_event(state, event);
        }
    }
    Ok(false)
}

/// Handle metadata editor events
fn handle_metadata_editor_event(
    state: &mut AppState,
    event: metadata_editor::MetadataEditorEvent,
) -> Result<bool> {
    use metadata_editor::MetadataEditorEvent;

    match event {
        MetadataEditorEvent::MetadataUpdated {
            name,
            description,
            author,
            tags,
            name_changed,
        } => {
            // Apply changes to layout
            state.layout.metadata.name = name.clone();
            state.layout.metadata.description = description;
            state.layout.metadata.author = author;
            state.layout.metadata.tags = tags;
            state.layout.metadata.modified = chrono::Utc::now();
            state.mark_dirty();

            // If name changed and we have a source file, rename it
            if name_changed {
                if let Some(ref old_path) = state.source_path {
                    match LayoutService::rename_file_if_needed(old_path, &name) {
                        Ok(Some(new_path)) => {
                            state.source_path = Some(new_path);
                            state.set_status(format!("Layout renamed to '{name}'"));
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
            state.active_component = None;
            Ok(false)
        }
        MetadataEditorEvent::Cancelled => {
            state.active_popup = None;
            state.active_component = None;
            state.set_status("Metadata editing cancelled");
            Ok(false)
        }
        MetadataEditorEvent::Closed => {
            state.active_popup = None;
            state.active_component = None;
            Ok(false)
        }
    }
}

/// Handle input for layout variant picker
pub fn handle_layout_picker_input(state: &mut AppState, key: event::KeyEvent) -> Result<bool> {
    use crate::tui::ActiveComponent;

    // Extract the component from active_component
    let mut component = match state.active_component.take() {
        Some(ActiveComponent::LayoutVariantPicker(picker)) => picker,
        _ => {
            // Component not found - close popup
            state.active_popup = None;
            return Ok(false);
        }
    };

    // Handle input and get event
    if let Some(event) = component.handle_input(key) {
        // Process the event
        handle_layout_variant_picker_event(state, event)?;
    } else {
        // No event - restore component and continue
        state.active_component = Some(ActiveComponent::LayoutVariantPicker(component));
    }

    Ok(false)
}

/// Handle events from the layout variant picker component
fn handle_layout_variant_picker_event(
    state: &mut AppState,
    event: LayoutVariantPickerEvent,
) -> Result<()> {
    match event {
        LayoutVariantPickerEvent::LayoutSelected(selected) => {
            // User selected a layout - rebuild geometry and mapping
            match state.rebuild_geometry(&selected) {
                Ok(()) => {
                    if state.return_to_settings_after_picker {
                        state.return_to_settings_after_picker = false;
                        state.open_settings_manager();
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
                        state.open_settings_manager();
                    }
                }
            }
            state.active_component = None;
        }
        LayoutVariantPickerEvent::Cancelled => {
            // User cancelled
            if state.return_to_settings_after_picker {
                state.return_to_settings_after_picker = false;
                state.open_settings_manager();
                state.set_status("Layout selection cancelled");
            } else {
                state.active_popup = None;
                state.set_status("Layout selection cancelled");
            }
            state.active_component = None;
        }
    }
    Ok(())
}

/// Handle input for layer picker (for layer-switching keycodes)
pub fn handle_layer_picker_input(state: &mut AppState, key: event::KeyEvent) -> Result<bool> {
    // Extract the component from active_component
    let mut component = match state.active_component.take() {
        Some(ActiveComponent::LayerPicker(picker)) => picker,
        _ => {
            // Component not found - close popup
            state.active_popup = None;
            return Ok(false);
        }
    };

    // Handle input and get event
    if let Some(event) = component.handle_input(key, &state.layout.layers) {
        // Process the event
        handle_layer_picker_event(state, event)?;
    } else {
        // No event - restore component and continue
        state.active_component = Some(ActiveComponent::LayerPicker(component));
    }

    Ok(false)
}

/// Handle events from the LayerPicker component
fn handle_layer_picker_event(
    state: &mut AppState,
    event: crate::tui::layer_picker::LayerPickerEvent,
) -> Result<()> {
    use crate::tui::layer_picker::LayerPickerEvent;

    match event {
        LayerPickerEvent::LayerSelected(selected_idx) => {
            // Get the selected layer
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
                        state.refresh_layer_refs(); // Update layer reference index
                        state.set_status(format!("Updated: {new_keycode}"));
                    }

                    state.active_popup = Some(PopupType::KeyEditor);
                    state.active_component = None;
                    return Ok(());
                }

                // Data-driven parameterized keycode flow
                if state.pending_keycode.keycode_template.is_some() {
                    handle_parameter_collected(state, layer_ref);
                    return Ok(());
                }

                // Regular layer keycode (MO, TG, TO, etc.) - assign directly
                // Get the keycode prefix from the layer picker component state (via pending keycode or default)
                // Since the legacy layer_picker_state is removed, we build the keycode directly
                // For parameterized keycodes, handle_parameter_collected() handles it above
                // For regular layer keycodes like MO/TO/TG/etc, they're also handled via parameterized flow
                // This code path is only reached if somehow a non-parameterized layer selection happens
                // which shouldn't occur in practice, but we handle it gracefully
                if let Some(key) = state.get_selected_key_mut() {
                    // Default to MO() for momentary layer switch
                    let keycode = format!("MO({})", layer_ref);
                    key.keycode = keycode.clone();
                    state.mark_dirty();
                    state.refresh_layer_refs(); // Update layer reference index
                    state.set_status(format!("Assigned: {keycode}"));
                }
            }

            state.active_popup = None;
            state.active_component = None;
        }
        LayerPickerEvent::Cancelled => {
            // Check if we were editing a combo part
            if state.key_editor_state.combo_edit.is_some() {
                state.key_editor_state.combo_edit = None;
                state.active_popup = Some(PopupType::KeyEditor);
                state.active_component = None;
                state.set_status("Cancelled - back to key editor");
                return Ok(());
            }
            // Check if this was part of a parameterized keycode flow
            if state.pending_keycode.keycode_template.is_some() {
                state.pending_keycode.reset();
            }
            state.active_popup = None;
            state.active_component = None;
            state.set_status("Layer selection cancelled");
        }
    }
    Ok(())
}

/// Handle input for tap keycode picker (second stage of `LT/MT/SH_T`)
pub fn handle_tap_keycode_picker_input(state: &mut AppState, key: event::KeyEvent) -> Result<bool> {
    // Use the component-based picker
    if let Some(ActiveComponent::KeycodePicker(ref mut picker)) = state.active_component {
        match key.code {
            KeyCode::Esc => {
                // Cancel the whole parameterized keycode flow
                state.pending_keycode.reset();
                state.active_popup = None;
                state.close_component();
                state.set_status("Cancelled");
                Ok(false)
            }
            KeyCode::Enter => {
                // Get the selected keycode from the component
                let keycodes = keycode_picker::get_filtered_keycodes_from_context(
                    picker.state(),
                    &state.keycode_db,
                );
                if let Some(kc) = keycodes.get(picker.state().selected) {
                    // Only allow basic keycodes for tap action (no parameterized keycodes)
                    if is_basic_keycode(&kc.code) {
                        // Data-driven approach: collect the parameter and continue the flow
                        handle_parameter_collected(state, kc.code.clone());
                        return Ok(false);
                    }
                    state.set_error("Only basic keycodes allowed for tap action");
                }
                Ok(false)
            }
            // Delegate all other input to the component
            _ => {
                // Let the picker handle navigation
                let _ = picker.handle_input(key, &state.keycode_db);
                Ok(false)
            }
        }
    } else {
        Ok(false)
    }
}

/// Handle input for modifier picker (for MT/LM keycodes)
#[allow(clippy::too_many_lines)]
pub fn handle_modifier_picker_input(state: &mut AppState, key: event::KeyEvent) -> Result<bool> {
    // Use Component trait pattern
    if let Some(ActiveComponent::ModifierPicker(ref mut picker)) = state.active_component {
        if let Some(event) = picker.handle_input(key) {
            return handle_modifier_picker_event(state, event);
        }
    }
    Ok(false)
}

/// Handle modifier picker events
fn handle_modifier_picker_event(
    state: &mut AppState,
    event: crate::tui::modifier_picker::ModifierPickerEvent,
) -> Result<bool> {
    use crate::tui::modifier_picker::ModifierPickerEvent;

    match event {
        ModifierPickerEvent::ModifiersSelected(modifiers) => {
            let mod_string = modifiers.join(" | ");

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
                state.close_component();
                return Ok(false);
            }

            // Data-driven parameterized keycode flow
            if state.pending_keycode.keycode_template.is_some() {
                handle_parameter_collected(state, mod_string);
            } else {
                // No parameterized flow - unexpected state
                state.pending_keycode.reset();
                state.close_component();
                state.set_error(
                    "Unexpected state: modifier selected but no parameterized flow active",
                );
            }
        }
        ModifierPickerEvent::Cancelled => {
            // Check if we were editing a combo part
            if state.key_editor_state.combo_edit.is_some() {
                state.key_editor_state.combo_edit = None;
                state.active_popup = Some(PopupType::KeyEditor);
                state.close_component();
                state.set_status("Cancelled - back to key editor");
                return Ok(false);
            }

            state.pending_keycode.reset();
            state.close_component();
            state.set_status("Cancelled");
        }
    }
    Ok(false)
}

/// Check if a keycode is a basic keycode (not parameterized)
fn is_basic_keycode(code: &str) -> bool {
    // Basic keycodes: KC_A-Z, KC_0-9, KC_F1-24, navigation, symbols, etc.
    // Exclude: layer keycodes, mod-taps, parameterized keycodes
    !code.contains('(') && !code.contains('@')
}

/// Check if a keycode is a basic keycode OR a simple layer keycode (MO, TG, TO, etc.)
/// Allows: KC_A, MO(1), TG(2), TO(3), TT(1), OSL(2)
/// Rejects: LT(1, KC_A), MT(MOD_LCTL, KC_A), MO(@layer_id), etc.
fn is_basic_or_layer_keycode(code: &str) -> bool {
    // Basic keycodes are always allowed
    if is_basic_keycode(code) {
        return true;
    }

    // Reject @ layer references (not yet supported in tap dance hold)
    if code.contains('@') {
        return false;
    }

    // Check if it's a simple layer keycode (single parameter, layer number)
    // MO(n), TG(n), TO(n), TT(n), OSL(n) are allowed
    // LT(n, KC_X), MT(...), LM(...) etc. are NOT allowed
    if let Some(prefix) = code.split('(').next() {
        matches!(prefix, "MO" | "TG" | "TO" | "TT" | "OSL" | "DF")
            && code.matches('(').count() == 1  // Only one opening paren
            && !code.contains(',') // No comma (no second parameter)
    } else {
        false
    }
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
                state.tap_dance_form_pick_target =
                    Some(crate::tui::tap_dance_form::FormRow::Single);

                // Open keycode picker
                state.open_keycode_picker();
                state.set_status("Select single tap keycode (required)");
                return Ok(false);
            }
            TapDanceFormEvent::PickDouble => {
                // Cache the form and set pick target
                state.tap_dance_form_cache = Some(component);
                state.tap_dance_form_pick_target =
                    Some(crate::tui::tap_dance_form::FormRow::Double);

                // Open keycode picker
                state.open_keycode_picker();
                state.set_status("Select double tap keycode (required)");
                return Ok(false);
            }
            TapDanceFormEvent::PickHold => {
                // Cache the form and set pick target
                state.tap_dance_form_cache = Some(component);
                state.tap_dance_form_pick_target = Some(crate::tui::tap_dance_form::FormRow::Hold);

                // Open keycode picker
                state.open_keycode_picker();
                state.set_status("Select hold keycode (optional, Esc to skip)");
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
                        state.open_keycode_picker();
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Layout, LayoutMetadata};
    use crate::tui::key_editor::{ComboEditPart, ComboKeycodeType as ComboType};
    use crate::tui::AppState;

    fn create_test_state() -> AppState {
        let layout = Layout {
            metadata: LayoutMetadata::default(),
            layers: vec![],
            categories: vec![],
            rgb_enabled: true,
            rgb_brightness: crate::models::RgbBrightness::default(),
            rgb_saturation: crate::models::RgbSaturation::default(),
            rgb_timeout_ms: 0,
            uncolored_key_behavior: crate::models::UncoloredKeyBehavior::default(),
            idle_effect_settings: crate::models::IdleEffectSettings::default(),
            tap_hold_settings: crate::models::TapHoldSettings::default(),
            tap_dances: vec![],
        };
        let mut state = AppState::new(
            layout,
            None,
            crate::models::KeyboardGeometry::new("test", "test", 4, 12),
            crate::models::VisualLayoutMapping::default(),
            crate::config::Config::default(),
        )
        .unwrap();
        // Add a dummy key to select
        state.selected_position = crate::models::Position::new(0, 0);
        state
    }

    #[test]
    fn test_combo_edit_preserves_hold_behavior() {
        let mut state = create_test_state();

        // Setup: We are editing the TAP part of a Layer Tap (LT)
        state.key_editor_state.combo_edit = Some((
            ComboEditPart::Tap,
            ComboType::LayerTap {
                layer: "1".to_string(),
                tap_key: "KC_TRNS".to_string(),
            },
        ));

        // Action: Select a basic keycode 'KC_A'
        let event = KeycodePickerEvent::KeycodeSelected("KC_A".to_string());

        // Setup layout
        use crate::models::{KeyDefinition, Layer, Position};
        let mut layer = Layer::new(0, "Base", crate::models::RgbColor::default()).unwrap();
        layer.add_key(KeyDefinition::new(Position::new(0, 0), "KC_TRNS"));
        state.layout.layers.push(layer);
        state.current_layer = 0;
        state.selected_position = Position::new(0, 0);

        // Run handler
        let _ = handle_keycode_picker_event(&mut state, event);

        // Assert: The keycode should be LT(1, KC_A), NOT just KC_A
        let key = state.get_selected_key_mut().unwrap();
        assert_eq!(key.keycode, "LT(1, KC_A)");

        // Assert: Combo edit state is cleared
        assert!(state.key_editor_state.combo_edit.is_none());
    }

    #[test]
    fn test_combo_edit_rejects_non_basic_keycode() {
        let mut state = create_test_state();

        // Setup: Editing Tap part
        state.key_editor_state.combo_edit = Some((
            ComboEditPart::Tap,
            ComboType::LayerTap {
                layer: "1".to_string(),
                tap_key: "KC_TRNS".to_string(),
            },
        ));

        // Action: Try to select a parameterized keycode 'MO(1)'
        let event = KeycodePickerEvent::KeycodeSelected("MO(1)".to_string());

        let _ = handle_keycode_picker_event(&mut state, event);

        // Assert: Error set
        assert!(state
            .error_message
            .as_deref()
            .unwrap_or("")
            .contains("Only basic keycodes allowed"));

        // Assert: Combo edit state is PRESERVED (not cleared)
        assert!(state.key_editor_state.combo_edit.is_some());
    }

    #[test]
    fn test_mod_tap_edit_preserves_modifiers() {
        let mut state = create_test_state();

        // Setup: We are editing the TAP part of a Mod Tap (MT)
        // MT(MOD_LSFT, KC_Z)
        state.key_editor_state.combo_edit = Some((
            ComboEditPart::Tap,
            ComboType::ModTapCustom {
                modifier: "MOD_LSFT".to_string(),
                tap_key: "KC_Z".to_string(),
            },
        ));

        // Setup layout
        use crate::models::{KeyDefinition, Layer, Position};
        let mut layer = Layer::new(0, "Base", crate::models::RgbColor::default()).unwrap();
        layer.add_key(KeyDefinition::new(Position::new(0, 0), "KC_TRNS"));
        state.layout.layers.push(layer);
        state.current_layer = 0;
        state.selected_position = Position::new(0, 0);

        // Action: Select 'KC_ENTER'
        let event = KeycodePickerEvent::KeycodeSelected("KC_ENTER".to_string());

        let _ = handle_keycode_picker_event(&mut state, event);

        // Assert: The keycode should be MT(MOD_LSFT, KC_ENTER)
        let key = state.get_selected_key_mut().unwrap();
        assert_eq!(key.keycode, "MT(MOD_LSFT, KC_ENTER)");
    }

    #[test]
    fn test_td_keycode_opens_tap_dance_form() {
        let mut state = create_test_state();

        // Setup layout with a key
        use crate::models::{KeyDefinition, Layer, Position};
        let mut layer = Layer::new(0, "Base", crate::models::RgbColor::default()).unwrap();
        layer.add_key(KeyDefinition::new(Position::new(0, 0), "KC_TRNS"));
        state.layout.layers.push(layer);
        state.current_layer = 0;
        state.selected_position = Position::new(0, 0);

        // Action: Select TD() keycode from picker
        let event = KeycodePickerEvent::KeycodeSelected("TD()".to_string());

        let _ = handle_keycode_picker_event(&mut state, event);

        // Assert: Tap dance form should be open
        assert!(
            matches!(
                state.active_component,
                Some(ActiveComponent::TapDanceForm(_))
            ),
            "Expected TapDanceForm to be active, got: {:?}",
            state.active_component.as_ref().map(std::mem::discriminant)
        );
        assert_eq!(
            state.active_popup,
            Some(PopupType::TapDanceForm),
            "Expected popup to be TapDanceForm"
        );

        // Assert: Context should be FromKeycodePicker
        assert!(
            matches!(
                state.tap_dance_form_context,
                Some(crate::tui::TapDanceFormContext::FromKeycodePicker)
            ),
            "Expected context to be FromKeycodePicker, got: {:?}",
            state.tap_dance_form_context
        );

        // Assert: Pending keycode should be cleared (no further param flow)
        assert!(
            state.pending_keycode.keycode_template.is_none(),
            "Expected pending keycode to be cleared"
        );
    }

    #[test]
    fn test_td_keycode_edit_existing_opens_form() {
        use crate::models::{KeyDefinition, Layer, Position, TapDanceAction};

        let mut state = create_test_state();

        // Setup: Add a tap dance definition
        let td = TapDanceAction::new("slash", "KC_SLSH").with_double_tap("KC_BSLS");
        state.layout.add_tap_dance(td).unwrap();

        // Setup: Add a layer with a key that has this tap dance
        let mut layer = Layer::new(0, "Base", crate::models::RgbColor::default()).unwrap();
        layer.add_key(KeyDefinition::new(Position::new(0, 0), "TD(slash)"));
        state.layout.layers.push(layer);
        state.current_layer = 0;
        state.selected_position = Position::new(0, 0);

        // Action: Select TD() keycode from picker while on a key that already has TD(slash)
        let event = KeycodePickerEvent::KeycodeSelected("TD()".to_string());

        let _ = handle_keycode_picker_event(&mut state, event);

        // Assert: Tap dance form should be open
        assert!(
            matches!(
                state.active_component,
                Some(ActiveComponent::TapDanceForm(_))
            ),
            "Expected TapDanceForm to be active, got: {:?}",
            state.active_component.as_ref().map(std::mem::discriminant)
        );
        assert_eq!(state.active_popup, Some(PopupType::TapDanceForm));
        assert_eq!(
            state.tap_dance_form_context,
            Some(crate::tui::TapDanceFormContext::FromKeycodePicker)
        );
    }

    #[test]
    fn test_extract_td_name() {
        assert_eq!(extract_td_name("TD(slash)"), Some("slash".to_string()));
        assert_eq!(
            extract_td_name("TD(esc_caps)"),
            Some("esc_caps".to_string())
        );
        assert_eq!(
            extract_td_name("TD(my_dance_123)"),
            Some("my_dance_123".to_string())
        );
        assert_eq!(extract_td_name("TD()"), None); // Empty name
        assert_eq!(extract_td_name("KC_A"), None); // Not a TD
        assert_eq!(extract_td_name("TD(incomplete"), None); // Missing closing paren
    }

    #[test]
    fn test_is_basic_or_layer_keycode() {
        // Basic keycodes should be allowed
        assert!(is_basic_or_layer_keycode("KC_A"));
        assert!(is_basic_or_layer_keycode("KC_ENTER"));
        assert!(is_basic_or_layer_keycode("KC_LSFT"));

        // Simple layer keycodes should be allowed
        assert!(is_basic_or_layer_keycode("MO(1)"));
        assert!(is_basic_or_layer_keycode("TG(2)"));
        assert!(is_basic_or_layer_keycode("TO(3)"));
        assert!(is_basic_or_layer_keycode("TT(1)"));
        assert!(is_basic_or_layer_keycode("OSL(2)"));
        assert!(is_basic_or_layer_keycode("DF(0)"));

        // Complex parameterized keycodes should be rejected
        assert!(!is_basic_or_layer_keycode("LT(1, KC_SPC)"));
        assert!(!is_basic_or_layer_keycode("MT(MOD_LCTL, KC_A)"));
        assert!(!is_basic_or_layer_keycode("LM(1, MOD_LSFT)"));
        assert!(!is_basic_or_layer_keycode("LCTL_T(KC_A)"));

        // Layer references should be rejected
        assert!(!is_basic_or_layer_keycode("MO(@layer_id)"));
    }
}
