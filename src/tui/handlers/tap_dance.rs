//! Tap dance editor input handler (Component trait pattern)

use anyhow::Result;
use crossterm::event;

use crate::tui::component::Component;
use crate::tui::tap_dance_editor::TapDanceEditorEvent;
use crate::tui::{ActiveComponent, AppState, PopupType};

/// Handle input for tap dance editor (Component trait pattern)
pub fn handle_tap_dance_editor_input(state: &mut AppState, key: event::KeyEvent) -> Result<bool> {
    // Extract the component from active_component
    let Some(ActiveComponent::TapDanceEditor(mut editor)) = state.active_component.take() else {
        // Component not found - this shouldn't happen
        state.set_error("Tap dance editor component not found");
        state.active_popup = None;
        return Ok(false);
    };

    // Handle input with the component
    let event = editor.handle_input(key);

    // Process the event if one was emitted
    if let Some(event) = event {
        match event {
            TapDanceEditorEvent::Selected(name) => {
                // Validate that the tap dance exists
                if state.layout.get_tap_dance(&name).is_none() {
                    state.set_error(format!("Tap dance '{name}' not found"));
                    state.active_component = Some(ActiveComponent::TapDanceEditor(editor));
                    return Ok(false);
                }

                // Apply TD(name) to selected key
                if let Some(key) = state.get_selected_key_mut() {
                    let td_keycode = format!("TD({name})");
                    key.keycode = td_keycode.clone();
                    state.mark_dirty();
                    state.set_status(format!("Applied: {td_keycode}"));
                } else {
                    state.set_error("No key selected");
                }
                
                // Close the component
                state.active_popup = None;
                state.active_component = None;
                return Ok(false);
            }
            TapDanceEditorEvent::CreateNew => {
                // Start tap dance creation flow
                state.start_tap_dance_create();
                
                // Close tap dance editor
                state.active_popup = None;
                state.active_component = None;
                
                // Get existing tap dance names for validation
                let existing_names: Vec<String> = state.layout.tap_dances.iter()
                    .map(|td| td.name.clone())
                    .collect();
                
                // Open name entry dialog
                let name_entry = crate::tui::tap_dance_name_entry::TapDanceNameEntry::new(existing_names);
                state.active_popup = Some(PopupType::TapDanceNameEntry);
                state.active_component = Some(ActiveComponent::TapDanceNameEntry(name_entry));
                state.set_status("Enter tap dance name (alphanumeric + underscore)");
                
                return Ok(false);
            }
            TapDanceEditorEvent::Edit(index) => {
                // Start tap dance edit flow
                state.start_tap_dance_edit(index);
                
                // Close tap dance editor
                state.active_popup = None;
                state.active_component = None;
                
                // Open keycode picker for single_tap field
                state.open_keycode_picker();
                state.set_status("Edit tap dance: Select single tap keycode");
                
                return Ok(false);
            }
            TapDanceEditorEvent::Delete(name) => {
                // Remove from layout.tap_dances
                state.layout.remove_tap_dance(&name);
                state.mark_dirty();
                state.set_status(format!("Deleted tap dance '{name}'"));
                
                // Refresh editor with updated list
                editor = crate::tui::tap_dance_editor::TapDanceEditor::new(&state.layout);
            }
            TapDanceEditorEvent::Cancelled => {
                // Close the component
                state.active_popup = None;
                state.active_component = None;
                state.set_status("Tap dance editor closed");
                return Ok(false);
            }
        }
    }

    // Put the component back in active_component
    state.active_component = Some(ActiveComponent::TapDanceEditor(editor));

    Ok(false)
}
