//! Action dispatch and firmware handling functions extracted from main TUI module.

use anyhow::Result;

use crate::firmware::BuildState;
use crate::models::{Position, VisualLayoutMapping};
use crate::services::LayoutService;
use crate::shortcuts::Action;
use crate::tui::{
    clipboard, key_editor, onboarding_wizard, ActiveComponent, AppState, CategoryPickerContext,
    PopupType, SelectionMode,
    TemplateSaveDialogState,
};

/// Calculate all positions within a rectangle defined by two corner positions.
fn calculate_rectangle_selection(
    start: Position,
    end: Position,
    mapping: &VisualLayoutMapping,
) -> Vec<Position> {
    let min_row = start.row.min(end.row);
    let max_row = start.row.max(end.row);
    let min_col = start.col.min(end.col);
    let max_col = start.col.max(end.col);

    let mut selected = Vec::new();
    for row in min_row..=max_row {
        for col in min_col..=max_col {
            let pos = Position::new(row, col);
            // Only include positions that exist in the keyboard mapping
            if mapping.is_valid_position(pos) {
                selected.push(pos);
            }
        }
    }
    selected
}

/// Handle firmware generation with validation
fn handle_firmware_generation(state: &mut AppState) -> Result<()> {
    use crate::firmware::{FirmwareGenerator, FirmwareValidator};

    // Step 1: Validate layout
    state.set_status("Validating layout...");

    let validator = FirmwareValidator::new(
        &state.layout,
        &state.geometry,
        &state.mapping,
        &state.keycode_db,
    );
    let report = validator.validate()?;

    if !report.is_valid() {
        // Show validation errors
        let error_msg = report.format_message();
        state.set_error(format!("Validation failed:\n{error_msg}"));
        return Ok(());
    }

    // Step 2: Generate firmware files
    state.set_status("Generating firmware files...");

    let generator = FirmwareGenerator::new(
        &state.layout,
        &state.geometry,
        &state.mapping,
        &state.config,
        &state.keycode_db,
    );

    match generator.generate() {
        Ok((keymap_path, config_path)) => {
            state.set_status(format!("✓ Generated: {keymap_path}, {config_path}"));
        }
        Err(e) => {
            state.set_error(format!("Generation failed: {e}"));
        }
    }

    Ok(())
}

/// Handle firmware build in background
fn handle_firmware_build(state: &mut AppState) -> Result<()> {
    // Generate firmware files first (keymap.c, config.h)
    handle_firmware_generation(state)?;

    // Check that QMK firmware path is configured
    let qmk_path = if let Some(path) = &state.config.paths.qmk_firmware {
        path.clone()
    } else {
        state.set_error("QMK firmware path not configured");
        return Ok(());
    };

    // Initialize build state if needed
    if state.build_state.is_none() {
        state.build_state = Some(BuildState::new());
    }

    let build_state = state.build_state.as_mut().unwrap();

    // Check if build already in progress
    if build_state.is_building() {
        state.set_error("Build already in progress");
        return Ok(());
    }

    // Determine correct keyboard variant path for building
    // This ensures we target the specific variant (e.g. "keebart/corne_choc_pro/standard")
    // so that QMK loads the correct configuration (including RGB settings)
    let keyboard = state.layout.metadata.keyboard.as_deref().unwrap_or("");
    let base_keyboard = AppState::extract_base_keyboard(keyboard);
    let key_count = state.geometry.keys.len();

    let build_keyboard = state
        .config
        .build
        .determine_keyboard_variant(&qmk_path, &base_keyboard, key_count)
        .unwrap_or_else(|e| {
            // Log warning but fall back to configured keyboard path
            eprintln!("Warning: Could not determine variant: {e}");
            keyboard.to_string()
        });

    // Start the build
    let keymap = state
        .layout
        .metadata
        .keymap_name
        .clone()
        .unwrap_or_else(|| "default".to_string());
    build_state.start_build(qmk_path, build_keyboard, keymap)?;

    state.set_status("Build started - check status with Shift+B");

    Ok(())
}

