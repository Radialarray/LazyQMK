//! Keycode picker dialog for selecting keycodes
//!
//! This module implements the `KeycodePicker` component for self-contained keycode selection.

use crossterm::event::{self, KeyCode, KeyModifiers};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
    Frame,
};

use super::component::ContextualComponent;
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
    /// Language selector has focus (when Languages category is selected)
    LanguageSelector,
}

/// Keycode picker state
#[derive(Debug, Clone)]
pub struct KeycodePickerState {
    /// Search query string
    pub search: String,
    /// Selected keycode index in the list
    pub selected: usize,
    /// Current category index (0 = All, last = Languages)
    pub category_index: usize,
    /// Which pane has focus
    pub focus: PickerFocus,
    /// Sidebar scroll offset (for very tall category lists)
    pub sidebar_scroll: usize,
    /// Selected language ID (when in Languages category)
    pub selected_language: Option<String>,
    /// Selected language index in language list
    pub language_list_index: usize,
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
            selected_language: None,
            language_list_index: 0,
        }
    }

    /// Creates a new keycode picker state with an initial language selection
    ///
    /// If `last_language` is Some, automatically sets the category to Languages
    /// and positions to show that language's keycodes.
    #[must_use]
    pub fn with_language(last_language: Option<String>, keycode_db: &KeycodeDb) -> Self {
        match last_language {
            Some(ref lang_id) => {
                // Find the language index in the list
                let languages = keycode_db.languages();
                let language_list_index = languages
                    .iter()
                    .position(|l| l.id == *lang_id)
                    .unwrap_or(0);

                // Languages category is at index: categories.len() + 1
                // (index 0 = "All", then regular categories, then "Languages")
                let languages_category_index = keycode_db.categories().len() + 1;

                Self {
                    search: String::new(),
                    selected: 0,
                    category_index: languages_category_index,
                    focus: PickerFocus::Keycodes,
                    sidebar_scroll: 0,
                    selected_language: Some(lang_id.clone()),
                    language_list_index,
                }
            }
            None => Self::new(),
        }
    }

    /// Reset to initial state
    pub fn reset(&mut self) {
        self.search.clear();
        self.selected = 0;
        self.category_index = 0;
        self.focus = PickerFocus::Keycodes;
        self.sidebar_scroll = 0;
        self.selected_language = None;
        self.language_list_index = 0;
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

    /// Create a new KeycodePicker with an initial language selection
    ///
    /// If `last_language` is Some, automatically sets the category to Languages
    /// and positions to show that language's keycodes.
    #[must_use]
    pub fn with_language(last_language: Option<String>, keycode_db: &KeycodeDb) -> Self {
        Self {
            state: KeycodePickerState::with_language(last_language, keycode_db),
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

    fn handle_input(
        &mut self,
        key: event::KeyEvent,
        context: &Self::Context,
    ) -> Option<Self::Event> {
        let total_categories = context.categories().len() + 2; // +1 for "All", +1 for "Languages"

        match self.state.focus {
            PickerFocus::Sidebar => self.handle_sidebar_input(key, total_categories, context),
            PickerFocus::Keycodes => self.handle_keycodes_input(key, context),
            PickerFocus::LanguageSelector => self.handle_language_selector_input(key, context),
        }
    }

    fn render(&self, f: &mut Frame, _area: Rect, theme: &super::Theme, context: &Self::Context) {
        render_keycode_picker_component(f, self, context, theme);
    }
}

impl KeycodePicker {
    /// Check if the Languages category is selected
    fn is_languages_selected(&self, context: &KeycodeDb) -> bool {
        // Languages is the last category (after all regular categories)
        self.state.category_index == context.categories().len() + 1
    }

    /// Handle input when sidebar has focus
    fn handle_sidebar_input(
        &mut self,
        key: event::KeyEvent,
        total_categories: usize,
        context: &KeycodeDb,
    ) -> Option<KeycodePickerEvent> {
        match key.code {
            KeyCode::Esc => {
                self.state.reset();
                Some(KeycodePickerEvent::Cancelled)
            }
            KeyCode::Up => {
                if self.state.category_index > 0 {
                    self.state.category_index -= 1;
                    self.state.selected = 0; // Reset keycode selection
                    self.state.selected_language = None; // Reset language selection
                }
                None
            }
            KeyCode::Down => {
                if self.state.category_index < total_categories - 1 {
                    self.state.category_index += 1;
                    self.state.selected = 0; // Reset keycode selection
                    self.state.selected_language = None; // Reset language selection
                }
                None
            }
            KeyCode::Home => {
                self.state.category_index = 0;
                self.state.selected = 0;
                self.state.selected_language = None;
                None
            }
            KeyCode::End => {
                self.state.category_index = total_categories - 1;
                self.state.selected = 0;
                self.state.selected_language = None;
                None
            }
            // Switch to keycodes pane or language selector
            KeyCode::Tab | KeyCode::Right | KeyCode::Enter => {
                if self.is_languages_selected(context) && self.state.selected_language.is_none() {
                    // Go to language selector first
                    self.state.focus = PickerFocus::LanguageSelector;
                } else {
                    self.state.focus = PickerFocus::Keycodes;
                }
                None
            }
            // Number keys for quick category jump
            KeyCode::Char(c) if c.is_ascii_digit() => {
                let idx = c.to_digit(10).unwrap() as usize;
                if idx < total_categories {
                    self.state.category_index = idx;
                    self.state.selected = 0;
                    self.state.selected_language = None;
                }
                None
            }
            _ => None,
        }
    }

    /// Handle input when language selector has focus
    fn handle_language_selector_input(
        &mut self,
        key: event::KeyEvent,
        context: &KeycodeDb,
    ) -> Option<KeycodePickerEvent> {
        let total_languages = context.language_count();

        match key.code {
            KeyCode::Esc => {
                // Go back to sidebar
                self.state.focus = PickerFocus::Sidebar;
                self.state.language_list_index = 0;
                None
            }
            KeyCode::Up => {
                if self.state.language_list_index > 0 {
                    self.state.language_list_index -= 1;
                }
                None
            }
            KeyCode::Down => {
                if self.state.language_list_index < total_languages.saturating_sub(1) {
                    self.state.language_list_index += 1;
                }
                None
            }
            KeyCode::Home => {
                self.state.language_list_index = 0;
                None
            }
            KeyCode::End => {
                self.state.language_list_index = total_languages.saturating_sub(1);
                None
            }
            KeyCode::Left | KeyCode::Tab if key.modifiers.contains(KeyModifiers::SHIFT) => {
                self.state.focus = PickerFocus::Sidebar;
                None
            }
            KeyCode::Tab => {
                self.state.focus = PickerFocus::Sidebar;
                None
            }
            KeyCode::Enter | KeyCode::Right => {
                // Select language and go to keycodes
                let languages = context.languages();
                if let Some(lang) = languages.get(self.state.language_list_index) {
                    self.state.selected_language = Some(lang.id.clone());
                    self.state.selected = 0;
                    self.state.focus = PickerFocus::Keycodes;
                }
                None
            }
            _ => None,
        }
    }

    /// Handle input when keycodes list has focus
    fn handle_keycodes_input(
        &mut self,
        key: event::KeyEvent,
        context: &KeycodeDb,
    ) -> Option<KeycodePickerEvent> {
        match key.code {
            KeyCode::Esc => {
                self.state.reset();
                Some(KeycodePickerEvent::Cancelled)
            }
            // Switch back to sidebar or language selector
            KeyCode::Left => {
                if self.is_languages_selected(context) && self.state.selected_language.is_some() {
                    // Go back to language selector
                    self.state.focus = PickerFocus::LanguageSelector;
                } else {
                    self.state.focus = PickerFocus::Sidebar;
                }
                None
            }
            KeyCode::Tab if key.modifiers.contains(KeyModifiers::SHIFT) => {
                if self.is_languages_selected(context) && self.state.selected_language.is_some() {
                    self.state.focus = PickerFocus::LanguageSelector;
                } else {
                    self.state.focus = PickerFocus::Sidebar;
                }
                None
            }
            KeyCode::Tab => {
                // Tab without shift cycles back to sidebar
                self.state.focus = PickerFocus::Sidebar;
                None
            }
            KeyCode::Enter => {
                let keycodes = get_filtered_keycodes_from_context(&self.state, context);
                let selected_keycode_opt =
                    keycodes.get(self.state.selected).map(|kc| kc.code.clone());

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
                self.state.selected =
                    (self.state.selected + 10).min(keycodes.len().saturating_sub(1));
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
pub fn get_filtered_keycodes_from_context<'a>(
    picker_state: &KeycodePickerState,
    context: &'a KeycodeDb,
) -> Vec<&'a crate::keycode_db::KeycodeDefinition> {
    let categories = context.categories();
    let category_index = picker_state.category_index;

    // Check if Languages category is selected
    let languages_index = categories.len() + 1;
    if category_index == languages_index {
        // Return language-specific keycodes if a language is selected
        if let Some(ref lang_id) = picker_state.selected_language {
            return context.search_in_language(&picker_state.search, lang_id);
        }
        // No language selected yet, return empty
        return Vec::new();
    }

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
    render_keycode_picker_internal(f, picker.state(), context, theme);
}

/// Internal shared rendering function for keycode picker
#[allow(clippy::too_many_lines)]
fn render_keycode_picker_internal(
    f: &mut Frame,
    picker_state: &KeycodePickerState,
    context: &KeycodeDb,
    theme: &super::Theme,
) {
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
    let category_index = picker_state.category_index;
    let focus = picker_state.focus;

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

    // Check if Languages category is selected
    let languages_index = categories.len() + 1;
    let is_languages_mode = category_index == languages_index;

    // Search box (hide when in language selector mode without language selected)
    let show_search = !is_languages_mode || picker_state.selected_language.is_some();
    if show_search {
        let search_border_color = if focus == PickerFocus::Keycodes {
            theme.primary
        } else {
            theme.surface
        };

        let search_text = vec![Line::from(vec![
            Span::styled(" Search: ", Style::default().fg(theme.text_muted)),
            Span::styled(
                &picker_state.search,
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
    } else {
        // Show "Select a language" prompt instead of search box
        let prompt = Paragraph::new(Line::from(vec![Span::styled(
            " Select a language to view keycodes",
            Style::default().fg(theme.text_muted),
        )]))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.surface))
                .style(Style::default().bg(theme.background)),
        );
        f.render_widget(prompt, content_chunks[0]);
    }

    // If in Languages mode without a language selected, show language selector
    if is_languages_mode && picker_state.selected_language.is_none() {
        render_language_selector(
            f,
            content_chunks[1],
            context,
            picker_state.language_list_index,
            focus,
            theme,
        );

        // Language selector help text
        let help_spans = vec![
            Span::styled(
                "↑↓",
                Style::default()
                    .fg(theme.primary)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" Navigate  "),
            Span::styled(
                "Enter/→",
                Style::default()
                    .fg(theme.success)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" Select  "),
            Span::styled(
                "←/Tab",
                Style::default()
                    .fg(theme.primary)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" Back  "),
            Span::styled(
                "Esc",
                Style::default()
                    .fg(theme.error)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" Cancel"),
        ];
        let help = Paragraph::new(Line::from(help_spans))
            .style(Style::default().fg(theme.text_muted))
            .block(Block::default().style(Style::default().bg(theme.background)));
        f.render_widget(help, content_chunks[2]);
        return;
    }

    // Get filtered keycodes based on search and category
    let keycodes = if is_languages_mode {
        // Language-specific keycodes
        if let Some(ref lang_id) = picker_state.selected_language {
            context.search_in_language(&picker_state.search, lang_id)
        } else {
            Vec::new()
        }
    } else {
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
    let category_name = if is_languages_mode {
        // Show selected language name
        if let Some(ref lang_id) = picker_state.selected_language {
            context
                .get_language(lang_id)
                .map_or_else(|| "Unknown".to_string(), |l| l.name.clone())
        } else {
            "Languages".to_string()
        }
    } else if category_index == 0 {
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
            picker_state.selected.min(keycodes.len().saturating_sub(1)),
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
        // Check if we're in a language and show back option
        let back_hint = if is_languages_mode && picker_state.selected_language.is_some() {
            "← Back to Languages  "
        } else {
            ""
        };

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
            Span::raw(format!(" {back_hint}")),
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

/// Render the language selector list
fn render_language_selector(
    f: &mut Frame,
    area: Rect,
    context: &KeycodeDb,
    selected_index: usize,
    focus: PickerFocus,
    theme: &super::Theme,
) {
    let languages = context.languages();

    let list_items: Vec<ListItem> = languages
        .iter()
        .map(|lang| {
            let name_style = Style::default().fg(theme.text);
            let prefix_style = Style::default().fg(theme.text_muted);
            let desc_style = Style::default().fg(theme.text_muted);

            let mut spans = vec![
                Span::styled(format!("{:20}", lang.name), name_style),
                Span::styled(format!("[{}] ", lang.prefix), prefix_style),
            ];

            if let Some(ref desc) = lang.description {
                spans.push(Span::styled(desc.as_str(), desc_style));
            }

            ListItem::new(Line::from(spans))
        })
        .collect();

    let border_color = if focus == PickerFocus::LanguageSelector {
        theme.primary
    } else {
        theme.surface
    };

    let list = List::new(list_items)
        .block(
            Block::default()
                .title(format!(" Languages ({}) ", languages.len()))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border_color))
                .style(Style::default().bg(theme.background)),
        )
        .highlight_style(
            Style::default()
                .bg(theme.surface)
                .fg(theme.text)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("► ");

    let mut list_state = ListState::default();
    if focus == PickerFocus::LanguageSelector {
        list_state.select(Some(selected_index.min(languages.len().saturating_sub(1))));
    }

    f.render_stateful_widget(list, area, &mut list_state);
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

    // Build category list items: "All" + all categories + "Languages"
    let mut items: Vec<ListItem> = Vec::with_capacity(categories.len() + 2);

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

    // "Languages" option at the end
    let languages_idx = categories.len() + 1;
    let languages_style = if selected == languages_idx {
        Style::default()
            .fg(theme.background)
            .bg(theme.accent) // Use accent color to distinguish from regular categories
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme.accent)
    };
    items.push(ListItem::new(Line::from(vec![Span::styled(
        " Languages ▸",
        languages_style,
    )])));

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
