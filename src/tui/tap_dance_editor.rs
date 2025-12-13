//! Tap Dance Editor Component
//!
//! Provides UI for browsing, selecting, and managing tap dance actions.
//! This is a view-only component - creation/editing happens through multi-stage picker flow.

use crate::models::Layout;
use crate::tui::theme::Theme;
use crate::tui::Component;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Direction, Layout as RatatuiLayout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};

/// Events emitted by the tap dance editor
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TapDanceEditorEvent {
    /// User selected a tap dance to apply to current key
    Selected(String),
    /// User wants to create a new tap dance
    CreateNew,
    /// User wants to edit an existing tap dance at index
    Edit(usize),
    /// User wants to delete a tap dance
    Delete(String),
    /// User cancelled the operation
    Cancelled,
}

/// Tap Dance Editor component state
#[derive(Debug)]
pub struct TapDanceEditor {
    /// List selection state
    list_state: ListState,
    /// Tap dances from the layout (read-only view)
    tap_dances: Vec<crate::models::TapDanceAction>,
}

impl TapDanceEditor {
    /// Creates a new tap dance editor with tap dances from the layout
    pub fn new(layout: &Layout) -> Self {
        let tap_dances = layout.tap_dances.clone();
        let mut list_state = ListState::default();
        
        // Select first item if any exist
        if !tap_dances.is_empty() {
            list_state.select(Some(0));
        }
        
        Self {
            list_state,
            tap_dances,
        }
    }

    /// Get the currently selected tap dance name
    fn selected_name(&self) -> Option<String> {
        self.list_state
            .selected()
            .and_then(|idx| self.tap_dances.get(idx))
            .map(|td| td.name.clone())
    }

    /// Get the currently selected index
    fn selected_index(&self) -> Option<usize> {
        self.list_state.selected()
    }
}

impl Component for TapDanceEditor {
    type Event = TapDanceEditorEvent;

    fn handle_input(&mut self, key: KeyEvent) -> Option<Self::Event> {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                if let Some(selected) = self.list_state.selected() {
                    if selected > 0 {
                        self.list_state.select(Some(selected - 1));
                    }
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if let Some(selected) = self.list_state.selected() {
                    if selected < self.tap_dances.len().saturating_sub(1) {
                        self.list_state.select(Some(selected + 1));
                    }
                }
            }
            KeyCode::Enter => {
                if let Some(name) = self.selected_name() {
                    return Some(TapDanceEditorEvent::Selected(name));
                }
            }
            KeyCode::Char('n') => {
                return Some(TapDanceEditorEvent::CreateNew);
            }
            KeyCode::Char('e') => {
                if let Some(index) = self.selected_index() {
                    return Some(TapDanceEditorEvent::Edit(index));
                }
            }
            KeyCode::Char('d') => {
                if let Some(name) = self.selected_name() {
                    return Some(TapDanceEditorEvent::Delete(name));
                }
            }
            KeyCode::Esc => {
                return Some(TapDanceEditorEvent::Cancelled);
            }
            _ => {}
        }
        None
    }

    fn render(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        // Need mutable self for rendering stateful widgets, use clone pattern
        let mut editor = self.clone();
        editor.render_impl(frame, area, theme);
    }
}

impl TapDanceEditor {
    /// Internal render implementation with mutable self
    fn render_impl(&mut self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let chunks = RatatuiLayout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(3), Constraint::Length(3)])
            .split(area);

        // List of tap dances
        let items: Vec<ListItem> = if self.tap_dances.is_empty() {
            vec![ListItem::new(
                Line::from("No tap dances defined. Press 'n' to create one.")
                    .style(Style::default().fg(theme.text_muted)),
            )]
        } else {
            self.tap_dances
                .iter()
                .map(|td| {
                    let mut parts = vec![format!("{}: {}", td.name, td.single_tap)];
                    if let Some(ref double) = td.double_tap {
                        parts.push(format!(" → {double}"));
                    }
                    if let Some(ref hold) = td.hold {
                        parts.push(format!(" (hold: {hold})"));
                    }
                    ListItem::new(parts.join(""))
                })
                .collect()
        };

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Tap Dances ")
                    .style(Style::default().bg(theme.background)),
            )
            .highlight_style(
                Style::default()
                    .bg(theme.highlight_bg)
                    .fg(theme.text)
                    .add_modifier(Modifier::BOLD),
            )
            .style(Style::default().bg(theme.background).fg(theme.text));

        frame.render_stateful_widget(list, chunks[0], &mut self.list_state);

        // Help text
        let help = Paragraph::new(vec![
            Line::from(vec![
                Span::styled("↑/↓", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
                Span::raw(" Navigate  "),
                Span::styled("Enter", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
                Span::raw(" Apply  "),
                Span::styled("n", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
                Span::raw(" New  "),
                Span::styled("e", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
                Span::raw(" Edit  "),
                Span::styled("d", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
                Span::raw(" Delete  "),
                Span::styled("Esc", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
                Span::raw(" Cancel"),
            ]),
        ])
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Help ")
                .style(Style::default().bg(theme.background)),
        )
        .style(Style::default().bg(theme.background).fg(theme.text));

        frame.render_widget(help, chunks[1]);
    }
}

// Implement Clone for TapDanceEditor (needed for render pattern)
impl Clone for TapDanceEditor {
    fn clone(&self) -> Self {
        Self {
            list_state: ListState::default().with_selected(self.list_state.selected()),
            tap_dances: self.tap_dances.clone(),
        }
    }
}
