//! Keycode picker dialog for selecting keycodes
//!
//! This module implements both:
//! - Legacy rendering function: `render_keycode_picker()` for backward compatibility
//! - Component trait: `KeycodePicker` for self-contained UI components

// Input handlers use Result<bool> for consistency even when they never fail
#![allow(clippy::unnecessary_wraps)]

use anyhow::Result;
use crossterm::event::{self, KeyCode, KeyModifiers};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
    Frame,
};

use super::component::ContextualComponent;
use super::{AppState};
use crate::keycode_db::KeycodeDb;

/// Events emitted by the KeycodePicker component
#[derive(Debug, Clone)]
pub enum KeycodePickerEvent {
    /// User selected a keycode
    KeycodeSelected(String),
    /// User cancelled without making changes
    Cancelled,
}

/// Which pane has focus in the keycode picker
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PickerFocus {
    /// Category sidebar has focus
    Sidebar,
    /// Keycode list has focus (default - allows immediate search)
    #[default]
    Keycodes,
}

/// Keycode picker state
#[derive(Debug, Clone)]
pub struct KeycodePickerState {
    /// Search query string
    pub search: String,
    /// Selected keycode index in the list
    pub selected: usize,
    /// Current category index (0 = All)
    pub category_index: usize,
    /// Which pane has focus
    pub focus: PickerFocus,
    /// Sidebar scroll offset (for very tall category lists)
    pub sidebar_scroll: usize,
}

impl Default for KeycodePickerState {
    fn default() -> Self {
        Self::new()
    }
}

impl KeycodePickerState {
    /// Creates a new keycode picker state
    #[must_use]
    pub const fn new() -> Self {
        Self {
            search: String::new(),
            selected: 0,
            category_index: 0,
            focus: PickerFocus::Keycodes,
            sidebar_scroll: 0,
        }
    }

    /// Reset to initial state
    pub fn reset(&mut self) {
        self.search.clear();
        self.selected = 0;
        self.category_index = 0;
        self.focus = PickerFocus::Keycodes;
        self.sidebar_scroll = 0;
    }
}

/// KeycodePicker component that implements the ContextualComponent trait
#[derive(Debug, Clone)]
pub struct KeycodePicker {
    /// Internal state of the keycode picker
    state: KeycodePickerState,
}

impl KeycodePicker {
    /// Create a new KeycodePicker with default state
    #[must_use]
    pub fn new() -> Self {
        Self {
            state: KeycodePickerState::new(),
        }
    }

    /// Get the current state (for rendering with parent context)
    #[must_use]
    pub const fn state(&self) -> &KeycodePickerState {
        &self.state
    }
}

impl Default for KeycodePicker {
    fn default() -> Self {
        Self::new()
    }
}

impl ContextualComponent for KeycodePicker {
    type Context = KeycodeDb;
    type Event = KeycodePickerEvent;

    fn handle_input(&mut self, key: event::KeyEvent, context: &Self::Context) -> Option<Self::Event> {
        let total_categories = context.categories().len() + 1; // +1 for "All"

        match self.state.focus {
            PickerFocus::Sidebar => self.handle_sidebar_input(key, total_categories),
            PickerFocus::Keycodes => self.handle_keycodes_input(key, context),
        }
    }

    fn render(&self, f: &mut Frame, _area: Rect, theme: &super::Theme, context: &Self::Context) {
        render_keycode_picker_component(f, self, context, theme);
    }
}

impl KeycodePicker {
    /// Handle input when sidebar has focus
    fn handle_sidebar_input(&mut self, key: event::KeyEvent, total_categories: usize) -> Option<KeycodePickerEvent> {
        match key.code {
            KeyCode::Esc => {
                self.state.reset();
                Some(KeycodePickerEvent::Cancelled)
            }
            KeyCode::Up => {
                if self.state.category_index > 0 {
                    self.state.category_index -= 1;
                    self.state.selected = 0; // Reset keycode selection
                }
                None
            }
            KeyCode::Down => {
                if self.state.category_index < total_categories - 1 {
                    self.state.category_index += 1;
                    self.state.selected = 0; // Reset keycode selection
                }
                None
            }
            KeyCode::Home => {
                self.state.category_index = 0;
                self.state.selected = 0;
                None
            }
            KeyCode::End => {
                self.state.category_index = total_categories - 1;
                self.state.selected = 0;
                None
            }
            // Switch to keycodes pane
            KeyCode::Tab | KeyCode::Right | KeyCode::Enter => {
                self.state.focus = PickerFocus::Keycodes;
                None
            }
            // Number keys for quick category jump
            KeyCode::Char(c) if c.is_ascii_digit() => {
                let idx = c.to_digit(10).unwrap() as usize;
                if idx < total_categories {
                    self.state.category_index = idx;
                    self.state.selected = 0;
                }
                None
            }
            _ => None,
        }
    }

