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
        let mut status_text: Vec<Line> = Vec::new();
        status_text.push(Self::get_selection_summary_line(state, theme));

        if let Some(error) = &state.error_message {
            status_text.push(Line::from(vec![
                Span::styled("Error: ", Style::default().fg(theme.error)),
                Span::styled(error, Style::default().fg(theme.text)),
            ]));
        } else if !state.status_message.is_empty() {
            status_text.push(if let Some(color) = state.status_color_override {
                Line::from(vec![Span::styled(
                    state.status_message.as_str(),
                    Style::default().fg(color),
                )])
            } else {
                Line::from(state.status_message.as_str())
            });
        } else {
            status_text.push(Self::get_clipboard_or_note_line(state, theme));
        }

        status_text.push(build_status_line.unwrap_or_else(|| Self::get_hints_line(state, theme)));
        status_text.push(Self::get_contextual_help_line(state, theme));

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

    fn get_selection_summary_line(state: &AppState, theme: &Theme) -> Line<'static> {
        let dirty_label = if state.dirty {
            "Unsaved changes"
        } else {
            "Saved"
        };
        let dirty_color = if state.dirty {
            theme.warning
        } else {
            theme.success
        };

        let key_summary = state.get_selected_key().map_or_else(
            || "No key selected".to_string(),
            |key| {
                format!(
                    "Layer {} • Key ({}, {}) • {}",
                    state.current_layer,
                    state.selected_position.row,
                    state.selected_position.col,
                    key.keycode
                )
            },
        );

        Line::from(vec![
            Span::styled("Selection: ", Style::default().fg(theme.primary)),
            Span::styled(key_summary, Style::default().fg(theme.text)),
            Span::styled("  |  Draft: ", Style::default().fg(theme.primary)),
            Span::styled(dirty_label, Style::default().fg(dirty_color)),
        ])
    }

    fn get_clipboard_or_note_line(state: &AppState, theme: &Theme) -> Line<'static> {
        if let Some(preview) = state.clipboard.get_preview() {
            let clipboard_type = if state.clipboard.is_single() {
                "Single key"
            } else {
                "Multi key"
            };
            return Line::from(vec![
                Span::styled("Clipboard: ", Style::default().fg(theme.primary)),
                Span::styled(clipboard_type, Style::default().fg(theme.text_muted)),
                Span::raw(" • "),
                Span::styled(preview, Style::default().fg(theme.accent)),
                if state.clipboard.can_undo() {
                    Span::styled(" • Ctrl+Z undo", Style::default().fg(theme.text_muted))
                } else {
                    Span::raw("")
                },
            ]);
        }

        if state.active_popup.is_none() {
            if let Some(desc) = state
                .get_selected_key()
                .and_then(|key| key.description.as_ref())
            {
                let truncated = if desc.len() > 72 {
                    format!("{}...", &desc[..69])
                } else {
                    desc.clone()
                };
                return Line::from(vec![
                    Span::styled("Key note: ", Style::default().fg(theme.primary)),
                    Span::styled(truncated, Style::default().fg(theme.text)),
                ]);
            }
        }

        Line::from(vec![Span::styled(
            "Enter opens key details. Use visible actions in keyboard view for common tasks.",
            Style::default().fg(theme.text_muted),
        )])
    }

    /// Get a line of contextual hints from help registry
    fn get_hints_line(state: &AppState, theme: &Theme) -> Line<'static> {
        let context_name = Self::get_current_context(state);
        let registry = HelpRegistry::default();

        let hints = registry.format_status_bar_hints(context_name, 4);

        if hints.is_empty() {
            return Line::from("");
        }

        let mut spans: Vec<Span<'static>> = Vec::new();
        spans.push(Span::styled(
            "Primary actions: ",
            Style::default().fg(theme.primary),
        ));
        for (i, (key, action)) in hints.into_iter().enumerate() {
            if i > 0 {
                spans.push(Span::raw("  |  "));
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
            Some(PopupType::TapDanceEditor) => help_registry::contexts::TAP_DANCE_EDITOR,
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
        spans.push(Span::styled("More: ", Style::default().fg(theme.primary)));

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
