//! Metadata editor dialog for editing layout metadata.

#![allow(clippy::assigning_clones)]
// Allow small types passed by reference for API consistency
#![allow(clippy::trivially_copy_pass_by_ref)]

use chrono::Utc;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

use crate::models::{Layout as LayoutModel, LayoutMetadata};
use super::component::Component;
use super::Theme;

/// Events emitted by the MetadataEditor component
#[derive(Debug, Clone)]
pub enum MetadataEditorEvent {
    /// User saved metadata changes
    MetadataUpdated {
        /// Updated layout name
        name: String,
        /// Updated description
        description: String,
        /// Updated author
        author: String,
        /// Updated tags
        tags: Vec<String>,
        /// Whether the name was changed (for renaming)
        name_changed: bool,
    },
    /// User cancelled without saving
    Cancelled,
    /// Editor closed
    Closed,
}

/// Field in the metadata editor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetadataField {
    /// Layout name field
    Name,
    /// Layout description field
    Description,
    /// Layout author field
    Author,
    /// Layout tags field
    Tags,
}

impl MetadataField {
    /// Get the next field.
    #[must_use]
    pub const fn next(self) -> Self {
        match self {
            Self::Name => Self::Description,
            Self::Description => Self::Author,
            Self::Author => Self::Tags,
            Self::Tags => Self::Name,
        }
    }

    /// Get the previous field.
    #[must_use]
    pub const fn previous(self) -> Self {
        match self {
            Self::Name => Self::Tags,
            Self::Description => Self::Name,
            Self::Author => Self::Description,
            Self::Tags => Self::Author,
        }
    }

    /// Get the field label.
    #[must_use]
    pub const fn label(&self) -> &'static str {
        match self {
            Self::Name => "Name",
            Self::Description => "Description",
            Self::Author => "Author",
            Self::Tags => "Tags",
        }
    }

    /// Get the field help text.
    #[must_use]
    pub const fn help_text(&self) -> &'static str {
        match self {
            Self::Name => "Layout name (max 100 characters) - changing this will rename the file",
            Self::Description => "Long description of the layout",
            Self::Author => "Creator name",
            Self::Tags => "Comma-separated keywords (lowercase, hyphens only)",
        }
    }
}

/// State for the metadata editor dialog.
#[derive(Debug, Clone)]
pub struct MetadataEditorState {
    /// Currently active field
    pub active_field: MetadataField,
    /// Name field value
    pub name: String,
    /// Description field value
    pub description: String,
    /// Author field value
    pub author: String,
    /// Tags field value (comma-separated)
    pub tags_input: String,
    /// Original name when editor was opened (for detecting renames)
    pub original_name: String,
}

impl MetadataEditorState {
    /// Create a new metadata editor state from layout metadata.
    #[must_use]
    pub fn new(metadata: &LayoutMetadata) -> Self {
        Self {
            active_field: MetadataField::Name,
            name: metadata.name.clone(),
            description: metadata.description.clone(),
            author: metadata.author.clone(),
            tags_input: metadata.tags.join(", "),
            original_name: metadata.name.clone(),
        }
    }

    /// Get a mutable reference to the active field's value.
    pub const fn get_active_field_mut(&mut self) -> &mut String {
        match self.active_field {
            MetadataField::Name => &mut self.name,
            MetadataField::Description => &mut self.description,
            MetadataField::Author => &mut self.author,
            MetadataField::Tags => &mut self.tags_input,
        }
    }

    /// Move to the next field.
    pub const fn next_field(&mut self) {
        self.active_field = self.active_field.next();
    }

    /// Move to the previous field.
    pub const fn previous_field(&mut self) {
        self.active_field = self.active_field.previous();
    }

    /// Parse tags from comma-separated input.
    #[must_use]
    pub fn parse_tags(&self) -> Vec<String> {
        self.tags_input
            .split(',')
            .map(|s| s.trim().to_lowercase())
            .filter(|s| !s.is_empty())
            .collect()
    }