    /// Handle input when keycodes list has focus
    fn handle_keycodes_input(&mut self, key: event::KeyEvent, context: &KeycodeDb) -> Option<KeycodePickerEvent> {
        match key.code {
            KeyCode::Esc => {
                self.state.reset();
                Some(KeycodePickerEvent::Cancelled)
            }
            // Switch back to sidebar (arrow keys always work)
            KeyCode::Left => {
                self.state.focus = PickerFocus::Sidebar;
                None
            }
            KeyCode::Tab if key.modifiers.contains(KeyModifiers::SHIFT) => {
                self.state.focus = PickerFocus::Sidebar;
                None
            }
            KeyCode::Tab => {
                // Tab without shift cycles back to sidebar
                self.state.focus = PickerFocus::Sidebar;
                None
            }
            KeyCode::Enter => {
                let keycodes = get_filtered_keycodes_from_context(&self.state, context);
                let selected_keycode_opt = keycodes
                    .get(self.state.selected)
                    .map(|kc| kc.code.clone());

                if let Some(keycode) = selected_keycode_opt {
                    self.state.reset();
                    Some(KeycodePickerEvent::KeycodeSelected(keycode))
                } else {
                    None
                }
            }
            // Arrow keys always navigate
            KeyCode::Up => {
                if self.state.selected > 0 {
                    self.state.selected -= 1;
                }
                None
            }
            KeyCode::Down => {
                let keycodes = get_filtered_keycodes_from_context(&self.state, context);
                if self.state.selected < keycodes.len().saturating_sub(1) {
                    self.state.selected += 1;
                }
                None
            }
            KeyCode::Home => {
                self.state.selected = 0;
                None
            }
            KeyCode::End => {
                let keycodes = get_filtered_keycodes_from_context(&self.state, context);
                self.state.selected = keycodes.len().saturating_sub(1);
                None
            }
            KeyCode::PageUp => {
                self.state.selected = self.state.selected.saturating_sub(10);
                None
            }
            KeyCode::PageDown => {
                let keycodes = get_filtered_keycodes_from_context(&self.state, context);
                self.state.selected = (self.state.selected + 10).min(keycodes.len().saturating_sub(1));
                None
            }
            KeyCode::Char(c) => {
                // Add to search (includes j, k, h, l when search is active)
                self.state.search.push(c);
                self.state.selected = 0; // Reset selection on new search
                None
            }
            KeyCode::Backspace => {
                // Remove from search
                self.state.search.pop();
                self.state.selected = 0; // Reset selection
                None
            }
            _ => None,
        }
    }
}

/// Get filtered keycodes based on state and context (helper for component)
fn get_filtered_keycodes_from_context<'a>(
    picker_state: &KeycodePickerState,
    context: &'a KeycodeDb,
) -> Vec<&'a crate::keycode_db::KeycodeDefinition> {
    let categories = context.categories();
    let category_index = picker_state.category_index;

    let active_category = if category_index == 0 {
        None
    } else {
        categories.get(category_index - 1).map(|c| c.id.as_str())
    };

    if let Some(cat_id) = active_category {
        context.search_in_category(&picker_state.search, cat_id)
    } else {
        context.search(&picker_state.search)
    }
}

