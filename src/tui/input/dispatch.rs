//! Keyboard event dispatch.
//!
//! Routes to the popup handler or main UI handler based on current state.

use anyhow::Result;
use crossterm::event::{self, KeyCode};

use crate::tui::app_state::AppState;
use crate::tui::handlers;

/// Handle keyboard input events
pub fn handle_key_event(state: &mut AppState, key: event::KeyEvent) -> Result<bool> {
    // If error overlay is shown, allow dismissing with Enter or Esc
    if state.error_message.is_some() {
        if matches!(key.code, KeyCode::Enter | KeyCode::Esc) {
            state.clear_error();
            return Ok(false);
        }
        // Block all other input while error is shown
        return Ok(false);
    }

    // Route to popup handler if popup is active
    if state.active_popup.is_some() {
        return handlers::handle_popup_input(state, key);
    }

    // Main UI key handling
    handlers::handle_main_input(state, key)
}
