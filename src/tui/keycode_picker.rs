//! Keycode picker dialog for selecting keycodes

use anyhow::Result;
use crossterm::event::{self, KeyCode, KeyModifiers};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
    Frame,
};

use super::layer_picker::LayerKeycodeType;
use super::{AppState, LayerPickerState, PopupType};

/// Keycode picker state
#[derive(Debug, Clone)]
pub struct KeycodePickerState {
    /// Search query string
    pub search: String,
    /// Selected keycode index
    pub selected: usize,
    /// Current category tab index (0 = All)
    pub category_index: usize,
    /// Tab scroll offset for overflow
    pub tab_scroll: usize,
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
            tab_scroll: 0,
        }
    }

    /// Reset to initial state
    pub fn reset(&mut self) {
        self.search.clear();
        self.selected = 0;
        self.category_index = 0;
        self.tab_scroll = 0;
    }

    /// Get the active category ID (None for "All")
    pub fn active_category(&self) -> Option<String> {
        if self.category_index == 0 {
            None
        } else {
            // Will be resolved against actual categories in render/input
            Some(format!("__idx_{}", self.category_index - 1))
        }
    }
}

/// Render the keycode picker popup
pub fn render_keycode_picker(f: &mut Frame, state: &AppState) {
    let theme = &state.theme;
    let area = centered_rect(70, 80, f.size());

    // Clear the background area first
    f.render_widget(Clear, area);

    // Render opaque background with theme color
    let background = Block::default().style(Style::default().bg(theme.background));
    f.render_widget(background, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Category tabs
            Constraint::Length(3), // Search box
            Constraint::Min(10),   // Keycode list
            Constraint::Length(2), // Help text
        ])
        .split(area);

    // Get categories from database
    let categories = state.keycode_db.categories();
    let category_index = state.keycode_picker_state.category_index;

    // Build category tabs
    render_category_tabs(f, chunks[0], categories, category_index, state.keycode_picker_state.tab_scroll, theme);

    // Search box
    let search_text = vec![Line::from(vec![
        Span::styled(" Search: ", Style::default().fg(theme.text_muted)),
        Span::styled(
            &state.keycode_picker_state.search,
            Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
        ),
        Span::styled("_", Style::default().fg(theme.primary).add_modifier(Modifier::SLOW_BLINK)),
    ])];
    let search = Paragraph::new(search_text).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.primary))
            .style(Style::default().bg(theme.background)),
    );
    f.render_widget(search, chunks[1]);

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
            let code_style = Style::default().fg(theme.primary).add_modifier(Modifier::BOLD);
            let name_style = Style::default().fg(theme.text);
            let desc_style = Style::default().fg(theme.text_muted);

            let mut spans = vec![
                Span::styled(format!("{:16}", keycode.code), code_style),
                Span::styled(&keycode.name, name_style),
            ];

            if let Some(desc) = &keycode.description {
                spans.push(Span::styled(format!(" - {}", desc), desc_style));
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
            .map(|c| c.name.clone())
            .unwrap_or_else(|| "Unknown".to_string())
    };

    let list = List::new(list_items)
        .block(
            Block::default()
                .title(format!(" {} ({}) ", category_name, keycodes.len()))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.surface))
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
    list_state.select(Some(
        state
            .keycode_picker_state
            .selected
            .min(keycodes.len().saturating_sub(1)),
    ));

    f.render_stateful_widget(list, chunks[2], &mut list_state);

    // Help text - compact version
    let help_spans = vec![
        Span::styled("◄►", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
        Span::raw(" Category  "),
        Span::styled("↑↓", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
        Span::raw(" Select  "),
        Span::styled("Enter", Style::default().fg(theme.success).add_modifier(Modifier::BOLD)),
        Span::raw(" Apply  "),
        Span::styled("Esc", Style::default().fg(theme.error).add_modifier(Modifier::BOLD)),
        Span::raw(" Cancel  "),
        Span::styled("0", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
        Span::raw(" All"),
    ];
    let help = Paragraph::new(Line::from(help_spans))
        .style(Style::default().fg(theme.text_muted))
        .block(Block::default().style(Style::default().bg(theme.background)));
    f.render_widget(help, chunks[3]);
}

/// Render the category tab bar
fn render_category_tabs(
    f: &mut Frame,
    area: Rect,
    categories: &[crate::keycode_db::KeycodeCategory],
    selected: usize,
    scroll: usize,
    theme: &crate::tui::theme::Theme,
) {
    let inner = Rect {
        x: area.x + 1,
        y: area.y + 1,
        width: area.width.saturating_sub(2),
        height: 1,
    };

    // Build tab labels: "All" + all categories
    let mut tabs: Vec<(&str, bool)> = vec![("All", selected == 0)];
    for (i, cat) in categories.iter().enumerate() {
        tabs.push((&cat.name, selected == i + 1));
    }

    // Calculate which tabs fit
    let available_width = inner.width as usize;
    let mut spans: Vec<Span> = Vec::new();
    let mut current_width = 0;

    // Add scroll indicator if needed
    if scroll > 0 {
        spans.push(Span::styled("◄ ", Style::default().fg(theme.text_muted)));
        current_width += 2;
    }

    let mut _visible_tabs = 0;
    let mut needs_right_scroll = false;

    for (i, (name, is_selected)) in tabs.iter().enumerate().skip(scroll) {
        let tab_width = name.len() + 3; // " Name " + separator

        if current_width + tab_width > available_width.saturating_sub(2) {
            needs_right_scroll = true;
            break;
        }

        let style = if *is_selected {
            Style::default()
                .fg(theme.background)
                .bg(theme.primary)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme.text)
        };

        // Add number hint for first 10 categories
        let label = if i < 10 {
            format!("{} ", i)
        } else {
            String::new()
        };

        spans.push(Span::styled(format!(" {}{} ", label, name), style));
        spans.push(Span::styled("│", Style::default().fg(theme.surface)));

        current_width += tab_width;
        _visible_tabs += 1;
    }

    // Remove last separator
    if !spans.is_empty() && spans.last().map(|s| s.content.as_ref()) == Some("│") {
        spans.pop();
    }

    // Add right scroll indicator if needed
    if needs_right_scroll {
        // Fill remaining space
        let remaining = available_width.saturating_sub(current_width).saturating_sub(2);
        if remaining > 0 {
            spans.push(Span::raw(" ".repeat(remaining)));
        }
        spans.push(Span::styled(" ►", Style::default().fg(theme.text_muted)));
    }

    // Render the border
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.surface))
        .title(" Categories ")
        .title_style(Style::default().fg(theme.text_muted))
        .style(Style::default().bg(theme.background));
    f.render_widget(block, area);

    // Render tabs inside
    let tabs_para = Paragraph::new(Line::from(spans));
    f.render_widget(tabs_para, inner);
}

