//! Layer manager for CRUD operations on layers.
//!
//! Provides a UI for creating, renaming, deleting, and reordering layers.
//! Accessible via Ctrl+H from the main editor.

use ratatui::{
    layout::{Alignment, Constraint, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::models::{layer::Layer, RgbColor};

use super::Theme;

/// Mode for the layer manager popup.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LayerManagerMode {
    /// Browsing existing layers
    Browsing,
    /// Creating a new layer (entering name)
    CreatingName {
        /// User input for new layer name
        input: String,
    },
    /// Renaming an existing layer
    Renaming {
        /// Index of layer being renamed
        index: usize,
        /// User input for new name
        input: String,
    },
    /// Confirming deletion of a layer
    ConfirmingDelete {
        /// Index of layer to delete
        index: usize,
    },
}

/// State for the layer manager dialog.
#[derive(Debug, Clone)]
pub struct LayerManagerState {
    /// Selected layer index in the list
    pub selected: usize,
    /// Current operation mode
    pub mode: LayerManagerMode,
}

impl LayerManagerState {
    /// Create a new layer manager state.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            selected: 0,
            mode: LayerManagerMode::Browsing,
        }
    }

    /// Reset to default browsing state.
    pub fn reset(&mut self) {
        self.selected = 0;
        self.mode = LayerManagerMode::Browsing;
    }

    /// Move selection up.
    pub const fn select_previous(&mut self, layer_count: usize) {
        if layer_count > 0 {
            if self.selected > 0 {
                self.selected -= 1;
            } else {
                self.selected = layer_count - 1;
            }
        }
    }

    /// Move selection down.
    pub const fn select_next(&mut self, layer_count: usize) {
        if layer_count > 0 {
            self.selected = (self.selected + 1) % layer_count;
        }
    }

    /// Start creating a new layer.
    pub fn start_creating(&mut self) {
        self.mode = LayerManagerMode::CreatingName {
            input: String::new(),
        };
    }

    /// Start renaming the selected layer.
    pub fn start_renaming(&mut self, index: usize, current_name: &str) {
        self.mode = LayerManagerMode::Renaming {
            index,
            input: current_name.to_string(),
        };
    }

    /// Start confirming deletion for the selected layer.
    pub fn start_deleting(&mut self, index: usize) {
        self.mode = LayerManagerMode::ConfirmingDelete { index };
    }

    /// Cancel current operation and return to browsing.
    pub fn cancel(&mut self) {
        self.mode = LayerManagerMode::Browsing;
    }

    /// Get the current input text (for name entry or renaming).
    #[must_use]
    pub fn get_input(&self) -> Option<&str> {
        match &self.mode {
            LayerManagerMode::CreatingName { input } => Some(input),
            LayerManagerMode::Renaming { input, .. } => Some(input),
            _ => None,
        }
    }

    /// Get mutable reference to current input text.
    pub fn get_input_mut(&mut self) -> Option<&mut String> {
        match &mut self.mode {
            LayerManagerMode::CreatingName { input } => Some(input),
            LayerManagerMode::Renaming { input, .. } => Some(input),
            _ => None,
        }
    }
}

impl Default for LayerManagerState {
    fn default() -> Self {
        Self::new()
    }
}

