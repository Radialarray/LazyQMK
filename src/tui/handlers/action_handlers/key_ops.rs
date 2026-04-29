// Key operations action handlers

use crate::models::Position;
use crate::tui::{clipboard, AppState};
use anyhow::Result;

/// Handle clear key action
pub fn handle_clear_key(state: &mut AppState) -> Result<bool> {
    if state.selection_mode.is_some() && !state.selected_keys.is_empty() {
        // Clear all selected keys
        let layer = state.current_layer;
        for pos in &state.selected_keys.clone() {
            if let Some(layer) = state.layout.layers.get_mut(layer) {
                if let Some(key) = layer.keys.iter_mut().find(|k| k.position == *pos) {
                    key.keycode = "KC_TRNS".to_string();
                }
            }
        }
        let count = state.selected_keys.len();
        state.selected_keys.clear();
        state.selection_mode = None;
        state.mark_dirty();
        state.set_status(format!(
            "Cleared {count} keys to KC_TRNS (clipboard unchanged)"
        ));
    } else if let Some(key) = state.get_selected_key_mut() {
        key.keycode = "KC_TRNS".to_string();
        state.mark_dirty();
        state.set_status("Key cleared to KC_TRNS (clipboard unchanged)");
    }
    Ok(false)
}

/// Handle copy key action
pub fn handle_copy_key(state: &mut AppState) -> Result<bool> {
    if state.selection_mode.is_some() && !state.selected_keys.is_empty() {
        // Copy all selected keys
        let layer = state.current_layer;
        let anchor = state.selected_keys[0];
        let mut keys: Vec<(Position, clipboard::ClipboardContent)> = Vec::new();

        for pos in &state.selected_keys {
            if let Some(layer) = state.layout.layers.get(layer) {
                if let Some(key) = layer.keys.iter().find(|k| k.position == *pos) {
                    keys.push((
                        *pos,
                        clipboard::ClipboardContent {
                            keycode: key.keycode.clone(),
                            color_override: key.color_override,
                            category_id: key.category_id.clone(),
                        },
                    ));
                }
            }
        }

        let msg = state.clipboard.copy_multi(keys, anchor);
        state.selection_mode = None;
        state.selected_keys.clear();
        state.set_status(format!("{msg} - original stays until paste"));
    } else if let Some(key) = state.get_selected_key() {
        // Clone key data to avoid borrow conflict with clipboard
        let keycode = key.keycode.clone();
        let color_override = key.color_override;
        let category_id = key.category_id.clone();
        let msg = state
            .clipboard
            .copy(&keycode, color_override, category_id.as_deref());
        state.set_status(format!("{msg} - original stays until paste"));
    } else {
        state.set_error("No key to copy");
    }
    Ok(false)
}

/// Handle cut key action
pub fn handle_cut_key(state: &mut AppState) -> Result<bool> {
    if state.selection_mode.is_some() && !state.selected_keys.is_empty() {
        // Cut all selected keys
        let layer = state.current_layer;
        let anchor = state.selected_keys[0];
        let positions = state.selected_keys.clone();
        let mut keys: Vec<(Position, clipboard::ClipboardContent)> = Vec::new();

        for pos in &state.selected_keys {
            if let Some(layer_ref) = state.layout.layers.get(layer) {
                if let Some(key) = layer_ref.keys.iter().find(|k| k.position == *pos) {
                    keys.push((
                        *pos,
                        clipboard::ClipboardContent {
                            keycode: key.keycode.clone(),
                            color_override: key.color_override,
                            category_id: key.category_id.clone(),
                        },
                    ));
                }
            }
        }

        let msg = state.clipboard.cut_multi(keys, anchor, layer, positions);
        state.selection_mode = None;
        state.selected_keys.clear();
        state.set_status(msg);
    } else if let Some(key) = state.get_selected_key() {
        // Clone key data to avoid borrow conflict with clipboard
        let keycode = key.keycode.clone();
        let color_override = key.color_override;
        let category_id = key.category_id.clone();
        let msg = state.clipboard.cut(
            &keycode,
            color_override,
            category_id.as_deref(),
            state.current_layer,
            state.selected_position,
        );
        state.set_status(msg);
    } else {
        state.set_error("No key to cut");
    }
    Ok(false)
}