/// Handle input for keycode picker
pub fn handle_input(state: &mut AppState, key: event::KeyEvent) -> Result<bool> {
    let categories = state.keycode_db.categories();
    let total_tabs = categories.len() + 1; // +1 for "All"

    match key.code {
        KeyCode::Esc => {
            state.active_popup = None;
            state.keycode_picker_state.reset();
            state.set_status("Cancelled");
            Ok(false)
        }
        KeyCode::Enter => {
            // Get filtered keycodes
            let category_index = state.keycode_picker_state.category_index;
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

            let selected_keycode_opt = keycodes
                .get(state.keycode_picker_state.selected)
                .map(|kc| kc.code.clone());

            // Select current keycode
            if let Some(keycode) = selected_keycode_opt {
                // Check if this is a layer-switching keycode (MO, TG, TO, etc.)
                if let Some(layer_type) = LayerKeycodeType::from_keycode(&keycode) {
                    // Switch to layer picker for selecting which layer
                    state.layer_picker_state = LayerPickerState::with_prefix(layer_type.prefix());
                    state.active_popup = Some(PopupType::LayerPicker);
                    state.keycode_picker_state.reset();
                    state.set_status(format!(
                        "Select layer for {} - {}",
                        layer_type.prefix(),
                        layer_type.description()
                    ));
                    return Ok(false);
                }

                // Regular keycode - assign directly
                if let Some(selected_key) = state.get_selected_key_mut() {
                    selected_key.keycode = keycode.clone();
                    state.mark_dirty();
                    state.set_status(format!("Keycode assigned: {keycode}"));
                }

                state.active_popup = None;
                state.keycode_picker_state.reset();
            }

            Ok(false)
        }
        // Category navigation with Left/Right arrows
        KeyCode::Left => {
            if state.keycode_picker_state.category_index > 0 {
                state.keycode_picker_state.category_index -= 1;
                state.keycode_picker_state.selected = 0;
                // Adjust scroll if needed
                if state.keycode_picker_state.category_index < state.keycode_picker_state.tab_scroll {
                    state.keycode_picker_state.tab_scroll = state.keycode_picker_state.category_index;
                }
            }
            Ok(false)
        }
        KeyCode::Right => {
            if state.keycode_picker_state.category_index < total_tabs - 1 {
                state.keycode_picker_state.category_index += 1;
                state.keycode_picker_state.selected = 0;
                // Adjust scroll if needed (rough estimate: scroll when > 5 tabs from start)
                if state.keycode_picker_state.category_index > state.keycode_picker_state.tab_scroll + 5 {
                    state.keycode_picker_state.tab_scroll = state.keycode_picker_state.category_index.saturating_sub(5);
                }
            }
            Ok(false)
        }
        // Tab/Shift+Tab for category cycling
        KeyCode::Tab if key.modifiers.contains(KeyModifiers::SHIFT) => {
            if state.keycode_picker_state.category_index > 0 {
                state.keycode_picker_state.category_index -= 1;
            } else {
                state.keycode_picker_state.category_index = total_tabs - 1;
            }
            state.keycode_picker_state.selected = 0;
            Ok(false)
        }
        KeyCode::Tab => {
            state.keycode_picker_state.category_index = (state.keycode_picker_state.category_index + 1) % total_tabs;
            state.keycode_picker_state.selected = 0;
            Ok(false)
        }
        // Number keys 0-9 for quick category jump
        KeyCode::Char(c) if c.is_ascii_digit() => {
            let idx = c.to_digit(10).unwrap() as usize;
            if idx < total_tabs {
                state.keycode_picker_state.category_index = idx;
                state.keycode_picker_state.selected = 0;
                let category_name = if idx == 0 {
                    "All".to_string()
                } else {
                    categories
                        .get(idx - 1)
                        .map(|c| c.name.clone())
                        .unwrap_or_else(|| "Unknown".to_string())
                };
                state.set_status(format!("Category: {}", category_name));
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
            let category_index = state.keycode_picker_state.category_index;
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

            if state.keycode_picker_state.selected < keycodes.len().saturating_sub(1) {
                state.keycode_picker_state.selected += 1;
            }
            Ok(false)
        }
        KeyCode::Home => {
            state.keycode_picker_state.selected = 0;
            Ok(false)
        }
        KeyCode::End => {
            let category_index = state.keycode_picker_state.category_index;
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

            state.keycode_picker_state.selected = keycodes.len().saturating_sub(1);
            Ok(false)
        }
        KeyCode::PageUp => {
            state.keycode_picker_state.selected = state.keycode_picker_state.selected.saturating_sub(10);
            Ok(false)
        }
        KeyCode::PageDown => {
            let category_index = state.keycode_picker_state.category_index;
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

            state.keycode_picker_state.selected = 
                (state.keycode_picker_state.selected + 10).min(keycodes.len().saturating_sub(1));
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