/// Render the keycode picker popup using the Component
fn render_keycode_picker_component(
    f: &mut Frame,
    picker: &KeycodePicker,
    context: &KeycodeDb,
    theme: &super::Theme,
) {
    let state = picker.state();
    let area = centered_rect(80, 85, f.size());

    // Clear the background area first
    f.render_widget(Clear, area);

    // Render opaque background with theme color
    let background = Block::default().style(Style::default().bg(theme.background));
    f.render_widget(background, area);

    // Main horizontal split: sidebar (20%) | content (80%)
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(22), // Fixed width sidebar for category names
            Constraint::Min(40),    // Keycode list takes remaining space
        ])
        .split(area);

    let sidebar_area = main_chunks[0];
    let content_area = main_chunks[1];

    // Get categories from database
    let categories = context.categories();
    let category_index = state.category_index;
    let focus = state.focus;

    // Render sidebar with categories
    render_sidebar(f, sidebar_area, categories, category_index, focus, theme);

    // Content area: search box + keycode list + help
    let content_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Search box
            Constraint::Min(10),   // Keycode list
            Constraint::Length(2), // Help text
        ])
        .split(content_area);

    // Search box
    let search_border_color = if focus == PickerFocus::Keycodes {
        theme.primary
    } else {
        theme.surface
    };

    let search_text = vec![Line::from(vec![
        Span::styled(" Search: ", Style::default().fg(theme.text_muted)),
        Span::styled(
            &state.search,
            Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "_",
            Style::default()
                .fg(theme.primary)
                .add_modifier(Modifier::SLOW_BLINK),
        ),
    ])];
    let search = Paragraph::new(search_text).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(search_border_color))
            .style(Style::default().bg(theme.background)),
    );
    f.render_widget(search, content_chunks[0]);

    // Get filtered keycodes based on search and category
    let active_category = if category_index == 0 {
        None
    } else {
        categories.get(category_index - 1).map(|c| c.id.as_str())
    };

    let keycodes = if let Some(cat_id) = active_category {
        context.search_in_category(&state.search, cat_id)
    } else {
        context.search(&state.search)
    };

    // Build list items with better formatting
    let list_items: Vec<ListItem> = keycodes
        .iter()
        .map(|keycode| {
            let code_style = Style::default()
                .fg(theme.primary)
                .add_modifier(Modifier::BOLD);
            let name_style = Style::default().fg(theme.text);
            let desc_style = Style::default().fg(theme.text_muted);

            let mut spans = vec![
                Span::styled(format!("{:16}", keycode.code), code_style),
                Span::styled(&keycode.name, name_style),
            ];

            if let Some(desc) = &keycode.description {
                spans.push(Span::styled(format!(" - {desc}"), desc_style));
            }

            ListItem::new(Line::from(spans))
        })
        .collect();

    // Create list widget with stateful selection
    let category_name = if category_index == 0 {
        "All".to_string()
    } else {
        categories
            .get(category_index - 1)
            .map_or_else(|| "Unknown".to_string(), |c| c.name.clone())
    };

    let list_border_color = if focus == PickerFocus::Keycodes {
        theme.primary
    } else {
        theme.surface
    };

    let list = List::new(list_items)
        .block(
            Block::default()
                .title(format!(" {} ({}) ", category_name, keycodes.len()))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(list_border_color))
                .style(Style::default().bg(theme.background)),
        )
        .highlight_style(
            Style::default()
                .bg(theme.surface)
                .fg(theme.text)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("► ");

    // Create list state for highlighting
    let mut list_state = ListState::default();
    if focus == PickerFocus::Keycodes {
        list_state.select(Some(
            state.selected.min(keycodes.len().saturating_sub(1)),
        ));
    }

    f.render_stateful_widget(list, content_chunks[1], &mut list_state);

    // Help text
    let help_spans = if focus == PickerFocus::Sidebar {
        vec![
            Span::styled(
                "↑↓",
                Style::default()
                    .fg(theme.primary)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" Category  "),
            Span::styled(
                "Tab/→",
                Style::default()
                    .fg(theme.primary)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" Keycodes  "),
            Span::styled(
                "Enter",
                Style::default()
                    .fg(theme.success)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" Select  "),
            Span::styled(
                "Esc",
                Style::default()
                    .fg(theme.error)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" Cancel"),
        ]
    } else {
        vec![
            Span::styled(
                "↑↓",
                Style::default()
                    .fg(theme.primary)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" Navigate  "),
            Span::styled(
                "Tab/←",
                Style::default()
                    .fg(theme.primary)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" Categories  "),
            Span::styled(
                "Enter",
                Style::default()
                    .fg(theme.success)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" Apply  "),
            Span::styled(
                "Esc",
                Style::default()
                    .fg(theme.error)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" Cancel  "),
            Span::styled(
                "Type",
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" Search"),
        ]
    };
    let help = Paragraph::new(Line::from(help_spans))
        .style(Style::default().fg(theme.text_muted))
        .block(Block::default().style(Style::default().bg(theme.background)));
    f.render_widget(help, content_chunks[2]);
}