    /// Validate the metadata.
    pub fn validate(&self) -> Result<(), String> {
        // Name validation
        if self.name.trim().is_empty() {
            return Err("Name cannot be empty".to_string());
        }
        if self.name.len() > 100 {
            return Err(format!(
                "Name exceeds maximum length of 100 characters (got {})",
                self.name.len()
            ));
        }

        // Tags validation
        for tag in self.parse_tags() {
            if !tag
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
            {
                return Err(format!(
                    "Tag '{tag}' must be lowercase with hyphens and alphanumeric characters only"
                ));
            }
        }

        Ok(())
    }

    /// Apply the changes to the layout metadata.
    pub fn apply_to_layout(&self, layout: &mut LayoutModel) -> Result<(), String> {
        // Validate first
        self.validate()?;

        // Update metadata
        layout.metadata.name = self.name.clone();
        layout.metadata.description = self.description.clone();
        layout.metadata.author = self.author.clone();
        layout.metadata.tags = self.parse_tags();
        layout.metadata.modified = Utc::now();

        Ok(())
    }

    /// Check if the name was changed from the original.
    #[must_use]
    pub fn name_changed(&self) -> bool {
        self.name != self.original_name
    }
}

impl Default for MetadataEditorState {
    fn default() -> Self {
        Self {
            active_field: MetadataField::Name,
            name: String::new(),
            description: String::new(),
            author: String::new(),
            tags_input: String::new(),
            original_name: String::new(),
        }
    }
}

/// MetadataEditor component that implements the Component trait
#[derive(Debug, Clone)]
pub struct MetadataEditor {
    /// Internal state of the metadata editor
    state: MetadataEditorState,
}

impl MetadataEditor {
    /// Create a new MetadataEditor from layout metadata
    #[must_use]
    pub fn new(metadata: &LayoutMetadata) -> Self {
        Self {
            state: MetadataEditorState::new(metadata),
        }
    }

    /// Get a reference to the internal state
    #[must_use]
    pub const fn state(&self) -> &MetadataEditorState {
        &self.state
    }

    /// Get a mutable reference to the internal state
    pub fn state_mut(&mut self) -> &mut MetadataEditorState {
        &mut self.state
    }
}

impl Component for MetadataEditor {
    type Event = MetadataEditorEvent;

    fn handle_input(&mut self, key: KeyEvent) -> Option<Self::Event> {
        match handle_metadata_editor_input(&mut self.state, key) {
            MetadataEditorAction::Confirm => {
                // Validate before emitting event
                if self.state.validate().is_ok() {
                    Some(MetadataEditorEvent::MetadataUpdated {
                        name: self.state.name.clone(),
                        description: self.state.description.clone(),
                        author: self.state.author.clone(),
                        tags: self.state.parse_tags(),
                        name_changed: self.state.name_changed(),
                    })
                } else {
                    // Validation failed - don't emit event, stay in editor
                    None
                }
            }
            MetadataEditorAction::Cancel => Some(MetadataEditorEvent::Cancelled),
            MetadataEditorAction::Continue => None,
        }
    }

    fn render(&self, f: &mut Frame, _area: Rect, theme: &Theme) {
        render_metadata_editor(f, &self.state, theme);
    }
}

