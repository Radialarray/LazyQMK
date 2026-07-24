//! Input entry point and key-position selection mode.

use anyhow::Result;
use crossterm::event;

use crate::tui::settings_manager::{ManagerMode, SettingsManagerContext};
use crate::tui::{ActiveComponent, AppState};

/// Handle input for settings manager using Component trait pattern
pub fn handle_settings_manager_input(state: &mut AppState, key: event::KeyEvent) -> Result<bool> {
    // Check if we're in key position selection mode
    let is_selecting_key_position =
        if let Some(ActiveComponent::SettingsManager(ref manager)) = state.active_component {
            matches!(
                manager.state().mode,
                ManagerMode::SelectingKeyPosition { .. }
            )
        } else {
            false
        };

    // Handle key position selection mode separately
    if is_selecting_key_position {
        return handle_key_position_selection(state, key);
    }

    // Check if we're in browsing mode and handle Enter key specially
    let is_browsing =
        if let Some(ActiveComponent::SettingsManager(ref manager)) = state.active_component {
            matches!(manager.state().mode, ManagerMode::Browsing)
        } else {
            false
        };

    // Handle Enter in browsing mode - triggers editing submodes or opens nested pickers
    if is_browsing && key.code == event::KeyCode::Enter {
        return super::browsing::handle_browsing_enter(state);
    }

    // Extract the component and handle input with context
    if let Some(ActiveComponent::SettingsManager(ref mut manager)) = state.active_component {
        // Build context from current state
        let context = SettingsManagerContext {
            rgb_enabled: state.layout.rgb_enabled,
            rgb_brightness: state.layout.rgb_brightness,
            rgb_timeout_ms: state.layout.rgb_timeout_ms,
            uncolored_key_behavior: state.layout.uncolored_key_behavior,
            idle_effect_settings: state.layout.idle_effect_settings.clone(),
            overlay_ripple_settings: state.layout.rgb_overlay_ripple.clone(),
            tap_hold_settings: state.layout.tap_hold_settings.clone(),
            config: state.config.clone(),
            layout: state.layout.clone(),
        };

        // Handle input and check for events
        if let Some(event) = manager.handle_input_with_context(key, &context) {
            return super::event::handle_settings_manager_event(state, event);
        }
    }
    Ok(false)
}

/// Handle key position selection mode
fn handle_key_position_selection(state: &mut AppState, key: event::KeyEvent) -> Result<bool> {
    match key.code {
        event::KeyCode::Esc => {
            // Cancel selection and return to browsing
            if let Some(ActiveComponent::SettingsManager(ref mut manager)) = state.active_component
            {
                manager.state_mut().cancel();
                state.set_status("Key position selection cancelled");
            }
            Ok(false)
        }
        event::KeyCode::Enter => {
            // Apply the selected position
            if let Some(ActiveComponent::SettingsManager(ref manager)) = state.active_component {
                if let ManagerMode::SelectingKeyPosition { setting, .. } = &manager.state().mode {
                    let setting = *setting;
                    // Apply the key position
                    super::apply::apply_combo_key_position(state, setting);
                }
            }
            // Return to browsing mode
            if let Some(ActiveComponent::SettingsManager(ref mut manager)) = state.active_component
            {
                manager.state_mut().cancel();
            }
            Ok(false)
        }
        event::KeyCode::Up | event::KeyCode::Char('k') => {
            // Navigate keyboard up
            if let Some(new_pos) = state.mapping.find_position_up(state.selected_position) {
                state.selected_position = new_pos;
            }
            Ok(false)
        }
        event::KeyCode::Down | event::KeyCode::Char('j') => {
            // Navigate keyboard down
            if let Some(new_pos) = state.mapping.find_position_down(state.selected_position) {
                state.selected_position = new_pos;
            }
            Ok(false)
        }
        event::KeyCode::Left | event::KeyCode::Char('h') => {
            // Navigate keyboard left
            if let Some(new_pos) = state.mapping.find_position_left(state.selected_position) {
                state.selected_position = new_pos;
            }
            Ok(false)
        }
        event::KeyCode::Right | event::KeyCode::Char('l') => {
            // Navigate keyboard right
            if let Some(new_pos) = state.mapping.find_position_right(state.selected_position) {
                state.selected_position = new_pos;
            }
            Ok(false)
        }
        _ => Ok(false),
    }
}

/// Handle combo action selection mode (j/k navigate options, Enter applies, Esc cancels).
///
/// Mirrors `handle_key_position_selection` for arrow-key navigation but delegates
/// navigation to the manager's action handlers (which know how to step through
/// the [`crate::models::ComboAction`] enum).
#[allow(dead_code)] // Exposed via the settings manager input dispatch
pub fn handle_selecting_action_input(state: &mut AppState, key: event::KeyEvent) -> Result<bool> {
    if let Some(ActiveComponent::SettingsManager(ref mut manager)) = state.active_component {
        // Build context (needed by other handlers, kept for parity).
        let context = SettingsManagerContext {
            rgb_enabled: state.layout.rgb_enabled,
            rgb_brightness: state.layout.rgb_brightness,
            rgb_timeout_ms: state.layout.rgb_timeout_ms,
            uncolored_key_behavior: state.layout.uncolored_key_behavior,
            idle_effect_settings: state.layout.idle_effect_settings.clone(),
            overlay_ripple_settings: state.layout.rgb_overlay_ripple.clone(),
            tap_hold_settings: state.layout.tap_hold_settings.clone(),
            config: state.config.clone(),
            layout: state.layout.clone(),
        };
        if let Some(event) = manager.handle_input_with_context(key, &context) {
            return super::event::handle_settings_manager_event(state, event);
        }
    }
    Ok(false)
}
