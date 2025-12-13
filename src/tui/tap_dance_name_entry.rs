//! Tap Dance Name Entry Dialog
//!
//! Simple text input dialog for entering a tap dance name.
//! Validates that the name is a valid C identifier.

use crate::tui::theme::Theme;
use crate::tui::Component;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Direction, Layout as RatatuiLayout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

/// Events emitted by the tap dance name entry dialog
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TapDanceNameEntryEvent {
    /// User confirmed the name
    Confirmed(String),
    /// User cancelled the operation
    Cancelled,
}

/// Tap Dance Name Entry component state
#[derive(Debug, Clone)]
pub struct TapDanceNameEntry {
    /// Current input buffer
    input: String,
    /// Error message (if validation fails)
    error: Option<String>,
    /// Existing tap dance names (for duplicate checking)
    existing_names: Vec<String>,
}

impl TapDanceNameEntry {
    /// Creates a new tap dance name entry dialog
    pub fn new(existing_names: Vec<String>) -> Self {
        Self {
            input: String::new(),
            error: None,
            existing_names,
        }
    }

    /// Validate the input name
    fn validate(&self) -> Result<(), String> {
        // Check if empty
        if self.input.trim().is_empty() {
            return Err("Name cannot be empty".to_string());
        }

        // Check if valid C identifier (alphanumeric + underscore)
        if !self.input.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
            return Err("Name must contain only alphanumeric characters and underscores".to_string());
        }

        // Check for duplicates
        if self.existing_names.iter().any(|n| n == &self.input) {
            return Err(format!("Tap dance '{}' already exists", self.input));
        }

        Ok(())
    }
}

impl Component for TapDanceNameEntry {
    type Event = TapDanceNameEntryEvent;

    fn handle_input(&mut self, key: KeyEvent) -> Option<Self::Event> {
        match key.code {
            KeyCode::Char(c) if key.modifiers.is_empty() => {
                self.input.push(c);
                self.error = None; // Clear error on new input
            }
            KeyCode::Backspace => {
                self.input.pop();
                self.error = None; // Clear error on edit
            }
            KeyCode::Enter => {
                // Validate and confirm
                match self.validate() {
                    Ok(()) => {
                        return Some(TapDanceNameEntryEvent::Confirmed(self.input.clone()));
                    }
                    Err(e) => {
                        self.error = Some(e);
                    }
                }
            }
            KeyCode::Esc => {
                return Some(TapDanceNameEntryEvent::Cancelled);
            }
            _ => {}
        }
        None
    }

    fn render(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        // Center the dialog
        let dialog_area = centered_rect(60, 40, area);

        // Clear background
        frame.render_widget(Clear, dialog_area);

        // Render background block
        let background = Block::default().style(Style::default().bg(theme.background));
        frame.render_widget(background, dialog_area);

        // Split into sections
        let chunks = RatatuiLayout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Title
                Constraint::Length(3), // Input field
                Constraint::Length(3), // Validation hint
                Constraint::Min(2),    // Error message (if any)
                Constraint::Length(2), // Help text
            ])
            .split(dialog_area);

        // Title
        let title = Paragraph::new("Enter Tap Dance Name")
            .style(
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            )
            .block(Block::default().borders(Borders::ALL).style(Style::default().bg(theme.background)));
        frame.render_widget(title, chunks[0]);

        // Input field with cursor
        let input_text = format!("{}█", self.input);
        let input = Paragraph::new(input_text)
            .style(Style::default().fg(theme.text))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Name ")
                    .style(Style::default().bg(theme.background)),
            );
        frame.render_widget(input, chunks[1]);

        // Validation hint
        let hint = Paragraph::new("Alphanumeric and underscores only")
            .style(Style::default().fg(theme.text_muted))
            .block(Block::default().borders(Borders::ALL).style(Style::default().bg(theme.background)));
        frame.render_widget(hint, chunks[2]);

        // Error message (if any)
        if let Some(ref error) = self.error {
            let error_widget = Paragraph::new(error.as_str())
                .style(Style::default().fg(theme.error))
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(" Error ")
                        .style(Style::default().bg(theme.background)),
                )
                .wrap(Wrap { trim: true });
            frame.render_widget(error_widget, chunks[3]);
        }

        // Help text
        let help = Paragraph::new(vec![Line::from(vec![
            Span::styled(
                "Enter",
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" Confirm  "),
            Span::styled(
                "Esc",
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" Cancel"),
        ])])
        .style(Style::default().fg(theme.text).bg(theme.background));
        frame.render_widget(help, chunks[4]);
    }
}

/// Helper to create a centered rectangle
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = RatatuiLayout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    RatatuiLayout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
