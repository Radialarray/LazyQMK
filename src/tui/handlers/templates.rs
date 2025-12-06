//! Template browser and save dialog input handlers.

use anyhow::{Context, Result};
use crossterm::event::{self, KeyCode, KeyModifiers};

use crate::services::LayoutService;
use crate::tui::{template_browser::TemplateBrowserState, AppState, component::Component};

/// Handle input for template browser
pub fn handle_template_browser_input(state: &mut AppState, key: event::KeyEvent) -> Result<bool> {
    // Use Component trait pattern
    use crate::tui::ActiveComponent;
    
    if let Some(ActiveComponent::TemplateBrowser(ref mut browser)) = state.active_component {
        if let Some(event) = browser.handle_input(key) {
            return handle_template_browser_event(state, event);
        }
    }
    Ok(false)
}

/// Handle template browser events
fn handle_template_browser_event(state: &mut AppState, event: crate::tui::template_browser::TemplateBrowserEvent) -> Result<bool> {
    use crate::tui::template_browser::TemplateBrowserEvent;
    
    match event {
        TemplateBrowserEvent::TemplateSelected(path) => {
            // Load the template
            match LayoutService::load(&path).with_context(|| format!("Loading template from {}", path.display())) {
                Ok(layout) => {
                    state.layout = layout;
                    state.source_path = None; // New layout from template
                    state.mark_dirty(); // Mark as dirty since it's unsaved
                    state.close_component();
                    state.set_status("Template loaded");
                }
                Err(e) => {
                    state.set_error(format!("Failed to load template: {e}"));
                }
            }
        }
        TemplateBrowserEvent::SaveAsTemplate => {
            // Open save dialog (existing functionality - this event is not currently used)
            state.close_component();
            state.set_status("Save as template feature coming soon");
        }
        TemplateBrowserEvent::Cancelled => {
            state.close_component();
            state.set_status("Template browser closed");
        }
    }
    Ok(false)
}

/// Handle input for template save dialog
pub fn handle_template_save_dialog_input(state: &mut AppState, key: event::KeyEvent) -> Result<bool> {
    match key.code {
        KeyCode::Char(c) => {
            // Add character to active field
            state
                .template_save_dialog_state
                .get_active_field_mut()
                .push(c);
            Ok(false)
        }
        KeyCode::Backspace => {
            // Remove character from active field
            state
                .template_save_dialog_state
                .get_active_field_mut()
                .pop();
            Ok(false)
        }
        KeyCode::Tab => {
            // Move to next field
            if key.modifiers.contains(KeyModifiers::SHIFT) {
                state.template_save_dialog_state.previous_field();
            } else {
                state.template_save_dialog_state.next_field();
            }
            Ok(false)
        }
        KeyCode::Enter => {
            // Save template
            let dialog_state = &state.template_save_dialog_state;

            // Validate name is not empty
            if dialog_state.name.trim().is_empty() {
                state.set_error("Template name cannot be empty");
                return Ok(false);
            }

            // Create template directory if it doesn't exist
            let templates_dir = TemplateBrowserState::templates_dir()?;
            std::fs::create_dir_all(&templates_dir).context(format!(
                "Failed to create templates directory: {}",
                templates_dir.display()
            ))?;

            // Update layout metadata for template
            let mut template_layout = state.layout.clone();
            template_layout.metadata.name = dialog_state.name.clone();
            template_layout.metadata.description = dialog_state.description.clone();
            template_layout.metadata.author = dialog_state.author.clone();
            template_layout.metadata.tags = dialog_state.parse_tags();
            template_layout.metadata.is_template = true;
            template_layout.metadata.touch();

            // Generate filename from name (sanitize)
            let filename = dialog_state
                .name
                .to_lowercase()
                .replace(' ', "-")
                .chars()
                .filter(|c| c.is_alphanumeric() || *c == '-')
                .collect::<String>();
            let template_path = templates_dir.join(format!("{filename}.md"));

            // Save template
            match LayoutService::save(&template_layout, &template_path) {
                Ok(()) => {
                    state.active_popup = None;
                    state.set_status(format!("Template saved: {}", template_path.display()));
                    Ok(false)
                }
                Err(e) => {
                    state.set_error(format!("Failed to save template: {e}"));
                    Ok(false)
                }
            }
        }
        KeyCode::Esc => {
            // Cancel
            state.active_popup = None;
            state.set_status("Template save cancelled");
            Ok(false)
        }
        _ => Ok(false),
    }
}