/// Dispatch action to appropriate handler
#[allow(clippy::too_many_lines, clippy::cognitive_complexity)]
pub fn dispatch_action(state: &mut AppState, action: Action) -> Result<bool> {
    match action {
        Action::Quit => {
            if state.dirty {
                state.active_popup = Some(PopupType::UnsavedChangesPrompt);
                Ok(false)
            } else {
                Ok(true)
            }
        }
        Action::Save => {
            if let Some(path) = &state.source_path.clone() {
                LayoutService::save(&state.layout, path)?;
                state.mark_clean();
                state.set_status("Saved");
            } else {
                state.set_error("No file path set");
            }
            Ok(false)
        }
        Action::ToggleHelp => {
            if state.active_popup == Some(PopupType::HelpOverlay) {
                state.close_component();
            } else {
                state.open_help_overlay();
            }
            Ok(false)
        }
        Action::OpenKeycodePicker => {
            // Open key editor if key is assigned, keycode picker if empty (Enter key logic)
            let key_info = state
                .get_selected_key()
                .map(|key| (key.clone(), key_editor::is_key_assigned(&key.keycode)));
            
            if let Some((key, is_assigned)) = key_info {
                if is_assigned {
                    // Key is assigned - open key editor
                    state.key_editor_state.init_for_key(&key, state.current_layer);
                    state.active_popup = Some(PopupType::KeyEditor);
                    state.set_status("Key editor - Enter: Reassign, D: Description, C: Color");
                } else {
                    // Key is empty (KC_NO, KC_TRNS) - open keycode picker
                    state.open_keycode_picker();
                    state.set_status("Select keycode - Type to search, Enter to apply");
                }
            } else {
                // No key selected - open keycode picker
                state.open_keycode_picker();
                state.set_status("Select keycode - Type to search, Enter to apply");
            }
            Ok(false)
        }
        Action::OpenLayerManager => {
            state.open_layer_manager();
            state.set_status("Layer Manager - n: new, d: delete, r: rename");
            Ok(false)
        }
        Action::OpenCategoryManager => {
            state.open_category_manager();
            state.set_status("Category Manager - n: new, r: rename, c: color, d: delete");
            Ok(false)
        }
        Action::OpenSettings => {
            state.open_settings_manager();
            state.set_status("Settings Manager - Enter: edit, Esc: close");
            Ok(false)
        }
        Action::EditMetadata => {
            state.open_metadata_editor();
            state.set_status("Edit Metadata - Tab: next field, Enter: save");
            Ok(false)
        }
        Action::SetupWizard => {
            let qmk_path = if let Some(path) = &state.config.paths.qmk_firmware {
                path.clone()
            } else {
                state.set_error("QMK firmware path not configured");
                return Ok(false);
            };

            match onboarding_wizard::OnboardingWizardState::new_for_keyboard_selection(&qmk_path) {
                Ok(wizard_state) => {
                    state.wizard_state = wizard_state;
                    state.active_popup = Some(PopupType::SetupWizard);
                    state.set_status("Setup Wizard - Follow prompts to configure");
                }
                Err(e) => {
                    state.set_error(format!("Failed to start wizard: {e}"));
                }
            }
            Ok(false)
        }
        Action::BuildFirmware => {
            handle_firmware_build(state)?;
            Ok(false)
        }
        Action::GenerateFirmware => {
            handle_firmware_generation(state)?;
            Ok(false)
        }
        Action::ViewBuildLog => {
            if state.build_state.is_some() {
                if matches!(state.active_component, Some(ActiveComponent::BuildLog(_))) {
                    state.close_component();
                } else {
                    state.open_build_log();
                }
                state.set_status("Build log toggled");
            } else {
                state.set_error("No build active");
            }
            Ok(false)
        }
        Action::BrowseTemplates => {
            state.open_template_browser();
            state.set_status("Template Browser - Enter: load, /: search");
            Ok(false)
        }
        Action::SaveAsTemplate => {
            state.template_save_dialog_state =
                TemplateSaveDialogState::new(state.layout.metadata.name.clone());
            state.active_popup = Some(PopupType::TemplateSaveDialog);
            state.set_status("Save as Template - Tab: next field, Enter: save");
            Ok(false)
        }
        Action::SwitchLayoutVariant => {
            let qmk_path = if let Some(path) = &state.config.paths.qmk_firmware {
                path.clone()
            } else {
                state.set_error("QMK firmware path not configured");
                return Ok(false);
            };

            let keyboard = state.layout.metadata.keyboard.as_deref().unwrap_or("");
            let base_keyboard = AppState::extract_base_keyboard(keyboard);

            if let Err(e) = state.open_layout_variant_picker(&qmk_path, &base_keyboard) {
                state.set_error(format!("Failed to load layouts: {e}"));
                return Ok(false);
            }

            state.set_status("Select layout variant - ↑↓: Navigate, Enter: Select");
            Ok(false)
        }
        Action::NextLayer => {
            if state.current_layer < state.layout.layers.len() - 1 {
                state.current_layer += 1;
                state.set_status(format!("Layer {}", state.current_layer));
                state.clear_error();
            }
            Ok(false)
        }
        Action::PreviousLayer => {
            if state.current_layer > 0 {
                state.current_layer -= 1;
                state.set_status(format!("Layer {}", state.current_layer));
                state.clear_error();
            }
            Ok(false)
        }
        Action::ClearKey => {
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
                state.set_status(format!("Cleared {count} keys"));
            } else if let Some(key) = state.get_selected_key_mut() {
                key.keycode = "KC_TRNS".to_string();
                state.mark_dirty();
                state.set_status("Key cleared (KC_TRNS)");
            }
            Ok(false)
        }
        Action::ToggleSelectionMode => {
            if state.selection_mode.is_some() {
                // Exit selection mode
                state.selection_mode = None;
                state.selected_keys.clear();
                state.set_status("Selection mode cancelled");
            } else {
                // Enter selection mode with current key selected
                state.selection_mode = Some(SelectionMode::Normal);
                state.selected_keys.clear();
                state.selected_keys.push(state.selected_position);
                state
                    .set_status("Selection mode - Space: toggle key, y: copy, d: cut, Esc: cancel");
            }
            Ok(false)
        }
        Action::ToggleCurrentKey => {
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
        Action::StartRectangleSelect => {
            if state.selection_mode.is_some() {
                // Start rectangle selection from current position
                state.selection_mode = Some(SelectionMode::Rectangle {
                    start: state.selected_position,
                });
                state.selected_keys.clear();
                state.selected_keys.push(state.selected_position);
                state.set_status("Rectangle select - move to opposite corner, Enter to confirm");
            } else {
                // Enter rectangle selection mode
                state.selection_mode = Some(SelectionMode::Rectangle {
                    start: state.selected_position,
                });
                state.selected_keys.clear();
                state.selected_keys.push(state.selected_position);
                state.set_status("Rectangle select - move to opposite corner, Enter to confirm");
            }
            Ok(false)
        }
        Action::NavigateUp => {
            if let Some(new_pos) = state.mapping.find_position_up(state.selected_position) {
                state.selected_position = new_pos;
                // Update rectangle selection if active
                if let Some(SelectionMode::Rectangle { start }) = state.selection_mode {
                    state.selected_keys =
                        calculate_rectangle_selection(start, new_pos, &state.mapping);
                }
                state.clear_error();
            }
            Ok(false)
        }
        Action::NavigateDown => {
            if let Some(new_pos) = state.mapping.find_position_down(state.selected_position) {
                state.selected_position = new_pos;
                if let Some(SelectionMode::Rectangle { start }) = state.selection_mode {
                    state.selected_keys =
                        calculate_rectangle_selection(start, new_pos, &state.mapping);
                }
                state.clear_error();
            }
            Ok(false)
        }
        Action::NavigateLeft => {
            if let Some(new_pos) = state.mapping.find_position_left(state.selected_position) {
                state.selected_position = new_pos;
                if let Some(SelectionMode::Rectangle { start }) = state.selection_mode {
                    state.selected_keys =
                        calculate_rectangle_selection(start, new_pos, &state.mapping);
                }
                state.clear_error();
            }
            Ok(false)
        }
        Action::NavigateRight => {
            if let Some(new_pos) = state.mapping.find_position_right(state.selected_position) {
                state.selected_position = new_pos;
                if let Some(SelectionMode::Rectangle { start }) = state.selection_mode {
                    state.selected_keys =
                        calculate_rectangle_selection(start, new_pos, &state.mapping);
                }
                state.clear_error();
            }
            Ok(false)
        }
        Action::JumpToFirst => {
            // Not yet implemented
            Ok(false)
        }
        Action::JumpToLast => {
            // Not yet implemented
            Ok(false)
        }
        Action::CopyKey => {
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
                state.set_status(msg);
            } else if let Some(key) = state.get_selected_key() {
                // Clone key data to avoid borrow conflict with clipboard
                let keycode = key.keycode.clone();
                let color_override = key.color_override;
                let category_id = key.category_id.clone();
                let msg = state
                    .clipboard
                    .copy(&keycode, color_override, category_id.as_deref());
                state.set_status(msg);
            } else {
                state.set_error("No key to copy");
            }
            Ok(false)
        }
        Action::CutKey => {
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
        Action::PasteKey => {
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
                    let mut paste_targets: Vec<(Position, clipboard::ClipboardContent)> =
                        Vec::new();
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
                                    if let Some(key) =
                                        layer.keys.iter().find(|k| k.position == target_pos)
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
                            if let Some(key) =
                                layer.keys.iter_mut().find(|k| k.position == *target_pos)
                            {
                                key.keycode = content.keycode.clone();
                                key.color_override = content.color_override;
                                key.category_id = content.category_id.clone();
                            }
                        }
                    }

                    // Clear cut sources if this was a cut operation
                    for (layer_idx, pos) in cut_sources {
                        if let Some(layer) = state.layout.layers.get_mut(layer_idx) {
                            if let Some(source_key) =
                                layer.keys.iter_mut().find(|k| k.position == pos)
                            {
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
                        if let Some(source_key) = layer.keys.iter_mut().find(|k| k.position == pos)
                        {
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
        Action::UndoPaste => {
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
        Action::AssignCategoryToKey => {
            // Assign category to individual key (Ctrl+K)
            if state.get_selected_key().is_some() {
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
        Action::AssignCategoryToLayer => {
            // Assign category to layer (Shift+L or Ctrl+L)
            let picker = crate::tui::CategoryPicker::new();
            state.active_component = Some(ActiveComponent::CategoryPicker(picker));
            state.category_picker_context = Some(CategoryPickerContext::Layer);
            state.active_popup = Some(PopupType::CategoryPicker);
            state.set_status("Select category for layer - Enter to apply");
            Ok(false)
        }
        Action::SetIndividualKeyColor => {
            // Set individual key color (Shift+C)
            if let Some(key) = state.get_selected_key() {
                // Initialize color picker with current key color
                let current_color = state.layout.resolve_key_color(state.current_layer, key);
                state.open_color_picker(
                    crate::tui::component::ColorPickerContext::IndividualKey,
                    current_color,
                );
                state
                    .set_status("Adjust color with arrows, Tab to switch channels, Enter to apply");
            } else {
                state.set_error("No key selected");
            }
            Ok(false)
        }
        Action::SetLayerColor => {
            // Set layer default color (c)
            if let Some(layer) = state.layout.layers.get(state.current_layer) {
                // Initialize color picker with current layer default color
                state.open_color_picker(
                    crate::tui::component::ColorPickerContext::LayerDefault,
                    layer.default_color,
                );
                state.set_status("Setting layer default color - Enter to apply");
            }
            Ok(false)
        }
        Action::ToggleLayerColors => {
            // Toggle colors for current layer (v)
            if let Some(enabled) = state.layout.toggle_layer_colors(state.current_layer) {
                state.mark_dirty();
                let status = if enabled {
                    format!("Layer {} colors enabled", state.current_layer)
                } else {
                    format!("Layer {} colors disabled", state.current_layer)
                };
                state.set_status(status);
            }
            Ok(false)
        }
        Action::ToggleAllLayerColors => {
            // Toggle colors for all layers (Alt+V)
            let enabled = state.layout.toggle_all_layer_colors();
            state.mark_dirty();
            let status = if enabled {
                "All layer colors enabled".to_string()
            } else {
                "All layer colors disabled".to_string()
            };
            state.set_status(status);
            Ok(false)
        }
        Action::Cancel => {
            // Cancel selection/cut/clipboard (Escape)
            #[allow(clippy::redundant_pattern_matching)]
            if let Some(_) = state.selection_mode {
                state.selection_mode = None;
                state.selected_keys.clear();
                state.set_status("Selection cancelled");
            } else if state.clipboard.is_cut() {
                state.clipboard.cancel_cut();
                state.set_status("Cut cancelled");
            } else if state.clipboard.has_content() {
                // Clear clipboard if there's content but no active cut
                state.clipboard.clear();
                state.set_status("Clipboard cleared");
            }
            Ok(false)
        }
    }
}
