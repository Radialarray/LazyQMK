//! Layer manager for CRUD operations on layers.
//!
//! Provides a UI for creating, renaming, reordering, toggling colors, and deleting layers.
//! Accessible via Shift+Y shortcut.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Alignment, Constraint, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};

use crate::models::{Layer, Position, RgbColor};
use crate::tui::component::Component;
use crate::tui::Theme;

/// Events emitted by the LayerManager component
#[derive(Debug, Clone)]
pub enum LayerManagerEvent {
    /// User created a new layer
    LayerAdded {
        /// The new layer
        layer: Layer,
    },
    /// User deleted a layer
    LayerDeleted {
        /// Index of the deleted layer
        index: usize,
    },
    /// User renamed a layer
    LayerRenamed {
        /// Index of the renamed layer
        index: usize,
        /// New name
        name: String,
    },
    /// User reordered layers (swap)
    LayerReordered {
        /// Original index
        from: usize,
        /// New index
        to: usize,
    },
    /// User duplicated a layer
    LayerDuplicated {
        /// Source layer index
        source_index: usize,
        /// The new layer
        layer: Layer,
    },
    /// User copied keys from one layer to another
    LayerKeysCopied {
        /// Source layer index
        from: usize,
        /// Target layer index
        to: usize,
        /// Keys to copy (position, keycode, color, category)
        keys: Vec<(Position, String, Option<RgbColor>, Option<String>)>,
    },
    /// User swapped keys between two layers
    LayersSwapped {
        /// First layer index
        layer1: usize,
        /// Second layer index
        layer2: usize,
        /// Keys from layer1 (position, keycode, color, category)
        keys1: Vec<(Position, String, Option<RgbColor>, Option<String>)>,
        /// Keys from layer2 (position, keycode, color, category)
        keys2: Vec<(Position, String, Option<RgbColor>, Option<String>)>,
    },
    /// User toggled layer colors
    LayerColorsToggled {
        /// Index of the layer
        index: usize,
        /// New enabled state
        enabled: bool,
    },
    /// User wants to switch to a layer
    LayerSwitched {
        /// Index of the layer to switch to
        index: usize,
    },
    /// User cancelled without making changes
    Cancelled,
    /// Component closed naturally
    Closed,
}

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

/// LayerManager component that implements the Component trait
#[derive(Debug, Clone)]
pub struct LayerManager {
    /// Internal state of the layer manager
    state: LayerManagerState,
    /// Layers to display and modify (cached copy)
    cached_layers: Vec<Layer>,
}

impl LayerManager {
    /// Create a new LayerManager with initial layers
    #[must_use]
    pub fn new(layers: Vec<Layer>, current_layer: usize) -> Self {
        let mut state = LayerManagerState::new();
        state.reset(current_layer);
        Self {
            state,
            cached_layers: layers,
        }
    }

    /// Update the layers list (needed after CRUD operations)
    pub fn set_layers(&mut self, layers: Vec<Layer>) {
        self.cached_layers = layers;
        // Clamp selection to valid range
        if self.state.selected >= self.cached_layers.len() && !self.cached_layers.is_empty() {
            self.state.selected = self.cached_layers.len() - 1;
        }
    }


}

impl Component for LayerManager {
    type Event = LayerManagerEvent;

    fn handle_input(&mut self, key: KeyEvent) -> Option<Self::Event> {
        match &self.state.mode {
            ManagerMode::Browsing => self.handle_browsing_input(key),
            ManagerMode::CreatingName { .. }
            | ManagerMode::Renaming { .. }
            | ManagerMode::Duplicating { .. } => self.handle_text_input(key),
            ManagerMode::ConfirmingDelete { layer_index } => {
                self.handle_delete_confirmation(key, *layer_index)
            }
            ManagerMode::CopyingTo {
                source_index,
                target_selected,
            } => self.handle_copy_to_input(key, *source_index, *target_selected),
            ManagerMode::Swapping {
                source_index,
                target_selected,
            } => self.handle_swapping_input(key, *source_index, *target_selected),
        }
    }

    fn render(&self, f: &mut Frame, area: Rect, theme: &Theme) {
        render_layer_manager(f, area, &self.state, &self.cached_layers, theme);
    }
}

