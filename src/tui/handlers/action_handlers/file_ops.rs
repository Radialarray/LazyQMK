// File operations action handlers

use crate::export::export_to_markdown;
use crate::services::LayoutService;
use crate::tui::{AppState, ExportFilenameDialogState, PopupType, TemplateSaveDialogState};
use anyhow::Result;
use std::fs;
use std::path::PathBuf;

/// Handle quit action
pub fn handle_quit(state: &mut AppState) -> Result<bool> {
    if state.dirty {
        state.active_popup = Some(PopupType::UnsavedChangesPrompt);
        Ok(false)
    } else {
        Ok(true)
    }
}

/// Handle save action
pub fn handle_save(state: &mut AppState) -> Result<bool> {
    if let Some(path) = &state.source_path.clone() {
        LayoutService::save(&state.layout, path)?;
        state.mark_clean();
        state.set_status("Saved");
    } else {
        state.set_error("No file path set");
    }
    Ok(false)
}

/// Handle export layout action
pub fn handle_export_layout(state: &mut AppState) -> Result<bool> {
    state.export_filename_dialog_state =
        ExportFilenameDialogState::new(&state.layout.metadata.name);
    state.active_popup = Some(PopupType::ExportFilenameDialog);
    state.set_status("Export Layout - Type filename, Enter: export, Esc: cancel");
    Ok(false)
}

/// Perform the actual export to markdown
pub fn perform_export(state: &mut AppState, filename: &str) -> Result<()> {
    // Generate markdown content
    let markdown_content = export_to_markdown(&state.layout, &state.geometry, &state.keycode_db)?;

    // Determine output path
    let output_path = if filename.contains('/') || filename.contains('\\') {
        // Absolute or relative path provided
        PathBuf::from(filename)
    } else {
        // Just filename provided, use current directory or source directory
        if let Some(source_path) = &state.source_path {
            if let Some(parent) = source_path.parent() {
                parent.join(filename)
            } else {
                PathBuf::from(filename)
            }
        } else {
            PathBuf::from(filename)
        }
    };

    // Write to file
    fs::write(&output_path, markdown_content)?;

    state.set_status(format!("✓ Exported to: {}", output_path.display()));
    Ok(())
}

/// Handle save as template action
pub fn handle_save_as_template(state: &mut AppState) -> Result<bool> {
    state.template_save_dialog_state =
        TemplateSaveDialogState::new(state.layout.metadata.name.clone());
    state.active_popup = Some(PopupType::TemplateSaveDialog);
    state.set_status("Save as Template - Tab: next field, Enter: save");
    Ok(false)
}
