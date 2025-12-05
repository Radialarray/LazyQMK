//! Layer manager for CRUD operations on layers.
//!
//! Provides a UI for creating, renaming, reordering, toggling colors, and deleting layers.
//! Accessible via Shift+Y shortcut.

use ratatui::{
    layout::{Alignment, Constraint, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};

use crate::models::Layer;

/// Manager mode - determines what operation is being performed
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ManagerMode {
    /// Browsing layers (default mode)
    Browsing,
    /// Creating a new layer (entering name)
    CreatingName {
        /// User input for layer name
        input: String,
    },
    /// Renaming a layer
    Renaming {
        /// Index of layer being renamed
        layer_index: usize,
        /// User input for new name
        input: String,
    },
    /// Confirming deletion
    ConfirmingDelete {
        /// Index of layer to delete
        layer_index: usize,
    },
    /// Duplicating a layer (entering name for copy)
    Duplicating {
        /// Index of layer being duplicated
        source_index: usize,
        /// User input for new layer name
        input: String,
    },
    /// Copying all keys to another layer (selecting target)
    CopyingTo {
        /// Index of source layer
        source_index: usize,
        /// Currently selected target layer
        target_selected: usize,
    },
    /// Swapping two layers (selecting swap target)
    Swapping {
        /// Index of first layer (source)
        source_index: usize,
        /// Currently selected swap target
        target_selected: usize,
    },
}

/// State for the layer manager dialog
#[derive(Debug, Clone)]
pub struct LayerManagerState {
    /// Currently selected layer index
    pub selected: usize,
    /// Current operation mode
    pub mode: ManagerMode,
}

impl LayerManagerState {
    /// Create a new layer manager state
    #[must_use]
    pub const fn new() -> Self {
        Self {
            selected: 0,
            mode: ManagerMode::Browsing,
        }
    }

    /// Reset to default state, optionally setting selection to current layer
    pub fn reset(&mut self, current_layer: usize) {
        self.selected = current_layer;
        self.mode = ManagerMode::Browsing;
    }

    /// Move selection up
    pub const fn select_previous(&mut self, layer_count: usize) {
        if layer_count > 0 {
            if self.selected > 0 {
                self.selected -= 1;
            } else {
                self.selected = layer_count - 1;
            }
        }
    }

    /// Move selection down
    pub const fn select_next(&mut self, layer_count: usize) {
        if layer_count > 0 {
            self.selected = (self.selected + 1) % layer_count;
        }
    }

    /// Start creating a new layer
    pub fn start_creating(&mut self) {
        self.mode = ManagerMode::CreatingName {
            input: String::new(),
        };
    }

    /// Start renaming the selected layer
    pub fn start_renaming(&mut self, layer: &Layer) {
        self.mode = ManagerMode::Renaming {
            layer_index: self.selected,
            input: layer.name.clone(),
        };
    }

    /// Start confirming deletion of the selected layer
    pub fn start_deleting(&mut self) {
        self.mode = ManagerMode::ConfirmingDelete {
            layer_index: self.selected,
        };
    }

    /// Start duplicating the selected layer
    pub fn start_duplicating(&mut self, layer: &Layer) {
        self.mode = ManagerMode::Duplicating {
            source_index: self.selected,
            input: format!("{} (copy)", layer.name),
        };
    }

    /// Start copying to another layer
    pub fn start_copy_to(&mut self, layer_count: usize) {
        // Start with next layer as default target (or first if at end)
        let target = if self.selected + 1 < layer_count {
            self.selected + 1
        } else if self.selected > 0 {
            0
        } else {
            return; // Only one layer, can't copy to itself
        };
        self.mode = ManagerMode::CopyingTo {
            source_index: self.selected,
            target_selected: target,
        };
    }

    /// Start swapping with another layer
    pub fn start_swapping(&mut self, layer_count: usize) {
        // Start with next layer as default target (or first if at end)
        let target = if self.selected + 1 < layer_count {
            self.selected + 1
        } else if self.selected > 0 {
            0
        } else {
            return; // Only one layer, can't swap with itself
        };
        self.mode = ManagerMode::Swapping {
            source_index: self.selected,
            target_selected: target,
        };
    }

    /// Navigate in copy-to or swap mode
    pub const fn select_target_previous(&mut self, layer_count: usize) {
        match &mut self.mode {
            ManagerMode::CopyingTo {
                source_index,
                target_selected,
            }
            | ManagerMode::Swapping {
                source_index,
                target_selected,
            } => {
                // Skip the source layer when navigating
                loop {
                    if *target_selected > 0 {
                        *target_selected -= 1;
                    } else {
                        *target_selected = layer_count - 1;
                    }
                    if *target_selected != *source_index {
                        break;
                    }
                }
            }
            _ => {}
        }
    }

    /// Navigate in copy-to or swap mode
    pub const fn select_target_next(&mut self, layer_count: usize) {
        match &mut self.mode {
            ManagerMode::CopyingTo {
                source_index,
                target_selected,
            }
            | ManagerMode::Swapping {
                source_index,
                target_selected,
            } => {
                // Skip the source layer when navigating
                loop {
                    *target_selected = (*target_selected + 1) % layer_count;
                    if *target_selected != *source_index {
                        break;
                    }
                }
            }
            _ => {}
        }
    }

    /// Cancel current operation and return to browsing
    pub fn cancel(&mut self) {
        self.mode = ManagerMode::Browsing;
    }

    /// Check if we're in browsing mode
    #[allow(dead_code)]
    #[must_use]
    pub const fn is_browsing(&self) -> bool {
        matches!(self.mode, ManagerMode::Browsing)
    }

    /// Get the current input text (for name entry or renaming)
    #[must_use]
    pub fn get_input(&self) -> Option<&str> {
        match &self.mode {
            ManagerMode::CreatingName { input }
            | ManagerMode::Renaming { input, .. }
            | ManagerMode::Duplicating { input, .. } => Some(input),
            _ => None,
        }
    }

    /// Get mutable reference to current input text
    pub const fn get_input_mut(&mut self) -> Option<&mut String> {
        match &mut self.mode {
            ManagerMode::CreatingName { input }
            | ManagerMode::Renaming { input, .. }
            | ManagerMode::Duplicating { input, .. } => Some(input),
            _ => None,
        }
    }
}

impl Default for LayerManagerState {
    fn default() -> Self {
        Self::new()
    }
}

use super::Theme;

/// Render the layer manager dialog
pub fn render_layer_manager(
    f: &mut Frame,
    area: Rect,
    state: &LayerManagerState,
    layers: &[Layer],
    theme: &Theme,
) {
    // Center the dialog (80% width, 80% height)
    let dialog_width = (area.width * 80) / 100;
    let dialog_height = (area.height * 80) / 100;
    let dialog_x = (area.width - dialog_width) / 2;
    let dialog_y = (area.height - dialog_height) / 2;

    let dialog_area = Rect {
        x: dialog_x,
        y: dialog_y,
        width: dialog_width,
        height: dialog_height,
    };

    // Clear the background area first
    f.render_widget(Clear, dialog_area);

    // Background block
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Layer Manager (Shift+Y) ")
        .style(Style::default().bg(theme.background));

    f.render_widget(block, dialog_area);

    // Inner area for content
    let inner_area = Rect {
        x: dialog_area.x + 2,
        y: dialog_area.y + 1,
        width: dialog_area.width.saturating_sub(4),
        height: dialog_area.height.saturating_sub(2),
    };

    match &state.mode {
        ManagerMode::Browsing => {
            render_layer_list(f, inner_area, state, layers, theme);
        }
        ManagerMode::CreatingName { input } => {
            render_name_input(
                f,
                inner_area,
                "Create Layer",
                input,
                "Enter layer name:",
                theme,
            );
        }
        ManagerMode::Renaming { input, .. } => {
            render_name_input(
                f,
                inner_area,
                "Rename Layer",
                input,
                "Enter new name:",
                theme,
            );
        }
        ManagerMode::ConfirmingDelete { layer_index } => {
            if let Some(layer) = layers.get(*layer_index) {
                render_delete_confirmation(f, inner_area, *layer_index, layer, layers.len(), theme);
            }
        }
        ManagerMode::Duplicating {
            source_index,
            input,
        } => {
            if let Some(layer) = layers.get(*source_index) {
                render_name_input(
                    f,
                    inner_area,
                    &format!("Duplicate Layer: {}", layer.name),
                    input,
                    "Enter name for duplicate:",
                    theme,
                );
            }
        }
        ManagerMode::CopyingTo {
            source_index,
            target_selected,
        } => {
            render_layer_picker(
                f,
                inner_area,
                "Copy Keys To Layer",
                *source_index,
                *target_selected,
                layers,
                theme,
            );
        }
        ManagerMode::Swapping {
            source_index,
            target_selected,
        } => {
            render_layer_picker(
                f,
                inner_area,
                "Swap With Layer",
                *source_index,
                *target_selected,
                layers,
                theme,
            );
        }
    }
}

/// Render the list of layers
fn render_layer_list(
    f: &mut Frame,
    area: Rect,
    state: &LayerManagerState,
    layers: &[Layer],
    theme: &Theme,
) {
    // Split area for list and help text
    let chunks = ratatui::layout::Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([
            Constraint::Min(5),    // Layer list
            Constraint::Length(8), // Help text (more lines now)
        ])
        .split(area);

    // Render layer list
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

            // Show color indicator for layer default color
            let color = &layer.default_color;
            let color_box = Span::styled(
                "█████ ",
                Style::default().fg(Color::Rgb(color.r, color.g, color.b)),
            );

            // Show colors enabled/disabled indicator
            let colors_indicator = if layer.layer_colors_enabled {
                Span::styled(" ●", Style::default().fg(theme.success))
            } else {
                Span::styled(" ○", Style::default().fg(theme.text_muted))
            };

            let content = Line::from(vec![
                color_box,
                Span::styled(
                    format!("Layer {i}: "),
                    Style::default().fg(theme.text_muted),
                ),
                Span::styled(&layer.name, style),
                colors_indicator,
            ]);

            ListItem::new(content)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Layers"))
        .highlight_style(Style::default().bg(theme.surface));

    f.render_widget(list, chunks[0]);

    // Render help text
    let help_text = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("n", Style::default().fg(theme.primary)),
            Span::raw(": New  "),
            Span::styled("D", Style::default().fg(theme.primary)),
            Span::raw(": Duplicate  "),
            Span::styled("r", Style::default().fg(theme.primary)),
            Span::raw(": Rename  "),
            Span::styled("d", Style::default().fg(theme.primary)),
            Span::raw(": Delete"),
        ]),
        Line::from(vec![
            Span::styled("c", Style::default().fg(theme.primary)),
            Span::raw(": Copy to  "),
            Span::styled("s", Style::default().fg(theme.primary)),
            Span::raw(": Swap with  "),
            Span::styled("v", Style::default().fg(theme.primary)),
            Span::raw(": Toggle Colors"),
        ]),
        Line::from(vec![
            Span::styled("↑/↓", Style::default().fg(theme.primary)),
            Span::raw(": Navigate  "),
            Span::styled("Shift+↑/↓", Style::default().fg(theme.primary)),
            Span::raw(": Reorder  "),
            Span::styled("Enter", Style::default().fg(theme.primary)),
            Span::raw(": Go to"),
        ]),
        Line::from(vec![
            Span::styled("Esc", Style::default().fg(theme.primary)),
            Span::raw(": Close"),
        ]),
    ];

    let help = Paragraph::new(help_text)
        .block(Block::default().borders(Borders::ALL).title("Help"))
        .alignment(Alignment::Left);

    f.render_widget(help, chunks[1]);
}