impl LayerManager {
    /// Handle input in browsing mode
    fn handle_browsing_input(&mut self, key: KeyEvent) -> Option<LayerManagerEvent> {
        match key.code {
            KeyCode::Esc => {
                self.state.reset(self.state.selected);
                Some(LayerManagerEvent::Closed)
            }
            KeyCode::Up if key.modifiers.contains(KeyModifiers::SHIFT) => {
                // Move layer up (reorder)
                let selected = self.state.selected;
                if selected > 0 && self.cached_layers.len() > 1 {
                    self.state.selected = selected - 1;
                    Some(LayerManagerEvent::LayerReordered {
                        from: selected,
                        to: selected - 1,
                    })
                } else {
                    None
                }
            }
            KeyCode::Down if key.modifiers.contains(KeyModifiers::SHIFT) => {
                // Move layer down (reorder)
                let selected = self.state.selected;
                if selected < self.cached_layers.len() - 1 {
                    self.state.selected = selected + 1;
                    Some(LayerManagerEvent::LayerReordered {
                        from: selected,
                        to: selected + 1,
                    })
                } else {
                    None
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.state.select_previous(self.cached_layers.len());
                None
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.state.select_next(self.cached_layers.len());
                None
            }
            KeyCode::Enter => {
                // Switch to selected layer
                Some(LayerManagerEvent::LayerSwitched {
                    index: self.state.selected,
                })
            }
            KeyCode::Char('n') => {
                // Start creating new layer
                self.state.start_creating();
                None
            }
            KeyCode::Char('r') => {
                // Start renaming
                if let Some(layer) = self.cached_layers.get(self.state.selected) {
                    self.state.start_renaming(layer);
                }
                None
            }
            KeyCode::Char('v') => {
                // Toggle layer colors
                let selected_idx = self.state.selected;
                if let Some(layer) = self.cached_layers.get(selected_idx) {
                    let new_enabled = !layer.layer_colors_enabled;
                    Some(LayerManagerEvent::LayerColorsToggled {
                        index: selected_idx,
                        enabled: new_enabled,
                    })
                } else {
                    None
                }
            }
            KeyCode::Char('d') => {
                // Start delete confirmation
                if self.cached_layers.len() <= 1 {
                    None // Can't delete last layer
                } else {
                    self.state.start_deleting();
                    None
                }
            }
            KeyCode::Char('D') if key.modifiers.contains(KeyModifiers::SHIFT) => {
                // Start duplicating
                if let Some(layer) = self.cached_layers.get(self.state.selected) {
                    self.state.start_duplicating(layer);
                }
                None
            }
            KeyCode::Char('c') => {
                // Start copy to another layer
                if self.cached_layers.len() <= 1 {
                    None // Need at least 2 layers
                } else {
                    self.state.start_copy_to(self.cached_layers.len());
                    None
                }
            }
            KeyCode::Char('s') => {
                // Start swap with another layer
                if self.cached_layers.len() <= 1 {
                    None // Need at least 2 layers
                } else {
                    self.state.start_swapping(self.cached_layers.len());
                    None
                }
            }
            _ => None,
        }
    }

    /// Handle text input (for creating, renaming, duplicating)
    fn handle_text_input(&mut self, key: KeyEvent) -> Option<LayerManagerEvent> {
        match key.code {
            KeyCode::Esc => {
                self.state.cancel();
                Some(LayerManagerEvent::Cancelled)
            }
            KeyCode::Enter => {
                if let Some(input) = self.state.get_input() {
                    let input = input.to_string();

                    if input.trim().is_empty() {
                        return None; // Don't process empty input
                    }

                    match &self.state.mode {
                        ManagerMode::CreatingName { .. } => {
                            // Create new layer
                            let new_index = self.cached_layers.len();
                            let default_color = RgbColor::new(128, 128, 128); // Gray-500

                            if let Ok(mut new_layer) =
                                Layer::new(new_index as u8, &input, default_color)
                            {
                                // Copy key positions from first layer (with transparent keycodes)
                                if let Some(first_layer) = self.cached_layers.first() {
                                    for key in &first_layer.keys {
                                        use crate::models::layer::KeyDefinition;
                                        new_layer
                                            .add_key(KeyDefinition::new(key.position, "KC_TRNS"));
                                    }
                                }

                                self.state.cancel();
                                Some(LayerManagerEvent::LayerAdded { layer: new_layer })
                            } else {
                                None
                            }
                        }
                        ManagerMode::Renaming { layer_index, .. } => {
                            let layer_index = *layer_index;
                            self.state.cancel();
                            Some(LayerManagerEvent::LayerRenamed {
                                index: layer_index,
                                name: input,
                            })
                        }
                        ManagerMode::Duplicating { source_index, .. } => {
                            let source_index = *source_index;
                            let new_index = self.cached_layers.len();

                            if let Some(source) = self.cached_layers.get(source_index) {
                                if let Ok(mut new_layer) =
                                    Layer::new(new_index as u8, &input, source.default_color)
                                {
                                    // Copy all keys from source
                                    for key in &source.keys {
                                        use crate::models::layer::KeyDefinition;
                                        let mut new_key =
                                            KeyDefinition::new(key.position, &key.keycode);
                                        new_key.color_override = key.color_override;
                                        new_key.category_id = key.category_id.clone();
                                        new_layer.add_key(new_key);
                                    }
                                    // Copy layer settings
                                    new_layer.layer_colors_enabled = source.layer_colors_enabled;
                                    new_layer.category_id = source.category_id.clone();

                                    self.state.cancel();
                                    Some(LayerManagerEvent::LayerDuplicated {
                                        source_index,
                                        layer: new_layer,
                                    })
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        }
                        _ => None,
                    }
                } else {
                    None
                }
            }
            KeyCode::Char(c) => {
                if let Some(input) = self.state.get_input_mut() {
                    input.push(c);
                }
                None
            }
            KeyCode::Backspace => {
                if let Some(input) = self.state.get_input_mut() {
                    input.pop();
                }
                None
            }
            _ => None,
        }
    }

    /// Handle delete confirmation input
    fn handle_delete_confirmation(
        &mut self,
        key: KeyEvent,
        layer_index: usize,
    ) -> Option<LayerManagerEvent> {
        match key.code {
            KeyCode::Char('y' | 'Y') => {
                if self.cached_layers.len() > 1 {
                    self.state.cancel();
                    Some(LayerManagerEvent::LayerDeleted { index: layer_index })
                } else {
                    // Can't delete last layer
                    self.state.cancel();
                    None
                }
            }
            KeyCode::Char('n' | 'N') | KeyCode::Esc => {
                self.state.cancel();
                Some(LayerManagerEvent::Cancelled)
            }
            _ => None,
        }
    }

    /// Handle copy-to mode input
    fn handle_copy_to_input(
        &mut self,
        key: KeyEvent,
        source_index: usize,
        target_selected: usize,
    ) -> Option<LayerManagerEvent> {
        match key.code {
            KeyCode::Esc => {
                self.state.cancel();
                Some(LayerManagerEvent::Cancelled)
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.state.select_target_previous(self.cached_layers.len());
                None
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.state.select_target_next(self.cached_layers.len());
                None
            }
            KeyCode::Enter => {
                // Copy all keys from source to target
                if let Some(source) = self.cached_layers.get(source_index) {
                    let keys: Vec<_> = source
                        .keys
                        .iter()
                        .map(|k| {
                            (
                                k.position,
                                k.keycode.clone(),
                                k.color_override,
                                k.category_id.clone(),
                            )
                        })
                        .collect();

                    self.state.cancel();
                    Some(LayerManagerEvent::LayerKeysCopied {
                        from: source_index,
                        to: target_selected,
                        keys,
                    })
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Handle swapping mode input
    fn handle_swapping_input(
        &mut self,
        key: KeyEvent,
        source_index: usize,
        target_selected: usize,
    ) -> Option<LayerManagerEvent> {
        match key.code {
            KeyCode::Esc => {
                self.state.cancel();
                Some(LayerManagerEvent::Cancelled)
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.state.select_target_previous(self.cached_layers.len());
                None
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.state.select_target_next(self.cached_layers.len());
                None
            }
            KeyCode::Enter => {
                // Swap all keys between source and target
                if let (Some(source), Some(target)) = (
                    self.cached_layers.get(source_index),
                    self.cached_layers.get(target_selected),
                ) {
                    let keys1: Vec<_> = source
                        .keys
                        .iter()
                        .map(|k| {
                            (
                                k.position,
                                k.keycode.clone(),
                                k.color_override,
                                k.category_id.clone(),
                            )
                        })
                        .collect();
                    let keys2: Vec<_> = target
                        .keys
                        .iter()
                        .map(|k| {
                            (
                                k.position,
                                k.keycode.clone(),
                                k.color_override,
                                k.category_id.clone(),
                            )
                        })
                        .collect();

                    self.state.cancel();
                    Some(LayerManagerEvent::LayersSwapped {
                        layer1: source_index,
                        layer2: target_selected,
                        keys1,
                        keys2,
                    })
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

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
