// Color management action handlers

use crate::tui::AppState;
use anyhow::Result;

/// Handle set individual key color action
pub fn handle_set_individual_key_color(state: &mut AppState) -> Result<bool> {
    // Check if in selection mode with selected keys
    if state.selection_mode.is_some() && !state.selected_keys.is_empty() {
        // Multi-key selection mode - use first key's color as initial
        if let Some(first_key) = state.layout.layers.get(state.current_layer)
            .and_then(|layer| layer.keys.iter().find(|k| state.selected_keys.contains(&k.position))) {
            let current_color = state.layout.resolve_key_color(state.current_layer, first_key);
            state.open_color_picker(
                crate::tui::component::ColorPickerContext::MultiKeySelection,
                current_color,
            );
            state.set_status("Adjust color with arrows, Tab to switch channels, Enter to apply to all selected");
        } else {
            state.set_error("No keys selected");
        }
    } else if let Some(key) = state.get_selected_key() {
        // Individual key mode
        let current_color = state.layout.resolve_key_color(state.current_layer, key);
        state.open_color_picker(
            crate::tui::component::ColorPickerContext::IndividualKey,
            current_color,
        );
        state.set_status("Adjust color with arrows, Tab to switch channels, Enter to apply");
    } else {
        state.set_error("No key selected");
    }
    Ok(false)
}

/// Handle set layer color action
pub fn handle_set_layer_color(state: &mut AppState) -> Result<bool> {
    // Set layer default color (c)
    if let Some(layer) = state.layout.layers.get(state.current_layer) {
        // Initialize color picker with current layer default color
        state.open_color_picker(
            crate::tui::component::ColorPickerContext::LayerDefault,
            layer.default_color,
        );
        state.set_status("Setting layer default color - Enter to apply");
    }
    Ok(false)
}

/// Handle toggle layer colors action
pub fn handle_toggle_layer_colors(state: &mut AppState) -> Result<bool> {
    // Toggle colors for current layer (v)
    if let Some(enabled) = state.layout.toggle_layer_colors(state.current_layer) {
        state.mark_dirty();
        let status = if enabled {
            format!("Layer {} colors enabled", state.current_layer)
        } else {
            format!("Layer {} colors disabled", state.current_layer)
        };
        state.set_status(status);
    }
    Ok(false)
}

/// Handle toggle all layer colors action
pub fn handle_toggle_all_layer_colors(state: &mut AppState) -> Result<bool> {
    // Toggle colors for all layers (Alt+V)
    let enabled = state.layout.toggle_all_layer_colors();
    state.mark_dirty();
    let status = if enabled {
        "All layer colors enabled".to_string()
    } else {
        "All layer colors disabled".to_string()
    };
    state.set_status(status);
    Ok(false)
}