/// Render the layer manager dialog.
pub fn render_layer_manager(
    f: &mut Frame,
    area: Rect,
    state: &LayerManagerState,
    layers: &[Layer],
    theme: &Theme,
) {
    // Center the dialog (70% width, 70% height)
    let dialog_width = (area.width * 70) / 100;
    let dialog_height = (area.height * 70) / 100;
    let dialog_x = (area.width - dialog_width) / 2;
    let dialog_y = (area.height - dialog_height) / 2;

    let dialog_area = Rect {
        x: dialog_x,
        y: dialog_y,
        width: dialog_width,
        height: dialog_height,
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Layer Manager (Ctrl+H) ")
        .style(Style::default().bg(theme.background));

    f.render_widget(block, dialog_area);

    let inner_area = Rect {
        x: dialog_area.x + 2,
        y: dialog_area.y + 1,
        width: dialog_area.width.saturating_sub(4),
        height: dialog_area.height.saturating_sub(2),
    };

    match &state.mode {
        LayerManagerMode::Browsing => {
            render_layer_list(f, inner_area, state, layers, theme);
        }
        LayerManagerMode::CreatingName { input } => {
            render_name_input(
                f,
                inner_area,
                "Create Layer",
                input,
                "Enter layer name:",
                theme,
            );
        }
        LayerManagerMode::Renaming { input, .. } => {
            render_name_input(
                f,
                inner_area,
                "Rename Layer",
                input,
                "Enter new name:",
                theme,
            );
        }
        LayerManagerMode::ConfirmingDelete { index } => {
            if let Some(layer) = layers.get(*index) {
                render_delete_confirmation(f, inner_area, layer, theme);
            }
        }
    }
}

fn render_layer_list(
    f: &mut Frame,
    area: Rect,
    state: &LayerManagerState,
    layers: &[Layer],
    theme: &Theme,
) {
    let chunks = ratatui::layout::Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([
            Constraint::Min(5),    // Layer list
            Constraint::Length(5), // Help text
        ])
        .split(area);

    let items: Vec<ListItem> = layers
        .iter()
        .enumerate()
        .map(|(i, layer)| {
            let style = if i == state.selected {
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.text)
            };

            let color = layer.default_color;
            let color_box = "████".to_string();
            let content = Line::from(vec![
                Span::styled(
                    format!("{:2}: ", layer.number),
                    Style::default().fg(theme.text_muted),
                ),
                Span::styled(
                    color_box,
                    Style::default().fg(ratatui::style::Color::Rgb(
                        color.r,
                        color.g,
                        color.b,
                    )),
                ),
                Span::raw(" "),
                Span::styled(&layer.name, style),
            ]);

            ListItem::new(content)
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Layers ")
                .border_style(Style::default().fg(theme.primary)),
        )
        .highlight_symbol("▶ ")
        .highlight_style(Style::default().fg(theme.accent).add_modifier(Modifier::BOLD));

    f.render_widget(list, chunks[0]);

    let help_lines = vec![
        Line::from(vec![
            Span::styled("Up/Down", Style::default().fg(theme.success)),
            Span::raw("  Select layer"),
        ]),
        Line::from(vec![
            Span::styled("Enter", Style::default().fg(theme.success)),
            Span::raw("    Switch to layer"),
        ]),
        Line::from(vec![
            Span::styled("n", Style::default().fg(theme.success)),
            Span::raw("        New layer"),
        ]),
        Line::from(vec![
            Span::styled("r", Style::default().fg(theme.success)),
            Span::raw("        Rename layer"),
        ]),
        Line::from(vec![
            Span::styled("d", Style::default().fg(theme.success)),
            Span::raw("        Delete layer (except 0)"),
        ]),
        Line::from(vec![
            Span::styled("u / j", Style::default().fg(theme.success)),
            Span::raw("  Move layer up/down"),
        ]),
        Line::from(vec![
            Span::styled("Esc", Style::default().fg(theme.success)),
            Span::raw("      Close"),
        ]),
    ];

    let help = Paragraph::new(help_lines)
        .alignment(Alignment::Left)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Help ")
                .border_style(Style::default().fg(theme.primary)),
        )
        .style(Style::default().fg(theme.text));

    f.render_widget(help, chunks[1]);
}

fn render_name_input(
    f: &mut Frame,
    area: Rect,
    title: &str,
    input: &str,
    prompt: &str,
    theme: &Theme,
) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(Style::default().fg(theme.primary));

    let inner = Rect {
        x: area.x + 2,
        y: area.y + 1,
        width: area.width.saturating_sub(4),
        height: area.height.saturating_sub(2),
    };

    f.render_widget(block, area);

    let lines = vec![
        Line::from(Span::styled(
            prompt,
            Style::default()
                .fg(theme.text)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            input,
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Enter to confirm, Esc to cancel",
            Style::default().fg(theme.text_muted),
        )),
    ];

    let paragraph = Paragraph::new(lines).alignment(Alignment::Left);
    f.render_widget(paragraph, inner);
}

fn render_delete_confirmation(f: &mut Frame, area: Rect, layer: &Layer, theme: &Theme) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Delete Layer ")
        .border_style(Style::default().fg(theme.warning));

    let inner = Rect {
        x: area.x + 2,
        y: area.y + 1,
        width: area.width.saturating_sub(4),
        height: area.height.saturating_sub(2),
    };

    f.render_widget(block, area);

    let lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            format!("Delete layer {}: '{}' ?", layer.number, layer.name),
            Style::default()
                .fg(theme.text)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "This action cannot be undone.",
            Style::default().fg(theme.text),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Press 'y' to confirm, 'n' or Esc to cancel",
            Style::default().fg(theme.text_muted),
        )),
    ];

    let paragraph = Paragraph::new(lines).alignment(Alignment::Left);
    f.render_widget(paragraph, inner);
}
