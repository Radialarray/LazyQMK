//! Layer manager input handler (Component trait pattern)

use anyhow::Result;
use crossterm::event;

use crate::tui::component::Component;
use crate::tui::{ActiveComponent, AppState, LayerManagerEvent};

/// Handle input for layer manager (Component trait pattern)
pub fn handle_layer_manager_input(state: &mut AppState, key: event::KeyEvent) -> Result<bool> {
    // Extract the component from active_component
    let Some(ActiveComponent::LayerManager(mut manager)) = state.active_component.take() else {
        // Component not found - this shouldn't happen
        state.set_error("Layer manager component not found");
        state.active_popup = None;
        return Ok(false);
    };

    // Handle input with the component
    let event = manager.handle_input(key);

    // Process the event if one was emitted
    if let Some(event) = event {
        match event {
            LayerManagerEvent::LayerAdded { layer } => {
                // Add the new layer
                state.layout.layers.push(layer);
                state.mark_dirty();
                state.refresh_layer_refs(); // Update layer reference index
                state.set_status(format!(
                    "Layer '{}' created",
                    state.layout.layers.last().unwrap().name
                ));

                // Update component with new layers
                manager.set_layers(state.layout.layers.clone());
            }
            LayerManagerEvent::LayerDeleted { index } => {
                // Delete layer (only if not the last one)
                if state.layout.layers.len() > 1 {
                    state.layout.layers.remove(index);

                    // Renumber remaining layers
                    for (i, layer) in state.layout.layers.iter_mut().enumerate() {
                        layer.number = i as u8;
                    }

                    // Adjust current layer if needed
                    if state.current_layer >= state.layout.layers.len() {
                        state.current_layer = state.layout.layers.len() - 1;
                    } else if state.current_layer == index {
                        // If we deleted the current layer, stay at the same index
                        // (which now points to the next layer)
                        state.current_layer = index.min(state.layout.layers.len() - 1);
                    } else if state.current_layer > index {
                        // If we deleted a layer before the current one, adjust index
                        state.current_layer -= 1;
                    }

                    state.mark_dirty();
                    state.refresh_layer_refs(); // Update layer reference index
                    state.set_status("Layer deleted");

                    // Update component with new layers
                    manager.set_layers(state.layout.layers.clone());
                } else {
                    state.set_error("Cannot delete the last layer");
                }
            }
            LayerManagerEvent::LayerRenamed { index, name } => {
                // Rename layer
                if let Some(layer) = state.layout.layers.get_mut(index) {
                    layer.name = name.clone();
                    state.mark_dirty();
                    state.set_status(format!("Layer renamed to '{name}'"));

                    // Update component with new layers
                    manager.set_layers(state.layout.layers.clone());
                }
            }
            LayerManagerEvent::LayerReordered { from, to } => {
                // Reorder layers (swap)
                state.layout.layers.swap(from, to);

                // Renumber layers
                for (i, layer) in state.layout.layers.iter_mut().enumerate() {
                    layer.number = i as u8;
                }

                // Adjust current_layer if needed
                if state.current_layer == from {
                    state.current_layer = to;
                } else if state.current_layer == to {
                    state.current_layer = from;
                }

                state.mark_dirty();
                state.set_status(format!("Layer moved to position {to}"));

                // Update component with new layers
                manager.set_layers(state.layout.layers.clone());
            }
            LayerManagerEvent::LayerDuplicated {
                source_index,
                layer,
            } => {
                // Duplicate layer
                state.layout.layers.push(layer);
                state.mark_dirty();
                state.set_status(format!(
                    "Duplicated layer {} as '{}'",
                    source_index,
                    state.layout.layers.last().unwrap().name
                ));

                // Update component with new layers
                manager.set_layers(state.layout.layers.clone());
            }
            LayerManagerEvent::LayerKeysCopied { from, to, keys } => {
                // Copy keys from source to target
                if let Some(target) = state.layout.layers.get_mut(to) {
                    for (pos, keycode, color, category) in keys {
                        if let Some(key) = target.keys.iter_mut().find(|k| k.position == pos) {
                            key.keycode = keycode;
                            key.color_override = color;
                            key.category_id = category;
                        }
                    }
                    state.mark_dirty();
                    state.set_status(format!("Copied layer {from} to layer {to}"));

                    // Update component with new layers
                    manager.set_layers(state.layout.layers.clone());
                }
            }
            LayerManagerEvent::LayersSwapped {
                layer1,
                layer2,
                keys1,
                keys2,
            } => {
                // Swap keys between two layers
                // Apply keys2 to layer1
                if let Some(layer) = state.layout.layers.get_mut(layer1) {
                    for (pos, keycode, color, category) in &keys2 {
                        if let Some(key) = layer.keys.iter_mut().find(|k| k.position == *pos) {
                            key.keycode = keycode.clone();
                            key.color_override = *color;
                            key.category_id = category.clone();
                        }
                    }
                }

                // Apply keys1 to layer2
                if let Some(layer) = state.layout.layers.get_mut(layer2) {
                    for (pos, keycode, color, category) in &keys1 {
                        if let Some(key) = layer.keys.iter_mut().find(|k| k.position == *pos) {
                            key.keycode = keycode.clone();
                            key.color_override = *color;
                            key.category_id = category.clone();
                        }
                    }
                }

                state.mark_dirty();
                state.set_status(format!("Swapped layers {layer1} and {layer2}"));

                // Update component with new layers
                manager.set_layers(state.layout.layers.clone());
            }
            LayerManagerEvent::LayerColorsToggled { index, enabled } => {
                // Toggle layer colors
                if let Some(layer) = state.layout.layers.get_mut(index) {
                    layer.layer_colors_enabled = enabled;
                    state.mark_dirty();
                    state.set_status(if enabled {
                        format!("Layer {index} colors enabled")
                    } else {
                        format!("Layer {index} colors disabled")
                    });

                    // Update component with new layers
                    manager.set_layers(state.layout.layers.clone());
                }
            }
            LayerManagerEvent::LayerSwitched { index } => {
                // Switch to selected layer
                state.current_layer = index;
                state.active_popup = None;
                state.active_component = None;
                state.set_status(format!("Switched to layer {index}"));
                return Ok(false);
            }
            LayerManagerEvent::Cancelled => {
                state.set_status("Cancelled");
            }
            LayerManagerEvent::Closed => {
                // Close the component
                state.active_popup = None;
                state.active_component = None;
                state.set_status("Layer manager closed");
                return Ok(false);
            }
        }
    }

    // Put the component back in active_component
    state.active_component = Some(ActiveComponent::LayerManager(manager));

    Ok(false)
}
