//! Status bar widget for displaying status messages and help

use ratatui::{
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use super::{AppState, Theme};

/// Status bar widget
pub struct StatusBar;

impl StatusBar {
    /// Render the status bar with contextual help
    pub fn render(f: &mut Frame, area: Rect, state: &AppState, theme: &Theme) {
        // Determine contextual help message based on active popup or mode
        let help_message = Self::get_contextual_help(state);

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

        let mut status_text = if let Some(error) = &state.error_message {
            vec![Line::from(vec![
                Span::styled("ERROR: ", Style::default().fg(theme.error)),
                Span::raw(error),
            ])]
        } else {
            vec![Line::from(state.status_message.as_str())]
        };

        // Add build status line if present
        if let Some(build_line) = build_status_line {
            status_text.push(build_line);
        } else {
            status_text.push(Line::from(""));
        }

        // Add help line
        status_text.push(Line::from(vec![
            Span::styled("Help: ", Style::default().fg(theme.primary)),
            Span::raw(help_message),
        ]));

        let status = Paragraph::new(status_text)
            .block(Block::default().borders(Borders::ALL).title(" Status "));

        f.render_widget(status, area);
    }

    /// Get contextual help message based on current application state
    const fn get_contextual_help(state: &AppState) -> &'static str {
        use super::PopupType;

        match &state.active_popup {
            Some(PopupType::KeycodePicker) => {
                "↑↓: Navigate | Enter: Select | Esc: Cancel | 1-8: Filter category | Type: Search"
            }
            Some(PopupType::ColorPicker) => {
                "↑↓: Change channel | ←→: Adjust value | Enter: Apply | Esc: Cancel"
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
            Some(PopupType::HelpOverlay) => "↑↓: Scroll | Home/End: Jump | Esc: Close",
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
            None => {
                // Main keyboard editing mode
                "↑↓←→/hjkl: Navigate | Enter: Edit key | x/Del: Clear | Tab: Layer | Ctrl+S: Save | Ctrl+Q: Quit | ?: Help"
            }
        }
    }
}
