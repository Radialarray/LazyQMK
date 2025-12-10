//! Configuration dialogs for layout selection.
//!
//! This module implements popup dialogs for layout variant selection.

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};
use std::path::PathBuf;

use crate::parser::keyboard_json::{
    extract_layout_variants, parse_keyboard_info_json, LayoutVariant,
};

/// Layout picker dialog state
#[derive(Debug, Clone)]
pub struct LayoutPickerState {
    /// Available layouts with key counts for current keyboard
    pub layouts: Vec<LayoutVariant>,
    /// Selected index
    pub selected_index: usize,
    /// Error message
    pub error_message: Option<String>,
}

impl LayoutPickerState {
    /// Creates a new layout picker
    #[must_use]
    pub const fn new() -> Self {
        Self {
            layouts: Vec::new(),
            selected_index: 0,
            error_message: None,
        }
    }

    /// Loads layouts for a specific keyboard
    pub fn load_layouts(
        &mut self,
        qmk_path: &PathBuf,
        keyboard: &str,
    ) -> Result<(), anyhow::Error> {
        match parse_keyboard_info_json(qmk_path, keyboard) {
            Ok(info) => {
                self.layouts = extract_layout_variants(&info);
                self.selected_index = 0;
                Ok(())
            }
            Err(e) => {
                self.error_message = Some(format!("Failed to load layouts: {e}"));
                Err(e)
            }
        }
    }

    /// Gets the currently selected layout name
    #[must_use]
    pub fn get_selected(&self) -> Option<String> {
        self.layouts
            .get(self.selected_index)
            .map(|v| v.name.clone())
    }

    /// Moves selection up
    pub const fn move_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    /// Moves selection down
    pub const fn move_down(&mut self) {
        if self.selected_index < self.layouts.len().saturating_sub(1) {
            self.selected_index += 1;
        }
    }
}

impl Default for LayoutPickerState {
    fn default() -> Self {
        Self::new()
    }
}

/// Events emitted by the LayoutPicker component (keyboard layout variant picker)
#[derive(Debug, Clone)]
pub enum LayoutPickerEvent {
    /// User selected a layout variant
    LayoutSelected(String),
    /// User cancelled the picker
    Cancelled,
}

/// LayoutPicker component that implements the Component trait (for keyboard layout variants)
#[derive(Debug, Clone)]
pub struct LayoutPicker {
    /// Internal state of the layout picker
    state: LayoutPickerState,
    /// Keyboard name for display
    keyboard: String,
}

impl LayoutPicker {
    /// Create a new LayoutPicker with layouts pre-loaded
    #[must_use]
    pub fn new(qmk_path: &PathBuf, keyboard: &str) -> Self {
        let mut state = LayoutPickerState::new();
        // Attempt to load layouts on creation (ignore errors)
        let _ = state.load_layouts(qmk_path, keyboard);
        Self {
            state,
            keyboard: keyboard.to_string(),
        }
    }
}

impl crate::tui::component::Component for LayoutPicker {
    type Event = LayoutPickerEvent;

    fn handle_input(&mut self, key: crossterm::event::KeyEvent) -> Option<Self::Event> {
        use crossterm::event::KeyCode;

        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                self.state.move_up();
                None
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.state.move_down();
                None
            }
            KeyCode::Enter => self
                .state
                .get_selected()
                .map(LayoutPickerEvent::LayoutSelected),
            KeyCode::Esc => Some(LayoutPickerEvent::Cancelled),
            _ => None,
        }
    }

    fn render(&self, f: &mut Frame, _area: Rect, theme: &Theme) {
        render_layout_picker_component(f, self, theme);
    }
}

/// Renders the layout picker dialog (for Component)
fn render_layout_picker_component(f: &mut Frame, picker: &LayoutPicker, theme: &Theme) {
    let state = &picker.state;
    let keyboard = &picker.keyboard;
    let area = centered_rect(70, 50, f.area());

    // Clear the background area first
    f.render_widget(Clear, area);

    // Render opaque background
    let background = Block::default().style(Style::default().bg(theme.background));
    f.render_widget(background, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Min(5),    // List
            Constraint::Length(2), // Instructions
        ])
        .split(area);

    // Layout list with key counts
    let items: Vec<ListItem> = state
        .layouts
        .iter()
        .enumerate()
        .map(|(i, variant)| {
            let style = if i == state.selected_index {
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            // Display layout name with key count
            let display = format!("{} ({} keys)", variant.name, variant.key_count);
            ListItem::new(display).style(style)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!("Layouts for {keyboard}")),
    );
    f.render_widget(list, chunks[0]);

    // Instructions
    let instructions = Paragraph::new("↑↓: Navigate | Enter: Select | Esc: Cancel")
        .style(Style::default().fg(theme.text_muted))
        .alignment(Alignment::Center);
    f.render_widget(instructions, chunks[1]);
}

use super::Theme;

/// Helper to create a centered rect
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
