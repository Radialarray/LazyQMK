//! Main UI input handler.

use anyhow::Result;
use crossterm::event;

use crate::shortcuts::ShortcutRegistry;
use crate::tui::AppState;

/// Handle input for main UI
pub fn handle_main_input(state: &mut AppState, key: event::KeyEvent) -> Result<bool> {
    let registry = ShortcutRegistry::new();
    
    if let Some(action) = registry.lookup("main", key) {
        super::dispatch_action(state, action)
    } else {
        // No action mapped - ignore key
        Ok(false)
    }
}