/// Render the keycode picker popup with sidebar layout (legacy - for backward compatibility)
#[allow(clippy::too_many_lines)]
pub fn render_keycode_picker(f: &mut Frame, state: &AppState) {
    let theme = &state.theme;
    let area = centered_rect(80, 85, f.size());

    // Clear the background area first
    f.render_widget(Clear, area);

    // Render opaque background with theme color
    let background = Block::default().style(Style::default().bg(theme.background));
    f.render_widget(background, area);

    // Main horizontal split: sidebar (20%) | content (80%)
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(22), // Fixed width sidebar for category names
            Constraint::Min(40),    // Keycode list takes remaining space
        ])
        .split(area);

    let sidebar_area = main_chunks[0];
    let content_area = main_chunks[1];

    // Get categories from database
    let categories = state.keycode_db.categories();
    let category_index = state.keycode_picker_state.category_index;
    let focus = state.keycode_picker_state.focus;

    // Render sidebar with categories
    render_sidebar(f, sidebar_area, categories, category_index, focus, theme);

    // Content area: search box + keycode list + help
    let content_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Search box
            Constraint::Min(10),   // Keycode list
            Constraint::Length(2), // Help text
        ])
        .split(content_area);

    // Search box
    let search_border_color = if focus == PickerFocus::Keycodes {
        theme.primary
    } else {
        theme.surface
    };

    let search_text = vec![Line::from(vec![
        Span::styled(" Search: ", Style::default().fg(theme.text_muted)),
        Span::styled(
            &state.keycode_picker_state.search,
            Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "_",
            Style::default()
                .fg(theme.primary)
                .add_modifier(Modifier::SLOW_BLINK),
        ),
    ])];
    let search = Paragraph::new(search_text).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(search_border_color))
            .style(Style::default().bg(theme.background)),
    );
    f.render_widget(search, content_chunks[0]);

    // Get filtered keycodes based on search and category
    let active_category = if category_index == 0 {
        None
    } else {
        categories.get(category_index - 1).map(|c| c.id.as_str())
    };

    let keycodes = if let Some(cat_id) = active_category {
        state
            .keycode_db
            .search_in_category(&state.keycode_picker_state.search, cat_id)
    } else {
        state.keycode_db.search(&state.keycode_picker_state.search)
    };

    // Build list items with better formatting
    let list_items: Vec<ListItem> = keycodes
        .iter()
        .map(|keycode| {
            let code_style = Style::default()
                .fg(theme.primary)
                .add_modifier(Modifier::BOLD);
            let name_style = Style::default().fg(theme.text);
            let desc_style = Style::default().fg(theme.text_muted);

            let mut spans = vec![
                Span::styled(format!("{:16}", keycode.code), code_style),
                Span::styled(&keycode.name, name_style),
            ];

            if let Some(desc) = &keycode.description {
                spans.push(Span::styled(format!(" - {desc}"), desc_style));
            }

            ListItem::new(Line::from(spans))
        })
        .collect();

    // Create list widget with stateful selection
    let category_name = if category_index == 0 {
        "All".to_string()
    } else {
        categories
            .get(category_index - 1)
            .map_or_else(|| "Unknown".to_string(), |c| c.name.clone())
    };

    let list_border_color = if focus == PickerFocus::Keycodes {
        theme.primary
    } else {
        theme.surface
    };

    let list = List::new(list_items)
        .block(
            Block::default()
                .title(format!(" {} ({}) ", category_name, keycodes.len()))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(list_border_color))
                .style(Style::default().bg(theme.background)),
        )
        .highlight_style(
            Style::default()
                .bg(theme.surface)
                .fg(theme.text)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("► ");

    // Create list state for highlighting
    let mut list_state = ListState::default();
    if focus == PickerFocus::Keycodes {
        list_state.select(Some(
            state
                .keycode_picker_state
                .selected
                .min(keycodes.len().saturating_sub(1)),
        ));
    }

    f.render_stateful_widget(list, content_chunks[1], &mut list_state);

    // Help text
    let help_spans = if focus == PickerFocus::Sidebar {
        vec![
            Span::styled(
                "↑↓",
                Style::default()
                    .fg(theme.primary)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" Category  "),
            Span::styled(
                "Tab/→",
                Style::default()
                    .fg(theme.primary)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" Keycodes  "),
            Span::styled(
                "Enter",
                Style::default()
                    .fg(theme.success)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" Select  "),
            Span::styled(
                "Esc",
                Style::default()
                    .fg(theme.error)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" Cancel"),
        ]
    } else {
        vec![
            Span::styled(
                "↑↓",
                Style::default()
                    .fg(theme.primary)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" Navigate  "),
            Span::styled(
                "Tab/←",
                Style::default()
                    .fg(theme.primary)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" Categories  "),
            Span::styled(
                "Enter",
                Style::default()
                    .fg(theme.success)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" Apply  "),
            Span::styled(
                "Esc",
                Style::default()
                    .fg(theme.error)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" Cancel  "),
            Span::styled(
                "Type",
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" Search"),
        ]
    };
    let help = Paragraph::new(Line::from(help_spans))
        .style(Style::default().fg(theme.text_muted))
        .block(Block::default().style(Style::default().bg(theme.background)));
    f.render_widget(help, content_chunks[2]);
}

