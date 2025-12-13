//! Tap Dance Form Dialog (Option B UX)
//!
//! Single form dialog for creating/editing tap dance actions with inline pickers.
//! Shows all fields (Name, Single Tap, Double Tap, Hold) in one form with Pick buttons.

use crate::models::TapDanceAction;
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

/// Events emitted by the tap dance form
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TapDanceFormEvent {
    /// User wants to pick a keycode for Single Tap field
    PickSingle,
    /// User wants to pick a keycode for Double Tap field
    PickDouble,
    /// User wants to pick a keycode for Hold field
    PickHold,
    /// User saved the tap dance (with draft and optional editing index)
    Save(TapDanceAction, Option<usize>),
    /// User cancelled the operation
    Cancel,
}

/// Form row selection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormRow {
    /// Name field (text input)
    Name,
    /// Single tap field (picker)
    Single,
    /// Double tap field (picker)
    Double,
    /// Hold field (picker)
    Hold,
}

impl FormRow {
    /// Get next row (wraps around)
    const fn next(&self) -> Self {
        match self {
            Self::Name => Self::Single,
            Self::Single => Self::Double,
            Self::Double => Self::Hold,
            Self::Hold => Self::Name,
        }
    }

    /// Get previous row (wraps around)
    const fn previous(&self) -> Self {
        match self {
            Self::Name => Self::Hold,
            Self::Single => Self::Name,
            Self::Double => Self::Single,
            Self::Hold => Self::Double,
        }
    }
}

/// Tap Dance Form component state
#[derive(Debug, Clone)]
pub struct TapDanceForm {
    /// Draft tap dance action being created/edited
    draft: TapDanceAction,
    /// Index of tap dance being edited (None = creating new)
    editing_index: Option<usize>,
    /// Currently selected row
    selected_row: FormRow,
    /// Name input buffer (for editing)
    name_input: String,
    /// Error message (if validation fails)
    error: Option<String>,
    /// Existing tap dance names (for duplicate checking)
    existing_names: Vec<String>,
    /// Whether name field is in edit mode
    name_editing: bool,
}

impl TapDanceForm {
    /// Creates a new form for creating a tap dance
    pub fn new_create(existing_names: Vec<String>) -> Self {
        Self {
            draft: TapDanceAction::new(String::new(), String::new()),
            editing_index: None,
            selected_row: FormRow::Name,
            name_input: String::new(),
            error: None,
            existing_names,
            name_editing: false,
        }
    }

    /// Creates a new form for editing an existing tap dance
    pub fn new_edit(
        tap_dance: TapDanceAction,
        index: usize,
        existing_names: Vec<String>,
    ) -> Self {
        let name_input = tap_dance.name.clone();
        Self {
            draft: tap_dance,
            editing_index: Some(index),
            selected_row: FormRow::Name,
            name_input,
            error: None,
            existing_names,
            name_editing: false,
        }
    }

    /// Get the number of required fields completed (out of 3: name, single, double)
    fn count_required_complete(&self) -> usize {
        let mut count = 0;
        if !self.draft.name.is_empty() {
            count += 1;
        }
        if !self.draft.single_tap.is_empty() {
            count += 1;
        }
        if self.draft.double_tap.is_some() {
            count += 1;
        }
        count
    }

    /// Check if the form can be saved (all required fields filled)
    fn can_save(&self) -> bool {
        !self.draft.name.is_empty()
            && !self.draft.single_tap.is_empty()
            && self.draft.double_tap.is_some()
    }

    /// Validate name field
    fn validate_name(&self, name: &str) -> Result<(), String> {
        if name.trim().is_empty() {
            return Err("Name cannot be empty".to_string());
        }

        if !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
            return Err("Name must be alphanumeric with underscores only".to_string());
        }

        // Check for duplicates (skip if editing the same tap dance)
        if let Some(existing_index) = self.editing_index {
            // Allow keeping the same name when editing
            if let Some(existing) = self.existing_names.get(existing_index) {
                if existing == name {
                    return Ok(());
                }
            }
        }

        if self.existing_names.iter().any(|n| n == name) {
            return Err(format!("Tap dance '{name}' already exists"));
        }