/// Render name input dialog
fn render_name_input(
    f: &mut Frame,
    area: Rect,
    title: &str,
    input: &str,
    prompt: &str,
    theme: &Theme,
) {
    let chunks = ratatui::layout::Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Prompt
            Constraint::Length(3), // Input
            Constraint::Min(1),    // Spacer
            Constraint::Length(3), // Help
        ])
        .split(area);

    // Prompt
    let prompt_text = Paragraph::new(prompt)
        .alignment(Alignment::Center)
        .style(Style::default().fg(theme.text));
    f.render_widget(prompt_text, chunks[0]);

    // Input box
    let input_block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .style(Style::default().fg(theme.primary));

    let input_text = Paragraph::new(input)
        .block(input_block)
        .style(Style::default().fg(theme.text));

    f.render_widget(input_text, chunks[1]);

    // Help text
    let help = vec![Line::from(vec![
        Span::styled("Enter", Style::default().fg(theme.primary)),
        Span::raw(": Confirm  "),
        Span::styled("Esc", Style::default().fg(theme.primary)),
        Span::raw(": Cancel"),
    ])];

    let help_widget = Paragraph::new(help)
        .alignment(Alignment::Center)
        .style(Style::default().fg(theme.text_muted));

    f.render_widget(help_widget, chunks[3]);
}

