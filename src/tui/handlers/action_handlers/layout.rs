// Layout action handlers

use crate::tui::AppState;
use anyhow::Result;

/// Handle switch layout variant action
pub fn handle_switch_layout_variant(state: &mut AppState) -> Result<bool> {
    let qmk_path = match &state.config.paths.qmk_firmware {
        Some(path) => path.clone(),
        None => {
            state.set_error("QMK firmware path not configured");
            return Ok(false);
        }
    };

    let keyboard = state.layout.metadata.keyboard.as_deref().unwrap_or("");
    let base_keyboard = AppState::extract_base_keyboard(keyboard);

    if let Err(e) = state.open_layout_variant_picker(&qmk_path, &base_keyboard) {
        state.set_error(format!("Failed to load layouts: {e}"));
        return Ok(false);
    }

    state.set_status("Select layout variant - ↑↓: Navigate, Enter: Apply, Esc: Cancel");
    Ok(false)
}