/// Render the category sidebar
fn render_sidebar(
    f: &mut Frame,
    area: Rect,
    categories: &[crate::keycode_db::KeycodeCategory],
    selected: usize,
    focus: PickerFocus,
    theme: &crate::tui::theme::Theme,
) {
    let border_color = if focus == PickerFocus::Sidebar {
        theme.primary
    } else {
        theme.surface
    };

    // Build category list items: "All" + all categories
    let mut items: Vec<ListItem> = Vec::with_capacity(categories.len() + 1);

    // "All" option
    let all_style = if selected == 0 {
        Style::default()
            .fg(theme.background)
            .bg(theme.primary)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme.text)
    };
    items.push(ListItem::new(Line::from(vec![Span::styled(
        " All", all_style,
    )])));

    // Category items
    for (i, cat) in categories.iter().enumerate() {
        let style = if selected == i + 1 {
            Style::default()
                .fg(theme.background)
                .bg(theme.primary)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme.text)
        };

        // Truncate long names to fit sidebar
        let name = if cat.name.len() > 18 {
            format!(" {}…", &cat.name[..17])
        } else {
            format!(" {}", cat.name)
        };

        items.push(ListItem::new(Line::from(vec![Span::styled(name, style)])));
    }

    let list = List::new(items)
        .block(
            Block::default()
                .title(" Categories ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border_color))
                .style(Style::default().bg(theme.background)),
        )
        .highlight_symbol(if focus == PickerFocus::Sidebar {
            "►"
        } else {
            " "
        });

    // Create list state for sidebar
    let mut list_state = ListState::default();
    if focus == PickerFocus::Sidebar {
        list_state.select(Some(selected));
    }

    f.render_stateful_widget(list, area, &mut list_state);
}

/// Get filtered keycodes based on current search and category
#[must_use]
pub fn get_filtered_keycodes(state: &AppState) -> Vec<&crate::keycode_db::KeycodeDefinition> {
    let categories = state.keycode_db.categories();
    let category_index = state.keycode_picker_state.category_index;

    let active_category = if category_index == 0 {
        None
    } else {
        categories.get(category_index - 1).map(|c| c.id.as_str())
    };

    if let Some(cat_id) = active_category {
        state
            .keycode_db
            .search_in_category(&state.keycode_picker_state.search, cat_id)
    } else {
        state.keycode_db.search(&state.keycode_picker_state.search)
    }
}