/// Handle paste key action
pub fn handle_paste_key(state: &mut AppState) -> Result<bool> {
    // Check if clipboard has content
    if !state.clipboard.has_content() {
        state.set_error("Nothing to paste");
        return Ok(false);
    }

    // Check for multi-key paste first
    if state.clipboard.is_multi() {
        if let Some(multi) = state.clipboard.get_multi_content().cloned() {
            // Calculate target positions relative to current position
            // The anchor is the reference point, we need to shift all keys
            let anchor = multi.anchor;
            let current = state.selected_position;

            // Calculate offset from anchor to current position
            let row_offset = current.row as isize - anchor.row as isize;
            let col_offset = current.col as isize - anchor.col as isize;

            // Collect valid target positions and save undo state
            let mut paste_targets: Vec<(Position, clipboard::ClipboardContent)> = Vec::new();
            let mut undo_keys: Vec<(Position, clipboard::ClipboardContent)> = Vec::new();

            for (pos, content) in &multi.keys {
                // Calculate target position
                let target_row = pos.row as isize + row_offset;
                let target_col = pos.col as isize + col_offset;

                if target_row >= 0
                    && target_col >= 0
                    && target_row <= u8::MAX as isize
                    && target_col <= u8::MAX as isize
                {
                    let target_pos = Position::new(target_row as u8, target_col as u8);

                    // Check if target position is valid
                    if state.mapping.is_valid_position(target_pos) {
                        // Save original for undo
                        if let Some(layer) = state.layout.layers.get(state.current_layer) {
                            if let Some(key) = layer.keys.iter().find(|k| k.position == target_pos)
                            {
                                undo_keys.push((
                                    target_pos,
                                    clipboard::ClipboardContent {
                                        keycode: key.keycode.clone(),
                                        color_override: key.color_override,
                                        category_id: key.category_id.clone(),
                                    },
                                ));
                            }
                        }
                        paste_targets.push((target_pos, content.clone()));
                    }
                }
            }

            if paste_targets.is_empty() {
                state.set_error("No valid positions for paste");
                return Ok(false);
            }

            // Save undo state
            state.clipboard.save_undo(
                state.current_layer,
                undo_keys,
                format!("Pasted {} keys", paste_targets.len()),
            );

            // Get cut sources before paste
            let cut_sources: Vec<(usize, Position)> =
                state.clipboard.get_multi_cut_sources().to_vec();

            // Apply pastes
            let paste_count = paste_targets.len();
            for (target_pos, content) in &paste_targets {
                if let Some(layer) = state.layout.layers.get_mut(state.current_layer) {
                    if let Some(key) = layer.keys.iter_mut().find(|k| k.position == *target_pos) {
                        key.keycode = content.keycode.clone();
                        key.color_override = content.color_override;
                        key.category_id = content.category_id.clone();
                    }
                }
            }

            // Clear cut sources if this was a cut operation
            for (layer_idx, pos) in cut_sources {
                if let Some(layer) = state.layout.layers.get_mut(layer_idx) {
                    if let Some(source_key) = layer.keys.iter_mut().find(|k| k.position == pos) {
                        source_key.keycode = "KC_TRNS".to_string();
                        source_key.color_override = None;
                        source_key.category_id = None;
                    }
                }
            }
            state.clipboard.clear_cut_source();

            // Flash the first pasted key (current position)
            state.flash_highlight = Some((state.current_layer, current, 5));

            state.mark_dirty();
            state.set_status(format!("Pasted {paste_count} keys"));
        }
    } else if let Some(content) = state.clipboard.get_content().cloned() {
        // Single key paste (original logic)
        // Get cut source before modifying clipboard
        let cut_source = state.clipboard.get_cut_source();

        // Save undo state before making changes
        if let Some(key) = state.get_selected_key() {
            let original = clipboard::ClipboardContent {
                keycode: key.keycode.clone(),
                color_override: key.color_override,
                category_id: key.category_id.clone(),
            };
            state.clipboard.save_undo(
                state.current_layer,
                vec![(state.selected_position, original)],
                format!("Pasted: {}", content.keycode),
            );
        }

        // Apply clipboard content to selected key
        if let Some(key) = state.get_selected_key_mut() {
            key.keycode = content.keycode.clone();
            key.color_override = content.color_override;
            key.category_id = content.category_id.clone();
            state.mark_dirty();
            state.set_status(format!("Pasted: {}", content.keycode));

            // Trigger flash highlight (5 frames ~= 250ms at 50ms/frame)
            state.flash_highlight = Some((state.current_layer, state.selected_position, 5));
        }

        // If this was a cut operation, clear the source key
        if let Some((layer_idx, pos)) = cut_source {
            if let Some(layer) = state.layout.layers.get_mut(layer_idx) {
                if let Some(source_key) = layer.keys.iter_mut().find(|k| k.position == pos) {
                    source_key.keycode = "KC_TRNS".to_string();
                    source_key.color_override = None;
                    source_key.category_id = None;
                }
            }
            state.clipboard.clear_cut_source();
        }
    } else {
        state.set_error("Nothing in clipboard");
    }
    Ok(false)
}

/// Handle undo paste action
pub fn handle_undo_paste(state: &mut AppState) -> Result<bool> {
    // Use get_undo() to peek at undo info before taking it
    if let Some(undo_info) = state.clipboard.get_undo() {
        let key_count = undo_info.original_keys.len();
        let layer_idx = undo_info.layer_index;
        let description = undo_info.description.clone();

        // Now take and apply the undo
        if let Some(undo) = state.clipboard.take_undo() {
            // Restore original keys
            for (pos, content) in undo.original_keys {
                if let Some(layer) = state.layout.layers.get_mut(layer_idx) {
                    if let Some(key) = layer.keys.iter_mut().find(|k| k.position == pos) {
                        key.keycode = content.keycode;
                        key.color_override = content.color_override;
                        key.category_id = content.category_id;
                    }
                }
            }
            state.mark_dirty();
            state.set_status(format!("Undone {key_count} key(s): {description}"));
        }
    } else {
        state.set_error("Nothing to undo");
    }
    Ok(false)
}

/// Handle toggle current key action
pub fn handle_toggle_current_key(state: &mut AppState) -> Result<bool> {
    if state.selection_mode.is_some() {
        let pos = state.selected_position;
        if let Some(idx) = state.selected_keys.iter().position(|p| *p == pos) {
            state.selected_keys.remove(idx);
        } else {
            state.selected_keys.push(pos);
        }
        state.set_status(format!("{} keys selected", state.selected_keys.len()));
    }
    Ok(false)
}
