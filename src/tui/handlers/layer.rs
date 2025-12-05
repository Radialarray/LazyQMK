//! Layer manager input handler

use anyhow::Result;
use crossterm::event::{self, KeyCode, KeyModifiers};

use crate::tui::layer_manager::ManagerMode;
use crate::tui::AppState;

/// Handle input for layer manager
#[allow(clippy::too_many_lines, clippy::cognitive_complexity)]
pub fn handle_layer_manager_input(state: &mut AppState, key: event::KeyEvent) -> Result<bool> {
    match &state.layer_manager_state.mode.clone() {
        ManagerMode::Browsing => {
            match key.code {
                KeyCode::Esc => {
                    state.active_popup = None;
                    state.set_status("Layer manager closed");
                    Ok(false)
                }
                KeyCode::Up if key.modifiers.contains(KeyModifiers::SHIFT) => {
                    // Move layer up (reorder)
                    let selected = state.layer_manager_state.selected;
                    if selected > 0 && state.layout.layers.len() > 1 {
                        state.layout.layers.swap(selected, selected - 1);
                        state.layer_manager_state.selected = selected - 1;
                        // Renumber layers
                        for (i, layer) in state.layout.layers.iter_mut().enumerate() {
                            layer.number = i as u8;
                        }
                        state.mark_dirty();
                        state.set_status(format!("Layer moved up to position {}", selected - 1));
                    }
                    Ok(false)
                }
                KeyCode::Down if key.modifiers.contains(KeyModifiers::SHIFT) => {
                    // Move layer down (reorder)
                    let selected = state.layer_manager_state.selected;
                    if selected < state.layout.layers.len() - 1 {
                        state.layout.layers.swap(selected, selected + 1);
                        state.layer_manager_state.selected = selected + 1;
                        // Renumber layers
                        for (i, layer) in state.layout.layers.iter_mut().enumerate() {
                            layer.number = i as u8;
                        }
                        state.mark_dirty();
                        state.set_status(format!("Layer moved down to position {}", selected + 1));
                    }
                    Ok(false)
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    state
                        .layer_manager_state
                        .select_previous(state.layout.layers.len());
                    Ok(false)
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    state
                        .layer_manager_state
                        .select_next(state.layout.layers.len());
                    Ok(false)
                }
                KeyCode::Enter => {
                    // Go to selected layer
                    state.current_layer = state.layer_manager_state.selected;
                    state.active_popup = None;
                    state.set_status(format!("Switched to layer {}", state.current_layer));
                    Ok(false)
                }
                KeyCode::Char('n') => {
                    // Start creating new layer
                    state.layer_manager_state.start_creating();
                    state.set_status("Enter layer name");
                    Ok(false)
                }
                KeyCode::Char('r') => {
                    // Start renaming
                    let selected_idx = state.layer_manager_state.selected;
                    if let Some(layer) = state.layout.layers.get(selected_idx) {
                        let layer_clone = layer.clone();
                        state.layer_manager_state.start_renaming(&layer_clone);
                        state.set_status("Enter new layer name");
                    } else {
                        state.set_error("No layer selected");
                    }
                    Ok(false)
                }
                KeyCode::Char('v') => {
                    // Toggle layer colors
                    let selected_idx = state.layer_manager_state.selected;
                    if let Some(enabled) = state.layout.toggle_layer_colors(selected_idx) {
                        state.mark_dirty();
                        state.set_status(if enabled {
                            format!("Layer {selected_idx} colors enabled")
                        } else {
                            format!("Layer {selected_idx} colors disabled")
                        });
                    }
                    Ok(false)
                }
                KeyCode::Char('d') => {
                    // Start delete confirmation
                    if state.layout.layers.len() <= 1 {
                        state.set_error("Cannot delete the last layer");
                    } else {
                        state.layer_manager_state.start_deleting();
                        state.set_status("Confirm deletion - y: Yes, n: No");
                    }
                    Ok(false)
                }
                KeyCode::Char('D') if key.modifiers.contains(KeyModifiers::SHIFT) => {
                    // Start duplicating
                    let selected_idx = state.layer_manager_state.selected;
                    if let Some(layer) = state.layout.layers.get(selected_idx) {
                        let layer_clone = layer.clone();
                        state.layer_manager_state.start_duplicating(&layer_clone);
                        state.set_status("Enter name for duplicate layer");
                    }
                    Ok(false)
                }
                KeyCode::Char('c') => {
                    // Start copy to another layer
                    if state.layout.layers.len() <= 1 {
                        state.set_error("Need at least 2 layers to copy");
                    } else {
                        state
                            .layer_manager_state
                            .start_copy_to(state.layout.layers.len());
                        state.set_status("Select target layer");
                    }
                    Ok(false)
                }
                KeyCode::Char('s') => {
                    // Start swap with another layer
                    if state.layout.layers.len() <= 1 {
                        state.set_error("Need at least 2 layers to swap");
                    } else {
                        state
                            .layer_manager_state
                            .start_swapping(state.layout.layers.len());
                        state.set_status("Select layer to swap with");
                    }
                    Ok(false)
                }
                _ => Ok(false),
            }
        }
        ManagerMode::CreatingName { .. }
        | ManagerMode::Renaming { .. }
        | ManagerMode::Duplicating { .. } => {
            // Handle text input
            match key.code {
                KeyCode::Esc => {
                    state.layer_manager_state.cancel();
                    state.set_status("Cancelled");
                    Ok(false)
                }
                KeyCode::Enter => {
                    // Process the input
                    if let Some(input) = state.layer_manager_state.get_input() {
                        let input = input.to_string();

                        if input.trim().is_empty() {
                            state.set_error("Layer name cannot be empty");
                            return Ok(false);
                        }

                        match &state.layer_manager_state.mode {
                            ManagerMode::CreatingName { .. } => {
                                // Create new layer
                                use crate::models::layer::Layer;
                                use crate::models::ColorPalette;
                                let new_index = state.layout.layers.len();

                                // Use the color palette's default layer color (Gray-500)
                                let palette = ColorPalette::load().unwrap_or_default();
                                let default_color = palette.default_layer_color();

                                match Layer::new(new_index as u8, &input, default_color) {
                                    Ok(mut new_layer) => {
                                        // Copy key positions from first layer (with transparent keycodes)
                                        if let Some(first_layer) = state.layout.layers.first() {
                                            for key in &first_layer.keys {
                                                use crate::models::layer::KeyDefinition;
                                                new_layer.add_key(KeyDefinition::new(
                                                    key.position,
                                                    "KC_TRNS",
                                                ));
                                            }
                                        }

                                        state.layout.layers.push(new_layer);
                                        state.layer_manager_state.selected = new_index;
                                        state.mark_dirty();
                                        state.layer_manager_state.cancel();
                                        state.set_status(format!("Layer '{input}' created"));
                                    }
                                    Err(e) => {
                                        state.set_error(format!("Failed to create layer: {e}"));
                                    }
                                }
                            }
                            ManagerMode::Renaming { layer_index, .. } => {
                                // Update layer name
                                let layer_index = *layer_index;
                                if let Some(layer) = state.layout.layers.get_mut(layer_index) {
                                    layer.name = input.clone();
                                    state.mark_dirty();
                                    state.layer_manager_state.cancel();
                                    state.set_status(format!("Layer renamed to '{input}'"));
                                }
                            }
                            ManagerMode::Duplicating { source_index, .. } => {
                                // Duplicate layer with new name
                                use crate::models::layer::Layer;
                                let source_index = *source_index;
                                let new_index = state.layout.layers.len();

                                if let Some(source) = state.layout.layers.get(source_index) {
                                    match Layer::new(new_index as u8, &input, source.default_color)
                                    {
                                        Ok(mut new_layer) => {
                                            // Copy all keys from source
                                            for key in &source.keys {
                                                use crate::models::layer::KeyDefinition;
                                                let mut new_key =
                                                    KeyDefinition::new(key.position, &key.keycode);
                                                new_key.color_override = key.color_override;
                                                new_key.category_id = key.category_id.clone();
                                                new_layer.add_key(new_key);
                                            }
                                            // Copy layer settings
                                            new_layer.layer_colors_enabled =
                                                source.layer_colors_enabled;
                                            new_layer.category_id = source.category_id.clone();

                                            state.layout.layers.push(new_layer);
                                            state.layer_manager_state.selected = new_index;
                                            state.mark_dirty();
                                            state.layer_manager_state.cancel();
                                            state.set_status(format!("Duplicated as '{input}'"));
                                        }
                                        Err(e) => {
                                            state.set_error(format!(
                                                "Failed to duplicate layer: {e}"
                                            ));
                                        }
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                    Ok(false)
                }
                KeyCode::Char(c) => {
                    if let Some(input) = state.layer_manager_state.get_input_mut() {
                        input.push(c);
                    }
                    Ok(false)
                }
                KeyCode::Backspace => {
                    if let Some(input) = state.layer_manager_state.get_input_mut() {
                        input.pop();
                    }
                    Ok(false)
                }
                _ => Ok(false),
            }
        }
        ManagerMode::ConfirmingDelete { layer_index } => {
            let layer_index = *layer_index;
            match key.code {
                KeyCode::Char('y' | 'Y') => {
                    // Delete layer (only if not the last one)
                    if state.layout.layers.len() > 1 {
                        state.layout.layers.remove(layer_index);

                        // Renumber remaining layers
                        for (i, layer) in state.layout.layers.iter_mut().enumerate() {
                            layer.number = i as u8;
                        }

                        // Adjust current layer if needed
                        if state.current_layer >= state.layout.layers.len() {
                            state.current_layer = state.layout.layers.len() - 1;
                        }

                        // Adjust selection if needed
                        if state.layer_manager_state.selected >= state.layout.layers.len() {
                            state.layer_manager_state.selected = state.layout.layers.len() - 1;
                        }

                        state.mark_dirty();
                        state.layer_manager_state.cancel();
                        state.set_status("Layer deleted");
                    } else {
                        state.set_error("Cannot delete the last layer");
                        state.layer_manager_state.cancel();
                    }
                    Ok(false)
                }
                KeyCode::Char('n' | 'N') | KeyCode::Esc => {
                    state.layer_manager_state.cancel();
                    state.set_status("Deletion cancelled");
                    Ok(false)
                }
                _ => Ok(false),
            }
        }
        ManagerMode::CopyingTo {
            source_index,
            target_selected,
        } => {
            let source_index = *source_index;
            let target_selected = *target_selected;
            match key.code {
                KeyCode::Esc => {
                    state.layer_manager_state.cancel();
                    state.set_status("Copy cancelled");
                    Ok(false)
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    state
                        .layer_manager_state
                        .select_target_previous(state.layout.layers.len());
                    Ok(false)
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    state
                        .layer_manager_state
                        .select_target_next(state.layout.layers.len());
                    Ok(false)
                }
                KeyCode::Enter => {
                    // Copy all keys from source to target
                    if let Some(source) = state.layout.layers.get(source_index) {
                        let keys_to_copy: Vec<_> = source
                            .keys
                            .iter()
                            .map(|k| {
                                (
                                    k.position,
                                    k.keycode.clone(),
                                    k.color_override,
                                    k.category_id.clone(),
                                )
                            })
                            .collect();

                        if let Some(target) = state.layout.layers.get_mut(target_selected) {
                            for (pos, keycode, color, category) in keys_to_copy {
                                if let Some(key) =
                                    target.keys.iter_mut().find(|k| k.position == pos)
                                {
                                    key.keycode = keycode;
                                    key.color_override = color;
                                    key.category_id = category;
                                }
                            }
                            state.mark_dirty();
                            state.layer_manager_state.cancel();
                            state.set_status(format!(
                                "Copied layer {source_index} to layer {target_selected}"
                            ));
                        }
                    }
                    Ok(false)
                }
                _ => Ok(false),
            }
        }
        ManagerMode::Swapping {
            source_index,
            target_selected,
        } => {
            let source_index = *source_index;
            let target_selected = *target_selected;
            match key.code {
                KeyCode::Esc => {
                    state.layer_manager_state.cancel();
                    state.set_status("Swap cancelled");
                    Ok(false)
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    state
                        .layer_manager_state
                        .select_target_previous(state.layout.layers.len());
                    Ok(false)
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    state
                        .layer_manager_state
                        .select_target_next(state.layout.layers.len());
                    Ok(false)
                }
                KeyCode::Enter => {
                    // Swap all keys between source and target
                    // We need to collect both sets of keys first to avoid borrow issues
                    let source_keys: Vec<_>;
                    let target_keys: Vec<_>;

                    if let (Some(source), Some(target)) = (
                        state.layout.layers.get(source_index),
                        state.layout.layers.get(target_selected),
                    ) {
                        source_keys = source
                            .keys
                            .iter()
                            .map(|k| {
                                (
                                    k.position,
                                    k.keycode.clone(),
                                    k.color_override,
                                    k.category_id.clone(),
                                )
                            })
                            .collect();
                        target_keys = target
                            .keys
                            .iter()
                            .map(|k| {
                                (
                                    k.position,
                                    k.keycode.clone(),
                                    k.color_override,
                                    k.category_id.clone(),
                                )
                            })
                            .collect();
                    } else {
                        return Ok(false);
                    }

                    // Apply target keys to source layer
                    if let Some(source) = state.layout.layers.get_mut(source_index) {
                        for (pos, keycode, color, category) in &target_keys {
                            if let Some(key) = source.keys.iter_mut().find(|k| k.position == *pos) {
                                key.keycode = keycode.clone();
                                key.color_override = *color;
                                key.category_id = category.clone();
                            }
                        }
                    }

                    // Apply source keys to target layer
                    if let Some(target) = state.layout.layers.get_mut(target_selected) {
                        for (pos, keycode, color, category) in &source_keys {
                            if let Some(key) = target.keys.iter_mut().find(|k| k.position == *pos) {
                                key.keycode = keycode.clone();
                                key.color_override = *color;
                                key.category_id = category.clone();
                            }
                        }
                    }

                    state.mark_dirty();
                    state.layer_manager_state.cancel();
                    state.set_status(format!(
                        "Swapped layers {source_index} and {target_selected}"
                    ));
                    Ok(false)
                }
                _ => Ok(false),
            }
        }
    }
}
