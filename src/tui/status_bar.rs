//! Status bar widget for displaying status messages and help

use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use super::help_registry::{self, HelpRegistry};
use super::{AppState, Theme};

/// Status bar widget
pub struct StatusBar;

impl StatusBar {
    /// Render the status bar with contextual help
    pub fn render(f: &mut Frame, area: Rect, state: &AppState, theme: &Theme) {
        // Build status indicator if build is active
        let build_status_line = if let Some(build_state) = &state.build_state {
            let status = &build_state.status;
            let color = match status {
                crate::firmware::BuildStatus::Idle => theme.inactive,
                crate::firmware::BuildStatus::Validating => theme.warning,
                crate::firmware::BuildStatus::Generating => theme.warning,
                crate::firmware::BuildStatus::Compiling => theme.warning,
                crate::firmware::BuildStatus::Success => theme.success,
                crate::firmware::BuildStatus::Failed => theme.error,
            };

            Some(Line::from(vec![
                Span::styled("Build: ", Style::default().fg(theme.primary)),
                Span::styled(status.to_string(), Style::default().fg(color)),
            ]))
        } else {
            None
        };

        // Build clipboard preview line
        let has_clipboard = state.clipboard.has_content();
        let clipboard_preview = state.clipboard.get_preview().map(|preview| {
            Line::from(vec![
                Span::styled("Clipboard: ", Style::default().fg(theme.primary)),
                Span::styled(preview, Style::default().fg(theme.accent)),
                if state.clipboard.can_undo() {
                    Span::styled(" | Ctrl+Z: Undo", Style::default().fg(theme.text_muted))
                } else {
                    Span::raw("")
                },
            ])
        });

        // Determine if we should show hints (no active status/error message)
        let show_hints = state.status_message.is_empty() 
            && state.error_message.is_none()
            && state.active_popup.is_none();

        let mut status_text = if let Some(error) = &state.error_message {
            vec![Line::from(vec![
                Span::styled("ERROR: ", Style::default().fg(theme.error)),
                Span::raw(error),
            ])]
        } else if !state.status_message.is_empty() {
            vec![Line::from(state.status_message.as_str())]
        } else if show_hints {
            // Show contextual hints when no status message
            vec![Self::get_hints_line(state, theme)]
        } else {
            vec![Line::from("")]
        };

        // Add clipboard preview if present
        if let Some(clip_line) = clipboard_preview {
            status_text.push(clip_line);
        }

        // Add build status line if present
        if let Some(build_line) = build_status_line {
            status_text.push(build_line);
        } else if !has_clipboard {
            status_text.push(Line::from(""));
        }

        // Add help line (contextual based on popup/mode)
        let help_message = Self::get_contextual_help(state);
        status_text.push(Line::from(vec![
            Span::styled("Help: ", Style::default().fg(theme.primary)),
            Span::raw(help_message),
        ]));

        let status = Paragraph::new(status_text)
            .block(Block::default().borders(Borders::ALL).title(" Status "));

        f.render_widget(status, area);
    }

    /// Get a line of contextual hints from the help registry
    fn get_hints_line(state: &AppState, theme: &Theme) -> Line<'static> {
        let context_name = Self::get_current_context(state);
        let registry = HelpRegistry::default();
        let hints = registry.format_status_bar_hints(context_name, 8);

        if hints.is_empty() {
            return Line::from("");
        }

        let mut spans: Vec<Span<'static>> = Vec::new();
        for (i, (key, action)) in hints.into_iter().enumerate() {
            if i > 0 {
                spans.push(Span::raw("  "));
            }
            spans.push(Span::styled(
                key,
                Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
            ));
            spans.push(Span::raw(" "));
            spans.push(Span::styled(action, Style::default().fg(theme.text_muted)));
        }

