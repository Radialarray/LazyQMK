//! Category manager for CRUD operations on categories.
//!
//! Provides a UI for creating, renaming, recoloring, and deleting categories.
//! Accessible via Ctrl+T shortcut.

use ratatui::{
    layout::{Alignment, Constraint, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};

use crate::models::Category;

/// Manager mode - determines what operation is being performed
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ManagerMode {
    /// Browsing categories (default mode)
    Browsing,
    /// Creating a new category (entering name)
    CreatingName {
        /// User input for category name
        input: String,
    },
    /// Creating a new category (selecting color)
    CreatingColor {
        /// Name of category being created
        name: String,
    },
    /// Renaming a category
    Renaming {
        /// ID of category being renamed
        category_id: String,
        /// User input for new name
        input: String,
    },
    /// Confirming deletion
    ConfirmingDelete {
        /// ID of category to delete
        category_id: String,
    },
}

/// State for the category manager dialog
#[derive(Debug, Clone)]
pub struct CategoryManagerState {
    /// List of categories (reference to layout.categories)
    /// We don't store the categories directly, just track selection
    pub selected: usize,
    /// Current operation mode
    pub mode: ManagerMode,
}

impl CategoryManagerState {
    /// Create a new category manager state
    #[must_use]
    pub const fn new() -> Self {
        Self {
            selected: 0,
            mode: ManagerMode::Browsing,
        }
    }

    /// Reset to default state
    pub fn reset(&mut self) {
        self.selected = 0;
        self.mode = ManagerMode::Browsing;
    }

    /// Move selection up
    pub const fn select_previous(&mut self, category_count: usize) {
        if category_count > 0 {
            if self.selected > 0 {
                self.selected -= 1;
            } else {
                self.selected = category_count - 1;
            }
        }
    }

    /// Move selection down
    pub const fn select_next(&mut self, category_count: usize) {
        if category_count > 0 {
            self.selected = (self.selected + 1) % category_count;
        }
    }

    /// Start creating a new category
    pub fn start_creating(&mut self) {
        self.mode = ManagerMode::CreatingName {
            input: String::new(),
        };
    }

    /// Start renaming the selected category
    pub fn start_renaming(&mut self, category: &Category) {
        self.mode = ManagerMode::Renaming {
            category_id: category.id.clone(),
            input: category.name.clone(),
        };
    }

    /// Start confirming deletion of the selected category
    pub fn start_deleting(&mut self, category: &Category) {
        self.mode = ManagerMode::ConfirmingDelete {
            category_id: category.id.clone(),
        };
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
            ManagerMode::CreatingName { input } => Some(input),
            ManagerMode::Renaming { input, .. } => Some(input),
            _ => None,
        }
    }

    /// Get mutable reference to current input text
    pub const fn get_input_mut(&mut self) -> Option<&mut String> {
        match &mut self.mode {
            ManagerMode::CreatingName { input } => Some(input),
            ManagerMode::Renaming { input, .. } => Some(input),
            _ => None,
        }
    }
}

impl Default for CategoryManagerState {
    fn default() -> Self {
        Self::new()
    }
}

use super::Theme;

/// Render the category manager dialog
pub fn render_category_manager(
    f: &mut Frame,
    area: Rect,
    state: &CategoryManagerState,
    categories: &[Category],
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
        .title(" Category Manager (Ctrl+T) ")
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
            render_category_list(f, inner_area, state, categories, theme);
        }
        ManagerMode::CreatingName { input } => {
            render_name_input(
                f,
                inner_area,
                "Create Category",
                input,
                "Enter category name:",
                theme,
            );
        }
        ManagerMode::Renaming { input, .. } => {
            render_name_input(
                f,
                inner_area,
                "Rename Category",
                input,
                "Enter new name:",
                theme,
            );
        }
        ManagerMode::ConfirmingDelete { category_id } => {
            if let Some(category) = categories.iter().find(|c| &c.id == category_id) {
                render_delete_confirmation(f, inner_area, category, theme);
            }
        }
        ManagerMode::CreatingColor { name } => {
            // Color picker will be rendered by main UI
            render_color_picker_prompt(f, inner_area, name, theme);
        }
    }
}