        Ok(())
    }

    /// Apply name from input buffer to draft
    fn apply_name(&mut self) {
        let trimmed = self.name_input.trim().to_string();
        match self.validate_name(&trimmed) {
            Ok(()) => {
                self.draft.name = trimmed;
                self.error = None;
            }
            Err(e) => {
                self.error = Some(e);
            }
        }
    }

    /// Update single tap keycode
    pub fn set_single_tap(&mut self, keycode: String) {
        self.draft.single_tap = keycode;
        self.error = None;
    }

    /// Update double tap keycode
    pub fn set_double_tap(&mut self, keycode: String) {
        self.draft.double_tap = Some(keycode);
        self.error = None;
    }

    /// Update hold keycode
    pub fn set_hold(&mut self, keycode: String) {
        self.draft.hold = Some(keycode);
        self.error = None;
    }

    /// Clear hold keycode
    pub fn clear_hold(&mut self) {
        self.draft.hold = None;
        self.error = None;
    }
}

impl Component for TapDanceForm {
    type Event = TapDanceFormEvent;

    fn handle_input(&mut self, key: KeyEvent) -> Option<Self::Event> {
        // If name field is in edit mode, handle text input
        if self.name_editing && self.selected_row == FormRow::Name {
            match key.code {
                KeyCode::Char(c) if key.modifiers.is_empty() => {
                    self.name_input.push(c);
                    self.error = None;
                    return None;
                }
                KeyCode::Backspace => {
                    self.name_input.pop();
                    self.error = None;
                    return None;
                }
                KeyCode::Enter => {
                    // Apply name and exit edit mode
                    self.apply_name();
                    self.name_editing = false;
                    return None;
                }
                KeyCode::Esc => {
                    // Cancel edit, restore original name
                    self.name_input = self.draft.name.clone();
                    self.name_editing = false;
                    self.error = None;
                    return None;
                }
                _ => return None,
            }
        }

        // Normal navigation and action handling
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                self.selected_row = self.selected_row.previous();
                self.error = None;
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.selected_row = self.selected_row.next();
                self.error = None;
            }
            KeyCode::Tab => {
                self.selected_row = self.selected_row.next();
                self.error = None;
            }
            KeyCode::BackTab => {
                self.selected_row = self.selected_row.previous();
                self.error = None;
            }
            KeyCode::Enter | KeyCode::Right => {
                match self.selected_row {
                    FormRow::Name => {
                        // Enter edit mode for name field
                        self.name_editing = true;
                        self.error = None;
                    }
                    FormRow::Single => {
                        return Some(TapDanceFormEvent::PickSingle);
                    }
                    FormRow::Double => {
                        return Some(TapDanceFormEvent::PickDouble);
                    }
                    FormRow::Hold => {
                        return Some(TapDanceFormEvent::PickHold);
                    }
                }
            }
            KeyCode::Backspace => {
                if self.selected_row == FormRow::Name && !self.name_editing {
                    // Delete name
                    self.name_input.clear();
                    self.draft.name.clear();
                    self.error = None;
                } else if self.selected_row == FormRow::Hold {
                    // Clear hold (optional field)
                    self.clear_hold();
                }
            }
            KeyCode::Char('s') if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => {
                // Ctrl+S to save
                if self.can_save() {
                    // Final validation
                    if let Err(e) = self.draft.validate() {
                        self.error = Some(format!("Validation failed: {e}"));
                        return None;
                    }
                    return Some(TapDanceFormEvent::Save(
                        self.draft.clone(),
                        self.editing_index,
                    ));
                } else {
                    self.error = Some("Cannot save: fill all required fields".to_string());
                }
            }
            KeyCode::Esc => {
                return Some(TapDanceFormEvent::Cancel);
            }
            _ => {}
        }
        None
    }

    fn render(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        // Center the dialog (larger than name entry, more fields)
        let dialog_area = centered_rect(70, 60, area);

        // Draw translucent-ish backdrop behind the modal
        frame.render_widget(Clear, dialog_area);
        let backdrop = Block::default().style(Style::default().bg(theme.surface));
        frame.render_widget(backdrop, dialog_area);

        // Split into sections
        let chunks = RatatuiLayout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Title
                Constraint::Length(3), // Name field
                Constraint::Length(3), // Single tap field
                Constraint::Length(3), // Double tap field
                Constraint::Length(3), // Hold field
                Constraint::Length(3), // Status line (required X/3)
                Constraint::Min(2),    // Error message (if any)
                Constraint::Length(3), // Action buttons
            ])
            .split(dialog_area);

        // Title
        let title_text = if self.editing_index.is_some() {
            "Edit Tap Dance"
        } else {
            "Create Tap Dance"
        };
        let title = Paragraph::new(title_text)
            .style(
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            )
            .block(Block::default().borders(Borders::ALL).style(Style::default().bg(theme.background)));
        frame.render_widget(title, chunks[0]);

        // Name field
        let name_style = if self.selected_row == FormRow::Name {
            Style::default().fg(theme.accent)
        } else {
            Style::default().fg(theme.text)
        };
        let name_text = if self.name_editing && self.selected_row == FormRow::Name {
            format!("Name: {}█", self.name_input)
        } else if !self.draft.name.is_empty() {
            format!("Name: {}", self.draft.name)
        } else {
            "Name: (required)".to_string()
        };
        let name_label = if self.selected_row == FormRow::Name {
            " Name [REQUIRED] ▶ "
        } else {
            " Name [REQUIRED] "
        };
        let name = Paragraph::new(name_text)
            .style(name_style)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(name_label)
                    .style(Style::default().bg(theme.background)),
            );
        frame.render_widget(name, chunks[1]);

        // Single tap field
        let single_style = if self.selected_row == FormRow::Single {
            Style::default().fg(theme.accent)
        } else {
            Style::default().fg(theme.text)
        };
        let single_text = if !self.draft.single_tap.is_empty() {
            format!("Single: {} [Pick]", self.draft.single_tap)
        } else {
            "Single: — [Pick]".to_string()
        };
        let single_label = if self.selected_row == FormRow::Single {
            " Single Tap [REQUIRED] ▶ "
        } else {
            " Single Tap [REQUIRED] "
        };
        let single = Paragraph::new(single_text)
            .style(single_style)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(single_label)
                    .style(Style::default().bg(theme.background)),
            );
        frame.render_widget(single, chunks[2]);

        // Double tap field
        let double_style = if self.selected_row == FormRow::Double {
            Style::default().fg(theme.accent)
        } else {
            Style::default().fg(theme.text)
        };
        let double_text = if let Some(ref double) = self.draft.double_tap {
            format!("Double: {} [Pick]", double)
        } else {
            "Double: — [Pick]".to_string()
        };
        let double_label = if self.selected_row == FormRow::Double {
            " Double Tap [REQUIRED] ▶ "
        } else {
            " Double Tap [REQUIRED] "
        };
        let double = Paragraph::new(double_text)
            .style(double_style)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(double_label)
                    .style(Style::default().bg(theme.background)),
            );
        frame.render_widget(double, chunks[3]);

        // Hold field
        let hold_style = if self.selected_row == FormRow::Hold {
            Style::default().fg(theme.accent)
        } else {
            Style::default().fg(theme.text)
        };
        let hold_text = if let Some(ref hold) = self.draft.hold {
            format!("Hold: {} [Pick/Clear]", hold)
        } else {
            "Hold: — [Pick/Skip]".to_string()
        };
        let hold_label = if self.selected_row == FormRow::Hold {
            " Hold [OPTIONAL] ▶ "
        } else {
            " Hold [OPTIONAL] "
        };
        let hold = Paragraph::new(hold_text)
            .style(hold_style)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(hold_label)
                    .style(Style::default().bg(theme.background)),
            );
        frame.render_widget(hold, chunks[4]);

        // Status line
        let required_count = self.count_required_complete();
        let status_text = format!("Required fields: {required_count}/3 complete");
        let status_style = if required_count == 3 {
            Style::default().fg(theme.success)
        } else {
            Style::default().fg(theme.warning)
        };
        let status = Paragraph::new(status_text)
            .style(status_style)
            .block(Block::default().borders(Borders::ALL).style(Style::default().bg(theme.background)));
        frame.render_widget(status, chunks[5]);

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
            frame.render_widget(error_widget, chunks[6]);
        }

        // Action buttons
        let save_text = if self.can_save() {
            Span::styled(
                "Ctrl+S",
                Style::default()
                    .fg(theme.success)
                    .add_modifier(Modifier::BOLD),
            )
        } else {
            Span::styled("Ctrl+S", Style::default().fg(theme.text_muted))
        };
        let help = Paragraph::new(vec![Line::from(vec![
            Span::styled(
                "↑/↓/Tab",
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" Navigate  "),
            Span::styled(
                "Enter",
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" Edit/Pick  "),
            save_text,
            Span::raw(" Save  "),
            Span::styled(
                "Esc",
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" Cancel"),
        ])])
        .style(Style::default().fg(theme.text).bg(theme.background))
        .block(Block::default().borders(Borders::ALL).style(Style::default().bg(theme.background)));
        frame.render_widget(help, chunks[7]);
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