        Line::from(spans)
    }

    /// Get the current context name based on application state
    fn get_current_context(state: &AppState) -> &'static str {
        use super::PopupType;

        match &state.active_popup {
            Some(PopupType::KeycodePicker) | Some(PopupType::TapKeycodePicker) => {
                help_registry::contexts::KEYCODE_PICKER
            }
            Some(PopupType::ColorPicker) => {
                // Check color picker mode
                if state.color_picker_state.mode == super::color_picker::ColorPickerMode::Palette {
                    help_registry::contexts::COLOR_PICKER_PALETTE
                } else {
                    help_registry::contexts::COLOR_PICKER_RGB
                }
            }
            Some(PopupType::CategoryManager) => help_registry::contexts::CATEGORY_MANAGER,
            Some(PopupType::LayerManager) => help_registry::contexts::LAYER_MANAGER,
            Some(PopupType::LayerPicker) => help_registry::contexts::LAYER_PICKER,
            Some(PopupType::HelpOverlay) => help_registry::contexts::HELP,
            Some(PopupType::BuildLog) => help_registry::contexts::BUILD_LOG,
            Some(PopupType::MetadataEditor) => help_registry::contexts::METADATA_EDITOR,
            Some(PopupType::SettingsManager) => help_registry::contexts::SETTINGS_MANAGER,
            Some(PopupType::ModifierPicker) => help_registry::contexts::MODIFIER_PICKER,
            _ => {
                // Check for selection mode
                if state.selection_mode.is_some() {
                    help_registry::contexts::SELECTION
                } else {
                    help_registry::contexts::MAIN
                }
            }
        }
    }

    /// Get contextual help message based on current application state
    fn get_contextual_help(state: &AppState) -> &'static str {
        use super::PopupType;

        match &state.active_popup {
            Some(PopupType::KeycodePicker) => {
                "↑↓: Navigate | Enter: Select | Esc: Cancel | Tab: Switch | Type: Search"
            }
            Some(PopupType::ColorPicker) => {
                "←→↑↓: Navigate | Tab: Switch | Enter: Apply | Esc: Cancel"
            }
            Some(PopupType::CategoryPicker) => "↑↓: Navigate | Enter: Select | Esc: Cancel",
            Some(PopupType::CategoryManager) => {
                "n: New | r: Rename | c: Color | d: Delete | Enter: Select | Esc: Close"
            }
            Some(PopupType::LayerManager) => {
                "n: New | r: Rename | v: Toggle Colors | d: Delete | Shift+↑↓: Reorder | Esc: Close"
            }
            Some(PopupType::LayerPicker) => {
                "↑↓: Navigate | Enter: Select layer | Esc: Cancel"
            }
            Some(PopupType::TemplateBrowser) => {
                "↑↓: Navigate | Enter: Load | /: Search | Esc: Cancel"
            }
            Some(PopupType::TemplateSaveDialog) => {
                "Tab: Next field | Enter: Save | Esc: Cancel | Type: Edit"
            }
            Some(PopupType::HelpOverlay) => "↑↓: Scroll | Home/End: Jump | ?: Close",
            Some(PopupType::BuildLog) => "↑↓: Scroll | Home/End: Jump | Esc: Close",
            Some(PopupType::MetadataEditor) => {
                "Tab: Next field | Enter: Save | Esc: Cancel | Type: Edit"
            }
            Some(PopupType::UnsavedChangesPrompt) => "y: Save and quit | n: Discard | Esc: Cancel",
            Some(PopupType::LayoutPicker) => "↑↓: Navigate | Enter: Select | Esc: Cancel",
            Some(PopupType::SetupWizard) => {
                "Enter: Next | Esc: Back/Cancel | ↑↓: Navigate | Type: Input"
            }
            Some(PopupType::SettingsManager) => {
                "↑↓: Navigate | Enter: Change | Esc: Close"
            }
            Some(PopupType::TapKeycodePicker) => {
                "↑↓: Navigate | Enter: Select tap keycode | Esc: Cancel"
            }
            Some(PopupType::ModifierPicker) => {
                "↑↓←→: Navigate | Space: Toggle | Enter: Confirm | Esc: Cancel"
            }
            None => {
                // Main keyboard editing mode
                if state.selection_mode.is_some() {
                    "↑↓←→: Move | Space: Toggle | y: Copy | d: Cut | Esc: Exit"
                } else {
                    "↑↓←→: Navigate | Enter: Edit | Shift+C: Color | Shift+N: Layers | ?: Help"
                }
            }
        }
    }
}