/// Render delete confirmation dialog
fn render_delete_confirmation(
    f: &mut Frame,
    area: Rect,
    layer_index: usize,
    layer: &Layer,
    layer_count: usize,
    theme: &Theme,
) {
    let chunks = ratatui::layout::Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Warning
            Constraint::Length(3), // Layer info
            Constraint::Length(2), // Additional warning if needed
            Constraint::Min(1),    // Spacer
            Constraint::Length(3), // Help
        ])
        .split(area);

    // Warning
    let warning = Paragraph::new("Are you sure you want to delete this layer?")
        .alignment(Alignment::Center)
        .style(
            Style::default()
                .fg(theme.error)
                .add_modifier(Modifier::BOLD),
        );
    f.render_widget(warning, chunks[0]);

    // Layer info
    let info = Line::from(vec![
        Span::raw("Layer "),
        Span::styled(format!("{layer_index}"), Style::default().fg(theme.accent)),
        Span::raw(": "),
        Span::styled(&layer.name, Style::default().fg(theme.accent)),
    ]);

    let info_widget = Paragraph::new(info)
        .alignment(Alignment::Center)
        .style(Style::default().fg(theme.text));
    f.render_widget(info_widget, chunks[1]);

    // Show warning if this is the last layer
    if layer_count <= 1 {
        let last_layer_warning = Paragraph::new("Cannot delete the last layer!")
            .alignment(Alignment::Center)
            .style(
                Style::default()
                    .fg(theme.warning)
                    .add_modifier(Modifier::BOLD),
            );
        f.render_widget(last_layer_warning, chunks[2]);
    }

    // Help
    let help = if layer_count <= 1 {
        vec![Line::from(vec![
            Span::styled("Esc", Style::default().fg(theme.primary)),
            Span::raw(": Cancel"),
        ])]
    } else {
        vec![Line::from(vec![
            Span::styled("y", Style::default().fg(theme.primary)),
            Span::raw(": Yes, delete  "),
            Span::styled("n/Esc", Style::default().fg(theme.primary)),
            Span::raw(": No, cancel"),
        ])]
    };

    let help_widget = Paragraph::new(help)
        .alignment(Alignment::Center)
        .style(Style::default().fg(theme.text_muted));

    f.render_widget(help_widget, chunks[4]);
}