/// Handle navigation-only input for keycode picker (used by `TapKeycodePicker`)
/// This handles all input except Enter (which is handled by caller)
#[allow(clippy::too_many_lines)]
pub fn handle_navigation(state: &mut AppState, key: event::KeyEvent) -> Result<bool> {
    let total_categories = state.keycode_db.categories().len() + 1;
    // Check if search is active (has content) - vim keys should type instead of navigate
    let _search_active = !state.keycode_picker_state.search.is_empty();

    match state.keycode_picker_state.focus {
        PickerFocus::Sidebar => match key.code {
            KeyCode::Up => {
                if state.keycode_picker_state.category_index > 0 {
                    state.keycode_picker_state.category_index -= 1;
                    state.keycode_picker_state.selected = 0;
                }
                Ok(false)
            }
            KeyCode::Down => {
                if state.keycode_picker_state.category_index < total_categories - 1 {
                    state.keycode_picker_state.category_index += 1;
                    state.keycode_picker_state.selected = 0;
                }
                Ok(false)
            }
            KeyCode::Home => {
                state.keycode_picker_state.category_index = 0;
                state.keycode_picker_state.selected = 0;
                Ok(false)
            }
            KeyCode::End => {
                state.keycode_picker_state.category_index = total_categories - 1;
                state.keycode_picker_state.selected = 0;
                Ok(false)
            }
            KeyCode::Tab | KeyCode::Right => {
                state.keycode_picker_state.focus = PickerFocus::Keycodes;
                Ok(false)
            }
            KeyCode::Char(c) if c.is_ascii_digit() => {
                let idx = c.to_digit(10).unwrap() as usize;
                if idx < total_categories {
                    state.keycode_picker_state.category_index = idx;
                    state.keycode_picker_state.selected = 0;
                }
                Ok(false)
            }
            _ => Ok(false),
        },
        PickerFocus::Keycodes => match key.code {
            KeyCode::Left => {
                state.keycode_picker_state.focus = PickerFocus::Sidebar;
                Ok(false)
            }
            KeyCode::Tab if key.modifiers.contains(KeyModifiers::SHIFT) => {
                state.keycode_picker_state.focus = PickerFocus::Sidebar;
                Ok(false)
            }
            KeyCode::Tab => {
                state.keycode_picker_state.focus = PickerFocus::Sidebar;
                Ok(false)
            }
            KeyCode::Up => {
                if state.keycode_picker_state.selected > 0 {
                    state.keycode_picker_state.selected -= 1;
                }
                Ok(false)
            }
            KeyCode::Down => {
                let keycodes = get_filtered_keycodes(state);
                if state.keycode_picker_state.selected < keycodes.len().saturating_sub(1) {
                    state.keycode_picker_state.selected += 1;
                }
                Ok(false)
            }
            // NOTE: Vim navigation (h/j/k/l) removed from keycodes pane to allow typing these
            // letters in search. Arrow keys, PageUp/Down, Home/End still work for navigation.
            KeyCode::Home => {
                state.keycode_picker_state.selected = 0;
                Ok(false)
            }
            KeyCode::End => {
                let keycodes = get_filtered_keycodes(state);
                state.keycode_picker_state.selected = keycodes.len().saturating_sub(1);
                Ok(false)
            }
            KeyCode::PageUp => {
                state.keycode_picker_state.selected =
                    state.keycode_picker_state.selected.saturating_sub(10);
                Ok(false)
            }
            KeyCode::PageDown => {
                let keycodes = get_filtered_keycodes(state);
                state.keycode_picker_state.selected = (state.keycode_picker_state.selected + 10)
                    .min(keycodes.len().saturating_sub(1));
                Ok(false)
            }
            KeyCode::Char(c) => {
                // Add to search (includes j, k, h, l when search is active)
                state.keycode_picker_state.search.push(c);
                state.keycode_picker_state.selected = 0;
                Ok(false)
            }
            KeyCode::Backspace => {
                state.keycode_picker_state.search.pop();
                state.keycode_picker_state.selected = 0;
                Ok(false)
            }
            _ => Ok(false),
        },
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
