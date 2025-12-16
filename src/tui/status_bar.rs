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
                crate::firmware::BuildStatus::Validating
                | crate::firmware::BuildStatus::Generating
                | crate::firmware::BuildStatus::Compiling => theme.warning,
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
        let clipboard_preview = state.clipboard.get_preview().map(|preview| {
            // Show clipboard type indicator using is_single()
            let type_indicator = if state.clipboard.is_single() {
                "Single"
            } else {
                "Multi"
            };
            Line::from(vec![
                Span::styled("Clipboard: ", Style::default().fg(theme.primary)),
                Span::styled(
                    format!("[{type_indicator}] "),
                    Style::default().fg(theme.text_muted),
                ),
                Span::styled(preview, Style::default().fg(theme.accent)),
                if state.clipboard.can_undo() {
                    Span::styled(" | Ctrl+Z: Undo", Style::default().fg(theme.text_muted))
                } else {
                    Span::raw("")
                },
            ])
        });

        // Get selected key's description (only show when no popup is active)
        let description_line = if state.active_popup.is_none() {
            state.get_selected_key().and_then(|key| {
                key.description.as_ref().map(|desc| {
                    // Truncate long descriptions
                    let truncated = if desc.len() > 60 {
                        format!("{}...", &desc[..57])
                    } else {
                        desc.clone()
                    };
                    Line::from(vec![
                        Span::styled("Note: ", Style::default().fg(theme.accent)),
                        Span::styled(truncated, Style::default().fg(theme.text)),
                    ])
                })
            })
        } else {
            None
        };

        // Determine if we should show hints (no active status/error message)
        let show_hints = state.status_message.is_empty()
            && state.error_message.is_none()
            && state.active_popup.is_none();

        // Build the help line first (always shown at bottom)
        let help_line = Self::get_contextual_help_line(state, theme);

        // Build content lines (status/hints, description, clipboard, build)
        let mut content_lines: Vec<Line> = Vec::new();

        // First line: error, status message, or hints
        if let Some(error) = &state.error_message {
            content_lines.push(Line::from(vec![
                Span::styled("ERROR: ", Style::default().fg(theme.error)),
                Span::raw(error),
            ]));
        } else if !state.status_message.is_empty() {
            let line = if let Some(color) = state.status_color_override {
                Line::from(vec![Span::styled(
                    state.status_message.as_str(),
                    Style::default().fg(color),
                )])
            } else {
                Line::from(state.status_message.as_str())
            };
            content_lines.push(line);
        } else if show_hints {
            content_lines.push(Self::get_hints_line(state, theme));
        }

        // Add key description if present
        if let Some(desc_line) = description_line {
            content_lines.push(desc_line);
        }

        // Add clipboard preview if present
        if let Some(clip_line) = clipboard_preview {
            content_lines.push(clip_line);
        }

        // Add build status line if present
        if let Some(build_line) = build_status_line {
            content_lines.push(build_line);
        }

        // Calculate available lines (6 height - 2 for borders = 4 lines, minus 1 for help = 3 for content)
        const MAX_CONTENT_LINES: usize = 3;

        // Pad with empty lines to push help to the bottom
        let padding_needed = MAX_CONTENT_LINES.saturating_sub(content_lines.len());

        let mut status_text: Vec<Line> = Vec::new();

        // Add content lines (truncate if too many)
        for line in content_lines.into_iter().take(MAX_CONTENT_LINES) {
            status_text.push(line);
        }

        // Add padding to push help line to bottom
        for _ in 0..padding_needed {
            status_text.push(Line::from(""));
        }

        // Add help line at the bottom
        status_text.push(help_line);

        let status = Paragraph::new(status_text)
            .style(Style::default().bg(theme.background))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Status ")
                    .style(Style::default().bg(theme.background)),
            );

        f.render_widget(status, area);
    }

    /// Get a line of contextual hints from the help registry (top hints line)
    fn get_hints_line(state: &AppState, theme: &Theme) -> Line<'static> {
        let context_name = Self::get_current_context(state);
        let registry = HelpRegistry::default();

        // Get top priority hints for this context (limit to 5 for space)
        let hints = registry.format_status_bar_hints(context_name, 5);

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
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
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
            Some(PopupType::KeycodePicker | PopupType::TapKeycodePicker) => {
                help_registry::contexts::KEYCODE_PICKER
            }
            Some(PopupType::ColorPicker) => {
                // Check color picker mode
                if let Some(super::ActiveComponent::ColorPicker(picker)) = &state.active_component {
                    if picker.get_mode() == super::color_picker::ColorPickerMode::Palette {
                        help_registry::contexts::COLOR_PICKER_PALETTE
                    } else {
                        help_registry::contexts::COLOR_PICKER_RGB
                    }
                } else {
                    help_registry::contexts::COLOR_PICKER_PALETTE
                }
            }
            Some(PopupType::CategoryManager) => help_registry::contexts::CATEGORY_MANAGER,
            Some(PopupType::CategoryPicker) => help_registry::contexts::CATEGORY_PICKER,
            Some(PopupType::LayerManager) => help_registry::contexts::LAYER_MANAGER,
            Some(PopupType::LayerPicker) => help_registry::contexts::LAYER_PICKER,
            Some(PopupType::LayoutPicker) => help_registry::contexts::LAYOUT_PICKER,
            Some(PopupType::HelpOverlay) => help_registry::contexts::HELP,
            Some(PopupType::BuildLog) => help_registry::contexts::BUILD_LOG,
            Some(PopupType::MetadataEditor) => help_registry::contexts::METADATA_EDITOR,
            Some(PopupType::SettingsManager) => help_registry::contexts::SETTINGS_MANAGER,
            Some(PopupType::ModifierPicker) => help_registry::contexts::MODIFIER_PICKER,
            Some(PopupType::TemplateBrowser) => help_registry::contexts::TEMPLATE_BROWSER,
            Some(PopupType::TemplateSaveDialog) => help_registry::contexts::TEMPLATE_SAVE,
            Some(PopupType::SetupWizard) => help_registry::contexts::SETUP_WIZARD,
            Some(PopupType::UnsavedChangesPrompt) => help_registry::contexts::UNSAVED_PROMPT,
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

    /// Get contextual help line from help registry (bottom help line)
    fn get_contextual_help_line(state: &AppState, theme: &Theme) -> Line<'static> {
        let context_name = Self::get_current_context(state);
        let registry = HelpRegistry::default();

        // Get hints for help line (show up to 5, always include "?: Help" at end)
        let all_hints = registry.get_status_bar_hints(context_name);

        // Take up to 5 hints (or 4 if we need to add "?: Help")
        let max_hints = if context_name == help_registry::contexts::MAIN {
            4
        } else {
            5
        };
        let help_hints: Vec<_> = all_hints
            .iter()
            .take(max_hints)
            .map(|binding| {
                let key = if !binding.alt_keys.is_empty() {
                    format!("{}/{}", binding.keys.join(","), binding.alt_keys.join(","))
                } else {
                    binding.keys.join(",")
                };
                let action = binding.hint.as_ref().unwrap_or(&binding.action);
                (key, action.as_str())
            })
            .collect();

        let mut spans: Vec<Span<'static>> = Vec::new();
        spans.push(Span::styled("Help: ", Style::default().fg(theme.primary)));

        if help_hints.is_empty() {
            spans.push(Span::raw("Press ? for help"));
            return Line::from(spans);
        }

        // Add all hints
        for (i, (key, action)) in help_hints.into_iter().enumerate() {
            if i > 0 {
                spans.push(Span::raw(" | "));
            }
            spans.push(Span::styled(key.clone(), Style::default().fg(theme.accent)));
            spans.push(Span::raw(": "));
            spans.push(Span::raw(action.to_string()));
        }

        // Always add "?: Help" at the end for main context
        if context_name == help_registry::contexts::MAIN {
            spans.push(Span::raw(" | "));
            spans.push(Span::styled(
                "?".to_string(),
                Style::default().fg(theme.accent),
            ));
            spans.push(Span::raw(": "));
            spans.push(Span::raw("Help"));
        }

        Line::from(spans)
    }
}
