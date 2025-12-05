//! Category manager input handler

use anyhow::Result;
use crossterm::event::{self, KeyCode, KeyModifiers};

use crate::tui::category_manager::ManagerMode;
use crate::tui::{AppState, ColorPickerContext, ColorPickerState, PopupType};

/// Handle input for category manager
#[allow(clippy::too_many_lines)]
pub fn handle_category_manager_input(state: &mut AppState, key: event::KeyEvent) -> Result<bool> {
    match &state.category_manager_state.mode.clone() {
        ManagerMode::Browsing => {
            match key.code {
                KeyCode::Esc => {
                    state.active_popup = None;
                    state.set_status("Category manager closed");
                    Ok(false)
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    state
                        .category_manager_state
                        .select_previous(state.layout.categories.len());
                    Ok(false)
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    state
                        .category_manager_state
                        .select_next(state.layout.categories.len());
                    Ok(false)
                }
                KeyCode::Char('n') => {
                    // Start creating new category (T107)
                    state.category_manager_state.start_creating();
                    state.set_status("Enter category name (kebab-case, e.g., 'navigation')");
                    Ok(false)
                }
                KeyCode::Char('r') => {
                    // Start renaming (T109)
                    let selected_idx = state.category_manager_state.selected;
                    if let Some(category) = state.layout.categories.get(selected_idx) {
                        let category_clone = category.clone();
                        state.category_manager_state.start_renaming(&category_clone);
                        state.set_status("Enter new category name");
                    } else {
                        state.set_error("No category selected");
                    }
                    Ok(false)
                }
                KeyCode::Char('c') => {
                    // Change color (T110)
                    let selected_idx = state.category_manager_state.selected;
                    if let Some(category) = state.layout.categories.get(selected_idx) {
                        let color = category.color;
                        state.color_picker_state = ColorPickerState::with_color(color);
                        state.color_picker_context = Some(ColorPickerContext::Category);
                        state.active_popup = Some(PopupType::ColorPicker);
                        state.set_status("Set category color");
                    } else {
                        state.set_error("No category selected");
                    }
                    Ok(false)
                }
                KeyCode::Char('d') => {
                    // Start delete confirmation (T111)
                    let selected_idx = state.category_manager_state.selected;
                    if let Some(category) = state.layout.categories.get(selected_idx) {
                        let category_clone = category.clone();
                        state.category_manager_state.start_deleting(&category_clone);
                        state.set_status("Confirm deletion - y: Yes, n: No");
                    } else {
                        state.set_error("No category selected");
                    }
                    Ok(false)
                }
                KeyCode::Char('L') if key.modifiers.contains(KeyModifiers::SHIFT) => {
                    // Assign category to layer (T113)
                    let selected_idx = state.category_manager_state.selected;
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
                    Ok(false)
                }
                _ => Ok(false),
            }
        }
        ManagerMode::CreatingName { .. } | ManagerMode::Renaming { .. } => {
            // Handle text input
            match key.code {
                KeyCode::Esc => {
                    state.category_manager_state.cancel();
                    state.set_status("Cancelled");
                    Ok(false)
                }
                KeyCode::Enter => {
                    // Process the input
                    if let Some(input) = state.category_manager_state.get_input() {
                        let input = input.to_string();

                        match &state.category_manager_state.mode {
                            ManagerMode::CreatingName { .. } => {
                                // Generate ID from name (T107)
                                let id = input.to_lowercase().replace(' ', "-");

                                // Check if ID already exists
                                if state.layout.categories.iter().any(|c| c.id == id) {
                                    state.set_error("Category with this ID already exists");
                                    return Ok(false);
                                }

                                // Move to color selection (T108)
                                state.category_manager_state.mode =
                                    ManagerMode::CreatingColor { name: input };
                                state.color_picker_state = ColorPickerState::new();
                                state.color_picker_context = Some(ColorPickerContext::Category);
                                state.active_popup = Some(PopupType::ColorPicker);
                                state.set_status("Select color for new category");
                            }
                            ManagerMode::Renaming { category_id, .. } => {
                                // Update category name (T109)
                                if let Some(category) = state
                                    .layout
                                    .categories
                                    .iter_mut()
                                    .find(|c| &c.id == category_id)
                                {
                                    if let Err(e) = category.set_name(&input) {
                                        state.set_error(format!("Invalid name: {e}"));
                                        return Ok(false);
                                    }
                                    state.mark_dirty();
                                    state.category_manager_state.cancel();
                                    state.set_status(format!("Category renamed to '{input}'"));
                                }
                            }
                            _ => {}
                        }
                    }
                    Ok(false)
                }
                KeyCode::Char(c) => {
                    if let Some(input) = state.category_manager_state.get_input_mut() {
                        input.push(c);
                    }
                    Ok(false)
                }
                KeyCode::Backspace => {
                    if let Some(input) = state.category_manager_state.get_input_mut() {
                        input.pop();
                    }
                    Ok(false)
                }
                _ => Ok(false),
            }
        }
        ManagerMode::ConfirmingDelete { category_id } => {
            match key.code {
                KeyCode::Char('y' | 'Y') => {
                    // Delete category (T111, T112)
                    let category_id = category_id.clone();

                    // Remove category
                    state.layout.categories.retain(|c| c.id != category_id);

                    // Clean up references in keys (T112)
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
                    state.category_manager_state.cancel();

                    // Adjust selection if needed
                    if state.category_manager_state.selected >= state.layout.categories.len()
                        && state.category_manager_state.selected > 0
                    {
                        state.category_manager_state.selected -= 1;
                    }

                    state.set_status("Category deleted");
                    Ok(false)
                }
                KeyCode::Char('n' | 'N' | '\x1b') => {
                    state.category_manager_state.cancel();
                    state.set_status("Deletion cancelled");
                    Ok(false)
                }
                _ => Ok(false),
            }
        }
        ManagerMode::CreatingColor { name: _ } => {
            // Color picker is handled by the color picker handler
            // We just need to handle the completion
            // This will be managed by returning from the color picker
            Ok(false)
        }
    }
}