/// Render the metadata editor dialog.
pub fn render_metadata_editor(f: &mut Frame, state: &MetadataEditorState, theme: &Theme) {
    let area = centered_rect(70, 60, f.size());

    // Clear the background
    f.render_widget(Clear, area);

    // Create the main block
    let block = Block::default()
        .title(" Edit Metadata ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.primary));

    let inner_area = block.inner(area);
    f.render_widget(block, area);

    // Split into sections: fields + help
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Name
            Constraint::Length(3), // Description
            Constraint::Length(3), // Author
            Constraint::Length(3), // Tags
            Constraint::Min(1),    // Help text
            Constraint::Length(2), // Controls
        ])
        .margin(1)
        .split(inner_area);

    // Render each field
    render_field(
        f,
        chunks[0],
        MetadataField::Name,
        &state.name,
        state.active_field == MetadataField::Name,
        theme,
    );
    render_field(
        f,
        chunks[1],
        MetadataField::Description,
        &state.description,
        state.active_field == MetadataField::Description,
        theme,
    );
    render_field(
        f,
        chunks[2],
        MetadataField::Author,
        &state.author,
        state.active_field == MetadataField::Author,
        theme,
    );
    render_field(
        f,
        chunks[3],
        MetadataField::Tags,
        &state.tags_input,
        state.active_field == MetadataField::Tags,
        theme,
    );

    // Render help text for active field
    let help_text = state.active_field.help_text();
    let help_paragraph = Paragraph::new(help_text)
        .style(Style::default().fg(theme.text_muted))
        .wrap(Wrap { trim: true });
    f.render_widget(help_paragraph, chunks[4]);

    // Render controls
    let controls_text = vec![Line::from(vec![
        Span::styled(
            "Enter",
            Style::default()
                .fg(theme.success)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" save  "),
        Span::styled(
            "Esc",
            Style::default()
                .fg(theme.error)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" cancel  "),
        Span::styled(
            "Tab",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" next field  "),
        Span::styled(
            "Shift+Tab",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" prev field"),
    ])];
    let controls = Paragraph::new(controls_text).alignment(Alignment::Center);
    f.render_widget(controls, chunks[5]);
}

/// Render a single field.
fn render_field(
    f: &mut Frame,
    area: Rect,
    field: MetadataField,
    value: &str,
    is_active: bool,
    theme: &Theme,
) {
    let label = field.label();
    let style = if is_active {
        Style::default()
            .fg(theme.accent)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme.text)
    };

    let border_style = if is_active {
        Style::default().fg(theme.accent)
    } else {
        Style::default().fg(theme.text_muted)
    };

    // Display value with cursor if active
    let display_value = if is_active {
        format!("{value}_")
    } else {
        value.to_string()
    };

    let block = Block::default()
        .title(label)
        .borders(Borders::ALL)
        .border_style(border_style);

    let paragraph = Paragraph::new(display_value).style(style).block(block);

    f.render_widget(paragraph, area);
}

/// Handle input for the metadata editor.
pub fn handle_metadata_editor_input(
    state: &mut MetadataEditorState,
    key: KeyEvent,
) -> MetadataEditorAction {
    match (key.code, key.modifiers) {
        // Confirm (Enter)
        (KeyCode::Enter, KeyModifiers::NONE) => MetadataEditorAction::Confirm,

        // Cancel (Escape)
        (KeyCode::Esc, _) => MetadataEditorAction::Cancel,

        // Next field (Tab)
        (KeyCode::Tab, KeyModifiers::NONE) => {
            state.next_field();
            MetadataEditorAction::Continue
        }

        // Previous field (Shift+Tab)
        (KeyCode::BackTab, _) => {
            state.previous_field();
            MetadataEditorAction::Continue
        }

        // Backspace
        (KeyCode::Backspace, _) => {
            let field = state.get_active_field_mut();
            field.pop();
            MetadataEditorAction::Continue
        }

        // Character input
        (KeyCode::Char(c), KeyModifiers::NONE | KeyModifiers::SHIFT) => {
            let field = state.get_active_field_mut();
            field.push(c);
            MetadataEditorAction::Continue
        }

        // Ignore other keys
        _ => MetadataEditorAction::Continue,
    }
}

/// Action returned by metadata editor input handler.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetadataEditorAction {
    /// Continue editing
    Continue,
    /// Confirm changes
    Confirm,
    /// Cancel editing
    Cancel,
}

/// Create a centered rect with the given percentage width and height.
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
