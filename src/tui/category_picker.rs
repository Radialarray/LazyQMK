//! Category picker dialog for assigning categories to keys

// Input handlers use Result<bool> for consistency even when they never fail
#![allow(clippy::unnecessary_wraps)]

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
    Frame,
};

/// State for the category picker dialog
#[derive(Debug, Clone)]
pub struct CategoryPickerState {
    /// Index of selected category (`usize::MAX` means "None" option)
    pub selected: usize,
    /// List state for Ratatui list widget
    pub list_state: ListState,
}

impl CategoryPickerState {
    /// Create a new category picker starting at first category
    #[must_use]
    pub fn new() -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0));

        Self {
            selected: 0,
            list_state,
        }
    }

    /// Move selection up
    pub fn previous(&mut self, category_count: usize) {
        if self.selected > 0 {
            self.selected -= 1;
        } else {
            // Wrap to last item (including "None" option)
            self.selected = category_count; // +1 for "None" option, but 0-indexed
        }
        self.list_state.select(Some(self.selected));
    }

    /// Move selection down
    pub fn next(&mut self, category_count: usize) {
        // Total items = categories + 1 for "None" option
        if self.selected < category_count {
            self.selected += 1;
        } else {
            // Wrap to first item
            self.selected = 0;
        }
        self.list_state.select(Some(self.selected));
    }

    /// Get the selected category ID (None if "None" is selected)
    #[must_use]
    pub fn get_selected_category_id(
        &self,
        categories: &[crate::models::Category],
    ) -> Option<String> {
        if self.selected < categories.len() {
            Some(categories[self.selected].id.clone())
        } else {
            // Last item is "None" option
            None
        }
    }
}

impl Default for CategoryPickerState {
    fn default() -> Self {
        Self::new()
    }
}

/// Render the category picker dialog
pub fn render_category_picker(f: &mut Frame, state: &super::AppState) {
    let theme = &state.theme;
    let area = centered_rect(60, 60, f.size());

    // Clear the background area first
    f.render_widget(Clear, area);

    // Render opaque background with theme color
    let background = Block::default()
        .style(Style::default().bg(theme.background));
    f.render_widget(background, area);

    // Build list items with color previews
    let mut items: Vec<ListItem> = state
        .layout
        .categories
        .iter()
        .map(|cat| {
            let color = Color::Rgb(cat.color.r, cat.color.g, cat.color.b);
            let line = Line::from(vec![
                Span::raw("  "),
                Span::styled("███", Style::default().fg(color)),
                Span::raw("  "),
                Span::raw(&cat.name),
                Span::raw(" ("),
                Span::styled(&cat.id, Style::default().fg(theme.text_muted)),
                Span::raw(")"),
            ]);
            ListItem::new(line)
        })
        .collect();

    // Add "None" option at the end
    items.push(ListItem::new(Line::from(vec![
        Span::raw("  "),
        Span::styled(
            "[ None ]",
            Style::default()
                .fg(theme.text_muted)
                .add_modifier(Modifier::ITALIC),
        ),
    ])));

    let list = List::new(items)
        .block(
            Block::default()
                .title(" Select Category ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.primary)),
        )
        .highlight_style(
            Style::default()
                .bg(theme.surface)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("► ");

    // Clone the list_state for rendering
    let mut list_state = state.category_picker_state.list_state.clone();

    // Render the list with state
    f.render_stateful_widget(list, area, &mut list_state);

    // Instructions at the bottom
    let instructions_area = Rect {
        x: area.x + 2,
        y: area.y + area.height - 2,
        width: area.width - 4,
        height: 1,
    };

    let instructions = Paragraph::new(Line::from(vec![
        Span::styled("↑↓", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" Navigate  "),
        Span::styled("Enter", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" Select  "),
        Span::styled("Esc", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" Cancel"),
    ]));
    f.render_widget(instructions, instructions_area);
}

/// Handle input for category picker
pub fn handle_input(state: &mut super::AppState, key: KeyEvent) -> anyhow::Result<bool> {
    match key.code {
        KeyCode::Esc => {
            // Cancel
            state.active_popup = None;
            state.category_picker_context = None;
            state.set_status("Cancelled");
            Ok(false)
        }
        KeyCode::Enter => {
            // Apply category based on context
            let category_id = state
                .category_picker_state
                .get_selected_category_id(&state.layout.categories);

            match state.category_picker_context {
                Some(super::CategoryPickerContext::IndividualKey) => {
                    if let Some(key) = state.get_selected_key_mut() {
                        key.category_id.clone_from(&category_id);
                        state.mark_dirty();

                        if let Some(id) = category_id {
                            state.set_status(format!("Assigned key category '{id}'"));
                        } else {
                            state.set_status("Removed key category");
                        }
                    }
                }
                Some(super::CategoryPickerContext::Layer) => {
                    if let Some(layer) = state.layout.layers.get_mut(state.current_layer) {
                        layer.category_id.clone_from(&category_id);
                        state.mark_dirty();

                        if let Some(id) = category_id {
                            state.set_status(format!("Assigned layer category '{id}'"));
                        } else {
                            state.set_status("Removed layer category");
                        }
                    }
                }
                None => {
                    state.set_error("No category context set");
                }
            }

            state.active_popup = None;
            state.category_picker_context = None;
            Ok(false)
        }
        KeyCode::Up => {
            state
                .category_picker_state
                .previous(state.layout.categories.len());
            Ok(false)
        }
        KeyCode::Down => {
            state
                .category_picker_state
                .next(state.layout.categories.len());
            Ok(false)
        }
        _ => Ok(false),
    }
}

/// Helper to create a centered rectangle
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
