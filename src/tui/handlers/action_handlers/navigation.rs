// Navigation action handlers

use crate::tui::AppState;
use crate::models::{Position, VisualLayoutMapping};
use crate::tui::SelectionMode;
use anyhow::Result;

/// Calculate all positions within a rectangle defined by two corner positions.
fn calculate_rectangle_selection(
    start: Position,
    end: Position,
    mapping: &VisualLayoutMapping,
) -> Vec<Position> {
    let min_row = start.row.min(end.row);
    let max_row = start.row.max(end.row);
    let min_col = start.col.min(end.col);
    let max_col = start.col.max(end.col);

    let mut selected = Vec::new();
    for row in min_row..=max_row {
        for col in min_col..=max_col {
            let pos = Position::new(row, col);
            // Only include positions that exist in the keyboard mapping
            if mapping.is_valid_position(pos) {
                selected.push(pos);
            }
        }
    }
    selected
}

/// Handle navigate up action
pub fn handle_navigate_up(state: &mut AppState) -> Result<bool> {
    if let Some(new_pos) = state.mapping.find_position_up(state.selected_position) {
        state.selected_position = new_pos;
        // Update rectangle selection if active
        if let Some(SelectionMode::Rectangle { start }) = state.selection_mode {
            state.selected_keys =
                calculate_rectangle_selection(start, new_pos, &state.mapping);
        }
        state.clear_error();
    }
    Ok(false)
}

/// Handle navigate down action
pub fn handle_navigate_down(state: &mut AppState) -> Result<bool> {
    if let Some(new_pos) = state.mapping.find_position_down(state.selected_position) {
        state.selected_position = new_pos;
        if let Some(SelectionMode::Rectangle { start }) = state.selection_mode {
            state.selected_keys =
                calculate_rectangle_selection(start, new_pos, &state.mapping);
        }
        state.clear_error();
    }
    Ok(false)
}

/// Handle navigate left action
pub fn handle_navigate_left(state: &mut AppState) -> Result<bool> {
    if let Some(new_pos) = state.mapping.find_position_left(state.selected_position) {
        state.selected_position = new_pos;
        if let Some(SelectionMode::Rectangle { start }) = state.selection_mode {
            state.selected_keys =
                calculate_rectangle_selection(start, new_pos, &state.mapping);
        }
        state.clear_error();
    }
    Ok(false)
}

/// Handle navigate right action
pub fn handle_navigate_right(state: &mut AppState) -> Result<bool> {
    if let Some(new_pos) = state.mapping.find_position_right(state.selected_position) {
        state.selected_position = new_pos;
        if let Some(SelectionMode::Rectangle { start }) = state.selection_mode {
            state.selected_keys =
                calculate_rectangle_selection(start, new_pos, &state.mapping);
        }
        state.clear_error();
    }
    Ok(false)
}

/// Handle jump to first key action
pub fn handle_jump_to_first(_state: &mut AppState) -> Result<bool> {
    // Not yet implemented
    Ok(false)
}

/// Handle jump to last key action
pub fn handle_jump_to_last(_state: &mut AppState) -> Result<bool> {
    // Not yet implemented
    Ok(false)
}

/// Handle next layer action
pub fn handle_next_layer(state: &mut AppState) -> Result<bool> {
    if state.layout.layers.is_empty() {
        return Ok(false);
    }
    
    // Cycle forward: if at last layer, wrap to 0
    if state.current_layer < state.layout.layers.len() - 1 {
        state.current_layer += 1;
    } else {
        state.current_layer = 0;
    }
    state.set_status(format!("Layer {}", state.current_layer));
    state.clear_error();
    Ok(false)
}

/// Handle previous layer action
pub fn handle_previous_layer(state: &mut AppState) -> Result<bool> {
    if state.layout.layers.is_empty() {
        return Ok(false);
    }
    
    // Cycle backward: if at layer 0, wrap to last layer
    if state.current_layer > 0 {
        state.current_layer -= 1;
    } else {
        state.current_layer = state.layout.layers.len() - 1;
    }
    state.set_status(format!("Layer {}", state.current_layer));
    state.clear_error();
    Ok(false)
}
