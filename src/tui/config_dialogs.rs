//! Configuration dialogs for path, keyboard, and layout selection.
//!
//! This module implements popup dialogs for runtime configuration changes
//! accessible via keyboard shortcuts (Ctrl+P, Ctrl+K, Ctrl+Y).

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};
use std::path::PathBuf;

use crate::parser::keyboard_json::{
    extract_layout_variants, parse_keyboard_info_json, scan_keyboards,
    LayoutVariant,
};

/// Path configuration dialog state
#[derive(Debug, Clone)]
pub struct PathConfigDialogState {
    /// Current input buffer
    pub input_buffer: String,
    /// Error message if validation fails
    pub error_message: Option<String>,
}

impl PathConfigDialogState {
    /// Creates a new path configuration dialog
    pub fn new(current_path: Option<&PathBuf>) -> Self {
        let input_buffer = current_path
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();

        Self {
            input_buffer,
            error_message: None,
        }
    }

    /// Validates and returns the path
    pub fn validate(&mut self) -> Result<PathBuf, String> {
        if self.input_buffer.is_empty() {
            return Err("Path cannot be empty".to_string());
        }

        let path = PathBuf::from(&self.input_buffer);
        if !path.exists() {
            return Err(format!("Path does not exist: {}", path.display()));
        }

        if !path.join("Makefile").exists() {
            return Err("Not a QMK firmware directory: Makefile not found".to_string());
        }

        if !path.join("keyboards").is_dir() {
            return Err("Not a QMK firmware directory: keyboards/ not found".to_string());
        }

        Ok(path)
    }
}

/// Keyboard picker dialog state
#[derive(Debug, Clone)]
pub struct KeyboardPickerState {
    /// Available keyboards
    pub keyboards: Vec<String>,
    /// Search/filter text
    pub search_query: String,
    /// Filtered keyboards matching search
    pub filtered_keyboards: Vec<String>,
    /// Selected index in filtered list
    pub selected_index: usize,
    /// Scroll offset for viewport
    pub scroll_offset: usize,
    /// Error message
    pub error_message: Option<String>,
}

impl KeyboardPickerState {
    /// Creates a new keyboard picker
    pub fn new() -> Self {
        Self {
            keyboards: Vec::new(),
            search_query: String::new(),
            filtered_keyboards: Vec::new(),
            selected_index: 0,
            scroll_offset: 0,
            error_message: None,
        }
    }

    /// Loads keyboards from QMK firmware path
    pub fn load_keyboards(&mut self, qmk_path: &PathBuf) -> Result<()> {
        match scan_keyboards(qmk_path) {
            Ok(keyboards) => {
                self.keyboards = keyboards;
                self.update_filter();
                Ok(())
            }
            Err(e) => {
                self.error_message = Some(format!("Failed to scan keyboards: {}", e));
                Err(e)
            }
        }
    }

    /// Updates the filter based on search query
    pub fn update_filter(&mut self) {
        if self.search_query.is_empty() {
            self.filtered_keyboards = self.keyboards.clone();
        } else {
            let query_lower = self.search_query.to_lowercase();
            self.filtered_keyboards = self
                .keyboards
                .iter()
                .filter(|kb| kb.to_lowercase().contains(&query_lower))
                .cloned()
                .collect();
        }

        // Reset selection if out of bounds
        if self.selected_index >= self.filtered_keyboards.len() {
            self.selected_index = 0;
        }
    }

    /// Gets the currently selected keyboard
    pub fn get_selected(&self) -> Option<String> {
        self.filtered_keyboards.get(self.selected_index).cloned()
    }

    /// Moves selection up
    pub fn move_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
            self.update_scroll();
        }
    }

    /// Moves selection down
    pub fn move_down(&mut self) {
        if self.selected_index < self.filtered_keyboards.len().saturating_sub(1) {
            self.selected_index += 1;
            self.update_scroll();
        }
    }

    /// Updates scroll offset to keep selection visible
    fn update_scroll(&mut self) {
        let viewport_height = 15; // Approximate visible items
        if self.selected_index < self.scroll_offset {
            self.scroll_offset = self.selected_index;
        } else if self.selected_index >= self.scroll_offset + viewport_height {
            self.scroll_offset = self.selected_index - viewport_height + 1;
        }
    }
}

