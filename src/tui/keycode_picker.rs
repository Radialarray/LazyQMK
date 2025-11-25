//! Keycode picker dialog for selecting keycodes

use anyhow::Result;
use crossterm::event::{self, KeyCode};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};

use super::AppState;

/// Keycode picker state
#[derive(Debug, Clone)]
pub struct KeycodePickerState {
    /// Search query string
    pub search: String,
    /// Selected keycode index
    pub selected: usize,
    /// Active category filter (if any)
    pub active_category: Option<String>,
}

impl Default for KeycodePickerState {
    fn default() -> Self {
        Self::new()
    }
}

impl KeycodePickerState {
    /// Creates a new keycode picker state
    #[must_use] pub const fn new() -> Self {
        Self {
            search: String::new(),
            selected: 0,
            active_category: None,
        }
    }

    /// Reset to initial state
    pub fn reset(&mut self) {
        self.search.clear();
        self.selected = 0;
        self.active_category = None;
    }
}

/// Render the keycode picker popup
pub fn render_keycode_picker(f: &mut Frame, state: &AppState) {
    let area = centered_rect(60, 70, f.size());

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Search box
            Constraint::Length(3), // Category info
            Constraint::Min(10),   // Keycode list
            Constraint::Length(3), // Help text
        ])
        .split(area);

    // Search box
    let search_text = vec![Line::from(vec![
        Span::raw("Search: "),
        Span::styled(
            &state.keycode_picker_state.search,
            Style::default().add_modifier(Modifier::BOLD),
        ),
    ])];
    let search = Paragraph::new(search_text).block(Block::default().borders(Borders::ALL));
    f.render_widget(search, chunks[0]);

    // Category info
    let category_text = if let Some(cat_id) = &state.keycode_picker_state.active_category {
        if let Some(cat) = state.keycode_db.get_category(cat_id) {
            format!("Category: {} - {}", cat.name, cat.description)
        } else {
            "All Categories".to_string()
        }
    } else {
        "All Categories (press 1-8 to filter)".to_string()
    };
    let category = Paragraph::new(category_text).block(Block::default().borders(Borders::ALL));
    f.render_widget(category, chunks[1]);

    // Get filtered keycodes based on search and category
    let keycodes = if let Some(cat_id) = &state.keycode_picker_state.active_category {
        state
            .keycode_db
            .search_in_category(&state.keycode_picker_state.search, cat_id)
    } else {
        state.keycode_db.search(&state.keycode_picker_state.search)
    };

    // Build list items
    let list_items: Vec<ListItem> = keycodes
        .iter()
        .map(|keycode| {
            let display = format!("{} - {}", keycode.code, keycode.name);
            ListItem::new(display)
        })
        .collect();

    // Create list widget with stateful selection
    let list = List::new(list_items)
        .block(
            Block::default()
                .title(format!(" Keycodes ({}) ", keycodes.len()))
                .borders(Borders::ALL),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    // Create list state for highlighting
    let mut list_state = ListState::default();
    list_state.select(Some(
        state
            .keycode_picker_state
            .selected
            .min(keycodes.len().saturating_sub(1)),
    ));

    f.render_stateful_widget(list, chunks[2], &mut list_state);

    // Help text
    let help = Paragraph::new(
        "↑↓: Navigate | Enter: Select | Esc: Cancel | 1-8: Filter by category | Type to search",
    )
    .block(Block::default().borders(Borders::ALL));
    f.render_widget(help, chunks[3]);
}

/// Handle input for keycode picker
pub fn handle_input(state: &mut AppState, key: event::KeyEvent) -> Result<bool> {
    match key.code {
        KeyCode::Esc => {
            state.active_popup = None;
            state.keycode_picker_state.reset();
            state.set_status("Cancelled");
            Ok(false)
        }
        KeyCode::Enter => {
            // Get filtered keycodes and clone the selected one to avoid borrow checker issues
            let keycodes = if let Some(cat_id) = &state.keycode_picker_state.active_category {
                state
                    .keycode_db
                    .search_in_category(&state.keycode_picker_state.search, cat_id)
            } else {
                state.keycode_db.search(&state.keycode_picker_state.search)
            };

            let selected_keycode_opt = keycodes
                .get(state.keycode_picker_state.selected)
                .map(|kc| kc.code.clone());

            // Select current keycode and close
            if let Some(keycode) = selected_keycode_opt {
                if let Some(selected_key) = state.get_selected_key_mut() {
                    selected_key.keycode = keycode.clone();
                    state.mark_dirty();
                    state.set_status(format!("Keycode assigned: {keycode}"));
                }
            }

            state.active_popup = None;
            state.keycode_picker_state.reset();
            Ok(false)
        }
        KeyCode::Char(c) if c.is_ascii_digit() && ('1'..='8').contains(&c) => {
            // Category switching with number keys 1-8
            let category_idx = c.to_digit(10).unwrap() as usize - 1;
            let categories = state.keycode_db.categories();

            if let Some(category) = categories.get(category_idx) {
                state.keycode_picker_state.active_category = Some(category.id.clone());
                state.keycode_picker_state.selected = 0;
                state.set_status(format!("Filtering by: {}", category.name));
            }
            Ok(false)
        }
        KeyCode::Char(c) => {
            // Add to search
            state.keycode_picker_state.search.push(c);
            state.keycode_picker_state.selected = 0; // Reset selection on new search
            Ok(false)
        }
        KeyCode::Backspace => {
            // Remove from search
            state.keycode_picker_state.search.pop();
            state.keycode_picker_state.selected = 0; // Reset selection
            Ok(false)
        }
        KeyCode::Up => {
            // Navigate up in list
            if state.keycode_picker_state.selected > 0 {
                state.keycode_picker_state.selected -= 1;
            }
            Ok(false)
        }
        KeyCode::Down => {
            // Navigate down in list
            let keycodes = if let Some(cat_id) = &state.keycode_picker_state.active_category {
                state
                    .keycode_db
                    .search_in_category(&state.keycode_picker_state.search, cat_id)
            } else {
                state.keycode_db.search(&state.keycode_picker_state.search)
            };

            if state.keycode_picker_state.selected < keycodes.len().saturating_sub(1) {
                state.keycode_picker_state.selected += 1;
            }
            Ok(false)
        }
        _ => Ok(false),
    }
}

/// Helper to create centered rectangle
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
