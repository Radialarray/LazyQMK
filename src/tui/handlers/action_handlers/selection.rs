// Selection action handlers

use crate::tui::{AppState, SelectionMode};
use anyhow::Result;

/// Handle toggle selection mode action
pub fn handle_toggle_selection_mode(state: &mut AppState) -> Result<bool> {
    if state.selection_mode.is_some() {
        // Exit selection mode
        state.selection_mode = None;
        state.selected_keys.clear();
        state.set_status("Selection mode cancelled");
    } else {
        // Enter selection mode with current key selected
        state.selection_mode = Some(SelectionMode::Normal);
        state.selected_keys.clear();
        state.selected_keys.push(state.selected_position);
        state.set_status("Selection mode - Space: toggle key, y: copy, d: cut, Esc: cancel");
    }
    Ok(false)
}

/// Handle start rectangle select action
pub fn handle_start_rectangle_select(state: &mut AppState) -> Result<bool> {
    if state.selection_mode.is_some() {
        // Start rectangle selection from current position
        state.selection_mode = Some(SelectionMode::Rectangle {
            start: state.selected_position,
        });
        state.selected_keys.clear();
        state.selected_keys.push(state.selected_position);
        state.set_status("Rectangle select - move to opposite corner, Enter to confirm");
    } else {
        // Enter rectangle selection mode
        state.selection_mode = Some(SelectionMode::Rectangle {
            start: state.selected_position,
        });
        state.selected_keys.clear();
        state.selected_keys.push(state.selected_position);
        state.set_status("Rectangle select - move to opposite corner, Enter to confirm");
    }
    Ok(false)
}

/// Handle swap keys action
pub fn handle_swap_keys(state: &mut AppState) -> Result<bool> {
    match &state.selection_mode {
        None => {
            // Enter swap mode - select first key
            state.selection_mode = Some(SelectionMode::Swap {
                first: state.selected_position,
            });
            state.selected_keys.clear();
            state.selected_keys.push(state.selected_position);
            state.set_status("Swap mode - navigate to second key, Shift+W to swap, Esc to cancel");
            Ok(false)
        }
        Some(SelectionMode::Swap { first }) => {
            // Already in swap mode - perform the swap between first and current position
            let first_pos = *first;
            let second_pos = state.selected_position;

            // Can't swap a key with itself
            if first_pos == second_pos {
                state.set_status("Cannot swap a key with itself");
                return Ok(false);
            }

            // Get the current layer
            let layer = if let Some(layer) = state.layout.layers.get_mut(state.current_layer) {
                layer
            } else {
                state.set_status("Invalid layer");
                return Ok(false);
            };

            // Find indices of both keys
            let first_idx = layer.keys.iter().position(|k| k.position == first_pos);
            let second_idx = layer.keys.iter().position(|k| k.position == second_pos);

            match (first_idx, second_idx) {
                (Some(idx1), Some(idx2)) => {
                    // Swap all properties: keycode, color_override, category_id
                    let first_key = layer.keys[idx1].clone();
                    let second_key = layer.keys[idx2].clone();

                    layer.keys[idx1].keycode = second_key.keycode;
                    layer.keys[idx1].color_override = second_key.color_override;
                    layer.keys[idx1].category_id = second_key.category_id;

                    layer.keys[idx2].keycode = first_key.keycode;
                    layer.keys[idx2].color_override = first_key.color_override;
                    layer.keys[idx2].category_id = first_key.category_id;

                    // Mark dirty and exit swap mode
                    state.dirty = true;
                    state.selection_mode = None;
                    state.selected_keys.clear();
                    state.set_status("Keys swapped");
                }
                _ => {
                    state.set_status("Invalid key positions");
                }
            }

            Ok(false)
        }
        Some(_) => {
            // In another mode (Normal or Rectangle) - enter swap mode
            state.selection_mode = Some(SelectionMode::Swap {
                first: state.selected_position,
            });
            state.selected_keys.clear();
            state.selected_keys.push(state.selected_position);
            state.set_status("Swap mode - navigate to second key, Shift+W to swap, Esc to cancel");
            Ok(false)
        }
    }
}