impl Default for KeyboardPickerState {
    fn default() -> Self {
        Self::new()
    }
}

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
    pub fn new() -> Self {
        Self {
            layouts: Vec::new(),
            selected_index: 0,
            error_message: None,
        }
    }

    /// Loads layouts for a specific keyboard
    pub fn load_layouts(&mut self, qmk_path: &PathBuf, keyboard: &str) -> Result<()> {
        match parse_keyboard_info_json(qmk_path, keyboard) {
            Ok(info) => {
                self.layouts = extract_layout_variants(&info);
                self.selected_index = 0;
                Ok(())
            }
            Err(e) => {
                self.error_message = Some(format!("Failed to load layouts: {}", e));
                Err(e)
            }
        }
    }

    /// Gets the currently selected layout name
    pub fn get_selected(&self) -> Option<String> {
        self.layouts
            .get(self.selected_index)
            .map(|v| v.name.clone())
    }

    /// Gets the currently selected layout variant (with key count)
    pub fn get_selected_variant(&self) -> Option<&LayoutVariant> {
        self.layouts.get(self.selected_index)
    }

    /// Moves selection up
    pub fn move_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    /// Moves selection down
    pub fn move_down(&mut self) {
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

/// Renders the path configuration dialog
pub fn render_path_dialog(f: &mut Frame, state: &PathConfigDialogState) {
    let area = centered_rect(60, 40, f.size());

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Length(3), // Input
            Constraint::Length(4), // Help text
            Constraint::Length(2), // Error
        ])
        .split(area);

    // Title
    let title = Paragraph::new("Configure QMK Firmware Path")
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    // Input field
    let input = Paragraph::new(format!("> {}_", state.input_buffer))
        .style(Style::default().fg(Color::Yellow))
        .block(Block::default().borders(Borders::ALL).title("Path"));
    f.render_widget(input, chunks[1]);

    // Help text
    let help = Paragraph::new("Enter: Save | Esc: Cancel | Backspace: Delete")
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    f.render_widget(help, chunks[2]);

    // Error message
    if let Some(error) = &state.error_message {
        let error_widget = Paragraph::new(error.as_str())
            .style(Style::default().fg(Color::Red))
            .alignment(Alignment::Center);
        f.render_widget(error_widget, chunks[3]);
    }
}

/// Renders the keyboard picker dialog
pub fn render_keyboard_picker(f: &mut Frame, state: &KeyboardPickerState) {
    let area = centered_rect(70, 80, f.size());

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3), // Search box
            Constraint::Min(10),   // List
            Constraint::Length(2), // Instructions
        ])
        .split(area);

    // Search box
    let search = Paragraph::new(format!("Search: {}_", state.search_query))
        .style(Style::default().fg(Color::Yellow))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(search, chunks[0]);

    // Keyboard list
    let items: Vec<ListItem> = state
        .filtered_keyboards
        .iter()
        .enumerate()
        .skip(state.scroll_offset)
        .map(|(i, kb)| {
            let style = if i == state.selected_index {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            ListItem::new(kb.as_str()).style(style)
        })
        .collect();

    let list = List::new(items).block(Block::default().borders(Borders::ALL).title(format!(
        "Keyboards ({}/{})",
        state.filtered_keyboards.len(),
        state.keyboards.len()
    )));
    f.render_widget(list, chunks[1]);

    // Instructions
    let instructions = Paragraph::new("↑↓: Navigate | Enter: Select | Type: Filter | Esc: Cancel")
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    f.render_widget(instructions, chunks[2]);
}

/// Renders the layout picker dialog
pub fn render_layout_picker(f: &mut Frame, state: &LayoutPickerState, keyboard: &str) {
    let area = centered_rect(70, 50, f.size());

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
                    .fg(Color::Yellow)
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
            .title(format!("Layouts for {}", keyboard)),
    );
    f.render_widget(list, chunks[0]);

    // Instructions
    let instructions = Paragraph::new("↑↓: Navigate | Enter: Select | Esc: Cancel")
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    f.render_widget(instructions, chunks[1]);
}

/// Handles input for path configuration dialog
pub fn handle_path_dialog_input(
    state: &mut PathConfigDialogState,
    key: KeyEvent,
) -> Result<Option<PathBuf>> {
    match key.code {
        KeyCode::Enter => match state.validate() {
            Ok(path) => Ok(Some(path)),
            Err(e) => {
                state.error_message = Some(e);
                Ok(None)
            }
        },
        KeyCode::Backspace => {
            state.input_buffer.pop();
            state.error_message = None;
            Ok(None)
        }
        KeyCode::Char(c) => {
            state.input_buffer.push(c);
            state.error_message = None;
            Ok(None)
        }
        KeyCode::Esc => Ok(Some(PathBuf::new())), // Return empty to signal cancel
        _ => Ok(None),
    }
}

/// Handles input for keyboard picker
pub fn handle_keyboard_picker_input(
    state: &mut KeyboardPickerState,
    key: KeyEvent,
) -> Option<String> {
    match key.code {
        KeyCode::Up => {
            state.move_up();
            None
        }
        KeyCode::Down => {
            state.move_down();
            None
        }
        KeyCode::Enter => state.get_selected(),
        KeyCode::Backspace => {
            state.search_query.pop();
            state.update_filter();
            None
        }
        KeyCode::Char(c) => {
            state.search_query.push(c);
            state.update_filter();
            None
        }
        KeyCode::Esc => Some(String::new()), // Empty string signals cancel
        _ => None,
    }
}

/// Handles input for layout picker
pub fn handle_layout_picker_input(state: &mut LayoutPickerState, key: KeyEvent) -> Option<String> {
    match key.code {
        KeyCode::Up => {
            state.move_up();
            None
        }
        KeyCode::Down => {
            state.move_down();
            None
        }
        KeyCode::Enter => state.get_selected(),
        KeyCode::Esc => Some(String::new()), // Empty string signals cancel
        _ => None,
    }
}

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
