//! Category manager input handler

use anyhow::Result;
use crossterm::event::{self, KeyModifiers};

use crate::models::Category;
use crate::tui::category_manager::{CategoryManagerEvent, ManagerMode};
use crate::tui::component::Component;
use crate::tui::{ActiveComponent, AppState};

/// Handle input for category manager
pub fn handle_category_manager_input(state: &mut AppState, key: event::KeyEvent) -> Result<bool> {
    // Extract the component from active_component
    let Some(ActiveComponent::CategoryManager(mut manager)) = state.active_component.take() else {
        // Component not found - this shouldn't happen
        state.set_error("Category manager component not found");
        state.active_popup = None;
        return Ok(false);
    };

    // Update component with latest categories before handling input
    manager.set_categories(state.layout.categories.clone());

    // Handle special case: Shift+L to assign category to layer (not handled by component)
    if key.code == event::KeyCode::Char('L') && key.modifiers.contains(KeyModifiers::SHIFT) {
        if matches!(manager.state().mode, ManagerMode::Browsing) {
            let selected_idx = manager.state().selected;
            if let Some(category) = state.layout.categories.get(selected_idx) {
                let category_id = category.id.clone();
                let category_name = category.name.clone();
                if let Some(layer) = state.layout.layers.get_mut(state.current_layer) {
                    layer.category_id = Some(category_id);
                    state.mark_dirty();
                    state.set_status(format!(
                        "Layer {} assigned to category '{}'",
                        state.current_layer, category_name
                    ));
                }
            } else {
                state.set_error("No category selected");
            }
            // Sync state and put component back
            state.category_manager_state = manager.state().clone();
            state.active_component = Some(ActiveComponent::CategoryManager(manager));
            return Ok(false);
        }
    }

    // Handle special case: 'c' to change color in browsing mode (opens color picker)
    if key.code == event::KeyCode::Char('c')
        && matches!(manager.state().mode, ManagerMode::Browsing)
    {
        let selected_idx = manager.state().selected;
        if let Some(category) = state.layout.categories.get(selected_idx) {
            let color = category.color;
            // Sync state before opening color picker (which will replace the component)
            state.category_manager_state = manager.state().clone();
            state.open_color_picker(crate::tui::component::ColorPickerContext::Category, color);
            state.set_status("Set category color");
        } else {
            state.set_error("No category selected");
            state.category_manager_state = manager.state().clone();
            state.active_component = Some(ActiveComponent::CategoryManager(manager));
        }
        return Ok(false);
    }

    // Handle input through the component
    let event = manager.handle_input(key);

    // Process events emitted by the component
    if let Some(event) = event {
        match event {
            CategoryManagerEvent::CategoryAdded { id, name, color } => {
                // Create new category (T107, T108)
                if let Ok(category) = Category::new(&id, &name, color) {
                    state.layout.categories.push(category);
                    state.mark_dirty();
                    state.set_status(format!("Created category '{name}'"));
                } else {
                    state.set_error("Failed to create category");
                }
                // Update component with new categories
                manager.set_categories(state.layout.categories.clone());
            }
            CategoryManagerEvent::CategoryDeleted(category_id) => {
                // Delete category and clean up references (T111, T112)
                state.layout.categories.retain(|c| c.id != category_id);

                // Clean up references in layers and keys
                for layer in &mut state.layout.layers {
                    if layer.category_id.as_ref() == Some(&category_id) {
                        layer.category_id = None;
                    }
                    for key in &mut layer.keys {
                        if key.category_id.as_ref() == Some(&category_id) {
                            key.category_id = None;
                        }
                    }
                }

                state.mark_dirty();
                state.set_status("Category deleted");
                // Update component with new categories
                manager.set_categories(state.layout.categories.clone());
            }
            CategoryManagerEvent::CategoryUpdated { id, name, color } => {
                // Update category (T109, T110)
                let mut status_message = None;
                let mut error_message = None;
                let mut dirty = false;
                
                if let Some(category) = state.layout.categories.iter_mut().find(|c| c.id == id) {
                    if let Some(new_name) = name {
                        if let Err(e) = category.set_name(&new_name) {
                            error_message = Some(format!("Invalid name: {e}"));
                        } else {
                            dirty = true;
                            status_message = Some(format!("Category renamed to '{new_name}'"));
                        }
                    }
                    if let Some(new_color) = color {
                        category.set_color(new_color);
                        dirty = true;
                        status_message = Some("Category color updated".to_string());
                    }
                }
                
                // Now update state outside the borrow
                if dirty {
                    state.mark_dirty();
                }
                if let Some(msg) = status_message {
                    state.set_status(msg);
                }
                if let Some(err) = error_message {
                    state.set_error(err);
                }
                
                // Update component with modified categories
                manager.set_categories(state.layout.categories.clone());
            }
            CategoryManagerEvent::Cancelled => {
                state.set_status("Cancelled");
            }
            CategoryManagerEvent::Closed => {
                // Close the component
                state.active_popup = None;
                state.active_component = None;
                state.category_manager_state.reset();
                state.set_status("Category manager closed");
                return Ok(false);
            }
        }
    }

    // Special case: check if we need to open color picker for category creation
    if let ManagerMode::CreatingColor { name } = &manager.state().mode {
        let name = name.clone();
        let id = name.to_lowercase().replace(' ', "-");

        // Check if ID already exists
        if state.layout.categories.iter().any(|c| c.id == id) {
            state.set_error("Category with this ID already exists");
            manager.state_mut().cancel();
        } else {
            // Sync state before opening color picker (which will replace the component)
            state.category_manager_state = manager.state().clone();
            state.open_color_picker(
                crate::tui::component::ColorPickerContext::Category,
                crate::models::RgbColor::new(255, 255, 255),
            );
            state.set_status("Select color for new category");
            return Ok(false);
        }
    }

    // Sync state and put the component back into active_component
    state.category_manager_state = manager.state().clone();
    state.active_component = Some(ActiveComponent::CategoryManager(manager));
    Ok(false)
}