/// Render the list of categories
fn render_category_list(
    f: &mut Frame,
    area: Rect,
    state: &CategoryManagerState,
    categories: &[Category],
    theme: &Theme,
) {
    // Split area for list and help text
    let chunks = ratatui::layout::Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([
            Constraint::Min(5),    // Category list
            Constraint::Length(5), // Help text
        ])
        .split(area);

    // Render category list
    let items: Vec<ListItem> = categories
        .iter()
        .enumerate()
        .map(|(i, cat)| {
            let style = if i == state.selected {
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.text)
            };

            let color_box = "█████ ".to_string();
            let content = Line::from(vec![
                Span::styled(
                    color_box,
                    Style::default().fg(Color::Rgb(cat.color.r, cat.color.g, cat.color.b)),
                ),
                Span::styled(&cat.name, style),
                Span::styled(
                    format!(" ({})", cat.id),
                    Style::default().fg(theme.text_muted),
                ),
            ]);

            ListItem::new(content)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Categories"))
        .highlight_style(Style::default().bg(theme.surface));

    f.render_widget(list, chunks[0]);

    // Render help text
    let help_text = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("n", Style::default().fg(theme.primary)),
            Span::raw(": New  "),
            Span::styled("r", Style::default().fg(theme.primary)),
            Span::raw(": Rename  "),
            Span::styled("c", Style::default().fg(theme.primary)),
            Span::raw(": Change Color  "),
            Span::styled("d", Style::default().fg(theme.primary)),
            Span::raw(": Delete"),
        ]),
        Line::from(vec![
            Span::styled("↑/↓", Style::default().fg(theme.primary)),
            Span::raw(": Navigate  "),
            Span::styled("Shift+L", Style::default().fg(theme.primary)),
            Span::raw(": Assign to Layer  "),
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
fn render_delete_confirmation(f: &mut Frame, area: Rect, category: &Category, theme: &Theme) {
    let chunks = ratatui::layout::Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Warning
            Constraint::Length(3), // Category info
            Constraint::Min(1),    // Spacer
            Constraint::Length(3), // Help
        ])
        .split(area);

    // Warning
    let warning = Paragraph::new("Are you sure you want to delete this category?")
        .alignment(Alignment::Center)
        .style(
            Style::default()
                .fg(theme.error)
                .add_modifier(Modifier::BOLD),
        );
    f.render_widget(warning, chunks[0]);

    // Category info
    let info = Line::from(vec![
        Span::raw("Category: "),
        Span::styled(&category.name, Style::default().fg(theme.accent)),
        Span::styled(
            format!(" ({})", category.id),
            Style::default().fg(theme.text_muted),
        ),
    ]);

    let info_widget = Paragraph::new(info)
        .alignment(Alignment::Center)
        .style(Style::default().fg(theme.text));
    f.render_widget(info_widget, chunks[1]);

    // Help
    let help = vec![Line::from(vec![
        Span::styled("y", Style::default().fg(theme.primary)),
        Span::raw(": Yes, delete  "),
        Span::styled("n/Esc", Style::default().fg(theme.primary)),
        Span::raw(": No, cancel"),
    ])];

    let help_widget = Paragraph::new(help)
        .alignment(Alignment::Center)
        .style(Style::default().fg(theme.text_muted));

    f.render_widget(help_widget, chunks[3]);
}

/// Render color picker prompt
fn render_color_picker_prompt(f: &mut Frame, area: Rect, name: &str, theme: &Theme) {
    let text = vec![
        Line::from(""),
        Line::from(vec![
            Span::raw("Creating category: "),
            Span::styled(
                name,
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
        Line::from("Choose a color for this category..."),
        Line::from(""),
        Line::from(vec![Span::styled(
            "(Color picker will open)",
            Style::default().fg(theme.text_muted),
        )]),
    ];

    let paragraph = Paragraph::new(text)
        .alignment(Alignment::Center)
        .style(Style::default().fg(theme.text));

    f.render_widget(paragraph, area);
}
