//! Picker handlers for keycode, color, category, layout, layer, tap-keycode, and modifier popups.
//!
//! Extracted from src/tui/handlers/popups.rs to reduce file size. Each
//! `handle_*_input` function dispatches input to the corresponding picker
//! component via the `Component` or `ContextualComponent` trait pattern.

use anyhow::Result;
use crossterm::event::{self, KeyCode, KeyModifiers};

use crate::services::LayoutService;
use crate::tui::build_log::BuildLogEvent;
use crate::tui::color_picker::ColorPickerEvent;
use crate::tui::component::{Component, ContextualComponent};
use crate::tui::editor::key_editor;
use crate::tui::keycode_picker::{self};
use crate::tui::metadata_editor;
use crate::tui::{ActiveComponent, AppState, LayoutVariantPickerEvent, PopupType};

/// Handle key events for the color picker popup. The color picker uses
/// the `Component` trait pattern, so we delegate input to the component
/// and process the resulting event.
pub fn handle_color_picker_event(state: &mut AppState, key: event::KeyEvent) -> Result<bool> {
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
                                    if let Some(key) =
                                        layer.keys.iter_mut().find(|k| k.position == *pos)
                                    {
                                        key.color_override = Some(color);
                                        count += 1;
                                    }
                                }

                                if count > 0 {
                                    state.mark_dirty();
                                    state.set_status(format!(
                                        "Set color to {} for {count} keys",
                                        color.to_hex()
                                    ));
                                }
                            }
                        }
                        crate::tui::component::ColorPickerContext::OverlayRippleFixedColor => {
                            state.layout.rgb_overlay_ripple.fixed_color = color;
                            state.mark_dirty();
                            state.set_status(format!(
                                "Set ripple fixed color to {}",
                                color.to_hex()
                            ));
                        }
                    }

                    // Close the color picker
                    state.close_component();
                    if state.return_to_settings_after_picker {
                        state.return_to_settings_after_picker = false;
                        state.open_settings_manager();
                    }
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
                                    if let Some(key) =
                                        layer.keys.iter_mut().find(|k| k.position == *pos)
                                    {
                                        key.color_override = None;
                                        count += 1;
                                    }
                                }

                                if count > 0 {
                                    state.mark_dirty();
                                    state.set_status(format!(
                                        "Cleared color for {count} keys (using layer default)"
                                    ));
                                }
                            }
                        }
                        crate::tui::component::ColorPickerContext::OverlayRippleFixedColor => {
                            let default_color = crate::models::RgbColor::new(0, 255, 255);
                            state.layout.rgb_overlay_ripple.fixed_color = default_color;
                            state.mark_dirty();
                            state.set_status("Reset ripple fixed color to default cyan");
                        }
                    }

                    // Close the color picker
                    state.close_component();
                    if state.return_to_settings_after_picker {
                        state.return_to_settings_after_picker = false;
                        state.open_settings_manager();
                    }
                }
                ColorPickerEvent::Cancelled => {
                    state.close_component();
                    if state.return_to_settings_after_picker {
                        state.return_to_settings_after_picker = false;
                        state.open_settings_manager();
                    }
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

/// Handle events from `BuildLog` component
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

/// Handle events from the `LayerPicker` component
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
                    crate::tui::handlers::popups::parameterized::handle_parameter_collected(
                        state, layer_ref,
                    );
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
                        crate::tui::handlers::popups::parameterized::handle_parameter_collected(
                            state,
                            kc.code.clone(),
                        );
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
                crate::tui::handlers::popups::parameterized::handle_parameter_collected(
                    state, mod_string,
                );
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
pub fn is_basic_keycode(code: &str) -> bool {
    // Basic keycodes: KC_A-Z, KC_0-9, KC_F1-24, navigation, symbols, etc.
    // Exclude: layer keycodes, mod-taps, parameterized keycodes
    !code.contains('(') && !code.contains('@')
}

/// Check if a keycode is a basic keycode OR a simple layer keycode.
///
/// Allows: `KC_A`, MO(1), TG(2), TO(3), TT(1), OSL(2).
/// Rejects: LT(1, KC_A), MT(MOD_LCTL, KC_A), MO(@layer_id), etc.
pub fn is_basic_or_layer_keycode(code: &str) -> bool {
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
