// Category assignment action handlers

use crate::tui::{ActiveComponent, AppState, CategoryPickerContext, PopupType};
use anyhow::Result;

/// Handle assign category to key action
pub fn handle_assign_category_to_key(state: &mut AppState) -> Result<bool> {
    // Check if in selection mode with selected keys
    if state.selection_mode.is_some() && !state.selected_keys.is_empty() {
        // Multi-key selection mode
        let picker = crate::tui::CategoryPicker::new();
        state.active_component = Some(ActiveComponent::CategoryPicker(picker));
        state.category_picker_context = Some(CategoryPickerContext::MultiKeySelection);
        state.active_popup = Some(PopupType::CategoryPicker);
        state.set_status("Select category for selected keys - Enter to apply");
        Ok(false)
    } else if state.get_selected_key().is_some() {
        // Individual key mode
        let picker = crate::tui::CategoryPicker::new();
        state.active_component = Some(ActiveComponent::CategoryPicker(picker));
        state.category_picker_context = Some(CategoryPickerContext::IndividualKey);
        state.active_popup = Some(PopupType::CategoryPicker);
        state.set_status("Select category for key - Enter to apply");
        Ok(false)
    } else {
        state.set_error("No key selected");
        Ok(false)
    }
}

/// Handle assign category to layer action
pub fn handle_assign_category_to_layer(state: &mut AppState) -> Result<bool> {
    // Assign category to layer (Shift+L or Ctrl+L)
    let picker = crate::tui::CategoryPicker::new();
    state.active_component = Some(ActiveComponent::CategoryPicker(picker));
    state.category_picker_context = Some(CategoryPickerContext::Layer);
    state.active_popup = Some(PopupType::CategoryPicker);
    state.set_status("Select category for layer - Enter to apply");
    Ok(false)
}