/// Render layer picker for copy-to or swap operations
fn render_layer_picker(
    f: &mut Frame,
    area: Rect,
    title: &str,
    source_index: usize,
    target_selected: usize,
    layers: &[Layer],
    theme: &Theme,
) {
    let chunks = ratatui::layout::Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([
            Constraint::Length(2), // Title/source info
            Constraint::Min(5),    // Target layer list
            Constraint::Length(3), // Help
        ])
        .split(area);

    // Source info
    let source_name = layers.get(source_index).map_or("?", |l| l.name.as_str());
    let info = Paragraph::new(format!("From Layer {source_index}: {source_name}"))
        .alignment(Alignment::Center)
        .style(Style::default().fg(theme.text));
    f.render_widget(info, chunks[0]);

    // Target layer list
    let items: Vec<ListItem> = layers
        .iter()
        .enumerate()
        .filter(|(i, _)| *i != source_index) // Exclude source layer
        .map(|(i, layer)| {
            let is_selected = i == target_selected;
            let style = if is_selected {
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.text)
            };

            let prefix = if is_selected { "→ " } else { "  " };
            let content = Line::from(vec![
                Span::raw(prefix),
                Span::styled(format!("Layer {i}: "), Style::default().fg(theme.text_muted)),
                Span::styled(&layer.name, style),
            ]);

            ListItem::new(content)
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!("Select Target - {title}")),
        )
        .highlight_style(Style::default().bg(theme.surface));

    f.render_widget(list, chunks[1]);

    // Help text
    let help = vec![Line::from(vec![
        Span::styled("↑/↓", Style::default().fg(theme.primary)),
        Span::raw(": Select  "),
        Span::styled("Enter", Style::default().fg(theme.primary)),
        Span::raw(": Confirm  "),
        Span::styled("Esc", Style::default().fg(theme.primary)),
        Span::raw(": Cancel"),
    ])];

    let help_widget = Paragraph::new(help)
        .alignment(Alignment::Center)
        .style(Style::default().fg(theme.text_muted));

    f.render_widget(help_widget, chunks[2]);
}
