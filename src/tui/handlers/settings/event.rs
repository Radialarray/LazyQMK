//! Settings manager event handler (dispatches events to apply logic).

use anyhow::Result;

use crate::tui::settings_manager::SettingsManagerEvent;
use crate::tui::{ActiveComponent, AppState};

/// Handle settings manager events
pub(super) fn handle_settings_manager_event(
    state: &mut AppState,
    event: SettingsManagerEvent,
) -> Result<bool> {
    match event {
        SettingsManagerEvent::SettingsUpdated => {
            // Apply the settings from the manager's current state
            super::apply::apply_settings(state)?;

            // Return to browsing mode
            if let Some(ActiveComponent::SettingsManager(ref mut manager)) = state.active_component
            {
                manager.state_mut().cancel();
            }

            Ok(false)
        }
        SettingsManagerEvent::Cancelled => {
            state.close_component();
            state.set_status("Settings closed");
            Ok(false)
        }
        SettingsManagerEvent::Closed => {
            state.close_component();
            state.set_status("Settings closed");
            Ok(false)
        }
    }
}
