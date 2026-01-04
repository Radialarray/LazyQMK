//! Action dispatch and firmware handling functions extracted from main TUI module.

use anyhow::Result;

use crate::firmware::BuildState;
use crate::shortcuts::Action;
use crate::tui::AppState;

use super::action_handlers;

use action_handlers::{
    category, color, file_ops, firmware, key_ops, layout, navigation, popups, selection,
};

/// Handle firmware generation with validation
pub(super) fn handle_firmware_generation(state: &mut AppState) -> Result<()> {
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
pub(super) fn handle_firmware_build(state: &mut AppState) -> Result<()> {
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
pub fn dispatch_action(state: &mut AppState, action: Action) -> Result<bool> {
    match action {
        // Navigation (8 actions)
        Action::NavigateUp => navigation::handle_navigate_up(state),
        Action::NavigateDown => navigation::handle_navigate_down(state),
        Action::NavigateLeft => navigation::handle_navigate_left(state),
        Action::NavigateRight => navigation::handle_navigate_right(state),
        Action::JumpToFirst => navigation::handle_jump_to_first(state),
        Action::JumpToLast => navigation::handle_jump_to_last(state),
        Action::NextLayer => navigation::handle_next_layer(state),
        Action::PreviousLayer => navigation::handle_previous_layer(state),

        // File operations (4 actions)
        Action::Quit => file_ops::handle_quit(state),
        Action::Save => file_ops::handle_save(state),
        Action::ExportLayout => file_ops::handle_export_layout(state),
        Action::SaveAsTemplate => file_ops::handle_save_as_template(state),

        // Popup management (10 actions)
        Action::OpenKeycodePicker => popups::handle_open_keycode_picker(state),
        Action::OpenLayerManager => popups::handle_open_layer_manager(state),
        Action::OpenCategoryManager => popups::handle_open_category_manager(state),
        Action::OpenSettings => popups::handle_open_settings(state),
        Action::EditMetadata => popups::handle_edit_metadata(state),
        Action::OpenTapDanceEditor => popups::handle_open_tap_dance_editor(state),
        Action::SetupWizard => popups::handle_setup_wizard(state),
        Action::BrowseTemplates => popups::handle_browse_templates(state),
        Action::ViewBuildLog => popups::handle_view_build_log(state),
        Action::ToggleHelp => popups::handle_toggle_help(state),

        // Key operations (6 actions)
        Action::ClearKey => key_ops::handle_clear_key(state),
        Action::CopyKey => key_ops::handle_copy_key(state),
        Action::CutKey => key_ops::handle_cut_key(state),
        Action::PasteKey => key_ops::handle_paste_key(state),
        Action::UndoPaste => key_ops::handle_undo_paste(state),
        Action::ToggleCurrentKey => key_ops::handle_toggle_current_key(state),

        // Selection (2 actions)
        Action::ToggleSelectionMode => selection::handle_toggle_selection_mode(state),
        Action::StartRectangleSelect => selection::handle_start_rectangle_select(state),

        // Color management (4 actions)
        Action::SetIndividualKeyColor => color::handle_set_individual_key_color(state),
        Action::SetLayerColor => color::handle_set_layer_color(state),
        Action::ToggleLayerColors => color::handle_toggle_layer_colors(state),
        Action::ToggleAllLayerColors => color::handle_toggle_all_layer_colors(state),

        // Category assignment (2 actions)
        Action::AssignCategoryToKey => category::handle_assign_category_to_key(state),
        Action::AssignCategoryToLayer => category::handle_assign_category_to_layer(state),

        // Firmware (2 actions)
        Action::BuildFirmware => firmware::handle_build_firmware(state),
        Action::GenerateFirmware => firmware::handle_generate_firmware(state),

        // Layout (1 action)
        Action::SwitchLayoutVariant => layout::handle_switch_layout_variant(state),

        // Cancel (1 action)
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
