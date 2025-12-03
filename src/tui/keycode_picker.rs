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

use crate::keycode_db::{KeycodeDb, ParamType};
use super::{AppState, LayerPickerState, ParameterizedKeycodeType, PendingKeycodeState, PopupType};
use super::key_editor::ComboEditPart;

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

/// Render the keycode picker popup with sidebar layout
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
        Span::styled("_", Style::default().fg(theme.primary).add_modifier(Modifier::SLOW_BLINK)),
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
            Span::styled("↑↓", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
            Span::raw(" Category  "),
            Span::styled("Tab/→", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
            Span::raw(" Keycodes  "),
            Span::styled("Enter", Style::default().fg(theme.success).add_modifier(Modifier::BOLD)),
            Span::raw(" Select  "),
            Span::styled("Esc", Style::default().fg(theme.error).add_modifier(Modifier::BOLD)),
            Span::raw(" Cancel"),
        ]
    } else {
        vec![
            Span::styled("↑↓", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
            Span::raw(" Navigate  "),
            Span::styled("Tab/←", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
            Span::raw(" Categories  "),
            Span::styled("Enter", Style::default().fg(theme.success).add_modifier(Modifier::BOLD)),
            Span::raw(" Apply  "),
            Span::styled("Esc", Style::default().fg(theme.error).add_modifier(Modifier::BOLD)),
            Span::raw(" Cancel  "),
            Span::styled("Type", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
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
    items.push(ListItem::new(Line::from(vec![
        Span::styled(" All", all_style),
    ])));

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
        
        items.push(ListItem::new(Line::from(vec![
            Span::styled(name, style),
        ])));
    }

    let list = List::new(items)
        .block(
            Block::default()
                .title(" Categories ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border_color))
                .style(Style::default().bg(theme.background)),
        )
        .highlight_symbol(if focus == PickerFocus::Sidebar { "►" } else { " " });

    // Create list state for sidebar
    let mut list_state = ListState::default();
    if focus == PickerFocus::Sidebar {
        list_state.select(Some(selected));
    }

    f.render_stateful_widget(list, area, &mut list_state);
}

/// Handle input for keycode picker
pub fn handle_input(state: &mut AppState, key: event::KeyEvent) -> Result<bool> {
    let total_categories = state.keycode_db.categories().len() + 1; // +1 for "All"

    match state.keycode_picker_state.focus {
        PickerFocus::Sidebar => handle_sidebar_input(state, key, total_categories),
        PickerFocus::Keycodes => handle_keycodes_input(state, key),
    }
}

/// Handle input when sidebar has focus
fn handle_sidebar_input(
    state: &mut AppState,
    key: event::KeyEvent,
    total_categories: usize,
) -> Result<bool> {
    match key.code {
        KeyCode::Esc => {
            state.active_popup = None;
            state.keycode_picker_state.reset();
            state.set_status("Cancelled");
            Ok(false)
        }
        KeyCode::Up => {
            if state.keycode_picker_state.category_index > 0 {
                state.keycode_picker_state.category_index -= 1;
                state.keycode_picker_state.selected = 0; // Reset keycode selection
            }
            Ok(false)
        }
        KeyCode::Down => {
            if state.keycode_picker_state.category_index < total_categories - 1 {
                state.keycode_picker_state.category_index += 1;
                state.keycode_picker_state.selected = 0; // Reset keycode selection
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
        // Switch to keycodes pane
        KeyCode::Tab | KeyCode::Right | KeyCode::Enter => {
            state.keycode_picker_state.focus = PickerFocus::Keycodes;
            Ok(false)
        }
        // Number keys for quick category jump
        KeyCode::Char(c) if c.is_ascii_digit() => {
            let idx = c.to_digit(10).unwrap() as usize;
            if idx < total_categories {
                state.keycode_picker_state.category_index = idx;
                state.keycode_picker_state.selected = 0;
            }
            Ok(false)
        }
        _ => Ok(false),
    }
}

/// Handle input when keycodes list has focus
fn handle_keycodes_input(
    state: &mut AppState,
    key: event::KeyEvent,
) -> Result<bool> {
    match key.code {
        KeyCode::Esc => {
            // If we were editing a combo part, go back to key editor
            if state.key_editor_state.combo_edit.is_some() {
                state.key_editor_state.combo_edit = None;
                state.active_popup = Some(PopupType::KeyEditor);
                state.keycode_picker_state.reset();
                state.set_status("Cancelled - back to key editor");
            } else {
                state.active_popup = None;
                state.keycode_picker_state.reset();
                state.set_status("Cancelled");
            }
            Ok(false)
        }
        // Switch back to sidebar
        KeyCode::Left => {
            state.keycode_picker_state.focus = PickerFocus::Sidebar;
            Ok(false)
        }
        KeyCode::Tab if key.modifiers.contains(KeyModifiers::SHIFT) => {
            state.keycode_picker_state.focus = PickerFocus::Sidebar;
            Ok(false)
        }
        KeyCode::Tab => {
            // Tab without shift cycles back to sidebar
            state.keycode_picker_state.focus = PickerFocus::Sidebar;
            Ok(false)
        }
        KeyCode::Enter => {
            let keycodes = get_filtered_keycodes(state);
            let selected_keycode_opt = keycodes
                .get(state.keycode_picker_state.selected)
                .map(|kc| kc.code.clone());

            if let Some(keycode) = selected_keycode_opt {
                // Check if we're editing a combo keycode part
                if let Some((part, combo_type)) = state.key_editor_state.combo_edit.take() {
                    // Updating part of a combo keycode
                    let new_combo = match part {
                        ComboEditPart::Hold => combo_type.with_hold(&keycode),
                        ComboEditPart::Tap => combo_type.with_tap(&keycode),
                    };
                    let new_keycode = new_combo.to_keycode();
                    
                    if let Some(selected_key) = state.get_selected_key_mut() {
                        selected_key.keycode = new_keycode.clone();
                        state.mark_dirty();
                        state.set_status(format!("Updated: {}", new_keycode));
                    }
                    
                    state.active_popup = Some(PopupType::KeyEditor);
                    state.keycode_picker_state.reset();
                    return Ok(false);
                }
                
                // Check if this keycode has parameters defined in the database
                // Clone the params to avoid borrow conflicts
                let params_opt = state.keycode_db.get_params(&keycode).map(|p| p.to_vec());
                if let Some(params) = params_opt {
                    if let Some(prefix) = KeycodeDb::get_prefix(&keycode) {
                        return handle_parameterized_keycode(state, prefix, &params);
                    }
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
            state.keycode_picker_state.selected = 
                (state.keycode_picker_state.selected + 10).min(keycodes.len().saturating_sub(1));
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
        _ => Ok(false),
    }
}

/// Get filtered keycodes based on current search and category
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

/// Handle navigation-only input for keycode picker (used by TapKeycodePicker)
/// This handles all input except Enter (which is handled by caller)
pub fn handle_navigation(state: &mut AppState, key: event::KeyEvent) -> Result<bool> {
    let total_categories = state.keycode_db.categories().len() + 1;
    
    match state.keycode_picker_state.focus {
        PickerFocus::Sidebar => {
            match key.code {
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
            }
        }
        PickerFocus::Keycodes => {
            match key.code {
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
                    state.keycode_picker_state.selected = 
                        (state.keycode_picker_state.selected + 10).min(keycodes.len().saturating_sub(1));
                    Ok(false)
                }
                KeyCode::Char(c) => {
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
            }
        }
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

/// Handle a parameterized keycode selection based on its params from the database.
/// This replaces the old hardcoded match statements with a data-driven approach.
fn handle_parameterized_keycode(
    state: &mut AppState,
    prefix: &str,
    params: &[crate::keycode_db::KeycodeParam],
) -> Result<bool> {
    state.pending_keycode = PendingKeycodeState::new();
    state.keycode_picker_state.reset();

    // Determine the keycode type based on the parameter pattern
    let keycode_type = determine_keycode_type(prefix, params);
    state.pending_keycode.keycode_type = Some(keycode_type.clone());

    // For keycodes with a fixed prefix (like LCTL_T, LCG), store it
    if matches!(
        keycode_type,
        ParameterizedKeycodeType::SimpleModTap | ParameterizedKeycodeType::SwapHandsTap
    ) {
        state.pending_keycode.param1 = Some(prefix.to_string());
    }

    // Set up the first picker based on the first parameter type
    let first_param = &params[0];
    match first_param.param_type {
        ParamType::Layer => {
            state.layer_picker_state = LayerPickerState::with_prefix(prefix);
            state.active_popup = Some(PopupType::LayerPicker);
            state.set_status(format!(
                "Select {} for {}",
                first_param.description.as_deref().unwrap_or("layer"),
                prefix
            ));
        }
        ParamType::Modifier => {
            state.active_popup = Some(PopupType::ModifierPicker);
            state.set_status(format!(
                "Select {} for {}",
                first_param.description.as_deref().unwrap_or("modifier"),
                prefix
            ));
        }
        ParamType::Keycode => {
            state.active_popup = Some(PopupType::TapKeycodePicker);
            state.keycode_picker_state = KeycodePickerState::new();
            state.set_status(format!(
                "Select {} for {}",
                first_param.description.as_deref().unwrap_or("keycode"),
                prefix
            ));
        }
    }

    Ok(false)
}

/// Determine the ParameterizedKeycodeType based on prefix and params pattern
fn determine_keycode_type(
    prefix: &str,
    params: &[crate::keycode_db::KeycodeParam],
) -> ParameterizedKeycodeType {
    match (params.len(), params.first().map(|p| &p.param_type)) {
        // Two params: Layer + Keycode = LayerTap
        (2, Some(ParamType::Layer)) if params[1].param_type == ParamType::Keycode => {
            ParameterizedKeycodeType::LayerTap
        }
        // Two params: Layer + Modifier = LayerMod
        (2, Some(ParamType::Layer)) if params[1].param_type == ParamType::Modifier => {
            ParameterizedKeycodeType::LayerMod
        }
        // Two params: Modifier + Keycode = ModTap (custom, like MT())
        (2, Some(ParamType::Modifier)) if params[1].param_type == ParamType::Keycode => {
            ParameterizedKeycodeType::ModTap
        }
        // One param: Keycode with special prefix
        (1, Some(ParamType::Keycode)) if prefix == "SH_T" => {
            ParameterizedKeycodeType::SwapHandsTap
        }
        // One param: Keycode (mod-tap or modifier wrapper)
        (1, Some(ParamType::Keycode)) => ParameterizedKeycodeType::SimpleModTap,
        // One param: Layer only (MO, TG, TO, etc.) - handled separately via layer_picker
        // One param: Modifier only (OSM) - for now use ModTap flow
        (1, Some(ParamType::Modifier)) => ParameterizedKeycodeType::ModTap,
        // Single layer param - shouldn't happen here as it's handled by layer_picker
        (1, Some(ParamType::Layer)) => ParameterizedKeycodeType::LayerTap,
        // Fallback
        _ => ParameterizedKeycodeType::SimpleModTap,
    }
}
