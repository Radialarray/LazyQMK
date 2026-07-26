//! External-change prompt for the TUI hot-reload flow.
//!
//! When the file watcher detects that the on-disk layout has been
//! modified by a process other than the running TUI, and the TUI has
//! unsaved local edits, the user is presented with this modal so they
//! can choose how to resolve the conflict.
//!
//! The three options mirror the design from the hot-reload planning
//! spec:
//!
//! 1. **Reload from disk** — discard local edits and load the new
//!    on-disk content.
//! 2. **Keep mine, overwrite disk** — save the local edits back to
//!    disk, overwriting whatever the external process wrote.
//! 3. **Save mine first, then reload** — save local edits to a
//!    `.json.local` sidecar, then reload from disk. (The sidecar is
//!    best-effort cleanup; see `cleanup_local_sidecar`.)
//!
//! The prompt also offers a `Cancel` option (Esc) which simply
//! dismisses the prompt and keeps the current in-memory state without
//! touching disk. Subsequent file changes will re-trigger the prompt.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};
use ratatui::Frame;

use crate::tui::component::Component;
use crate::tui::Theme;

/// Which button is currently highlighted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Selection {
    /// "Reload from disk" — discard local edits.
    Reload,
    /// "Keep mine, overwrite disk" — push local edits back.
    KeepMine,
    /// "Save mine first, then reload" — backup + reload.
    SaveThenReload,
    /// "Cancel" — dismiss without touching anything.
    Cancel,
}

impl Selection {
    const ALL: [Self; 4] = [
        Self::Reload,
        Self::KeepMine,
        Self::SaveThenReload,
        Self::Cancel,
    ];

    fn next(self) -> Self {
        let idx = Self::ALL.iter().position(|s| *s == self).unwrap_or(0);
        Self::ALL[(idx + 1) % Self::ALL.len()]
    }

    fn prev(self) -> Self {
        let idx = Self::ALL.iter().position(|s| *s == self).unwrap_or(0);
        Self::ALL[(idx + Self::ALL.len() - 1) % Self::ALL.len()]
    }
}

/// Outcome emitted by the prompt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExternalChangeEvent {
    /// User chose "Reload from disk" — discard local edits.
    Reload,
    /// User chose "Keep mine, overwrite disk".
    KeepMine,
    /// User chose "Save mine first, then reload".
    SaveThenReload,
    /// User dismissed the prompt without taking action.
    Cancel,
}

/// Hot-reload conflict prompt component.
#[derive(Debug, Clone)]
pub struct ExternalChangePrompt {
    /// Path of the file that changed (display only).
    pub changed_file: String,
    /// Currently highlighted button.
    selection: Selection,
}

impl ExternalChangePrompt {
    /// Creates a new prompt for the given changed file.
    #[must_use]
    pub fn new(changed_file: impl Into<String>) -> Self {
        Self {
            changed_file: changed_file.into(),
            selection: Selection::Reload,
        }
    }
}

impl Component for ExternalChangePrompt {
    type Event = ExternalChangeEvent;

    fn handle_input(&mut self, key: KeyEvent) -> Option<Self::Event> {
        match key.code {
            KeyCode::Left | KeyCode::Char('h') => {
                self.selection = self.selection.prev();
                None
            }
            KeyCode::Right | KeyCode::Char('l') => {
                self.selection = self.selection.next();
                None
            }
            KeyCode::Tab => {
                self.selection = self.selection.next();
                None
            }
            KeyCode::BackTab => {
                self.selection = self.selection.prev();
                None
            }
            KeyCode::Enter => Some(match self.selection {
                Selection::Reload => ExternalChangeEvent::Reload,
                Selection::KeepMine => ExternalChangeEvent::KeepMine,
                Selection::SaveThenReload => ExternalChangeEvent::SaveThenReload,
                Selection::Cancel => ExternalChangeEvent::Cancel,
            }),
            KeyCode::Char('r') | KeyCode::Char('R')
                if key.modifiers.is_empty() || key.modifiers == KeyModifiers::CONTROL =>
            {
                Some(ExternalChangeEvent::Reload)
            }
            KeyCode::Char('k') | KeyCode::Char('K') => Some(ExternalChangeEvent::KeepMine),
            KeyCode::Char('s') | KeyCode::Char('S')
                if key.modifiers.is_empty() || key.modifiers == KeyModifiers::CONTROL =>
            {
                Some(ExternalChangeEvent::SaveThenReload)
            }
            KeyCode::Esc => Some(ExternalChangeEvent::Cancel),
            _ => None,
        }
    }

    fn render(&self, f: &mut Frame, area: Rect, theme: &Theme) {
        // Centre the modal at roughly 60% width, 9 rows tall.
        let modal_w = area.width.saturating_sub(8).min(80);
        let modal_h = 11u16;
        let modal_w = modal_w.max(40);
        let x = area.x + (area.width.saturating_sub(modal_w)) / 2;
        let y = area.y + (area.height.saturating_sub(modal_h)) / 2;
        let modal = Rect::new(x, y, modal_w, modal_h);

        f.render_widget(Clear, modal);

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.warning))
            .title(Span::styled(
                " Confirm · External change ",
                Style::default().fg(theme.warning),
            ));
        f.render_widget(block, modal);

        let inner = Rect::new(
            modal.x + 2,
            modal.y + 2,
            modal.width.saturating_sub(4),
            modal.height.saturating_sub(4),
        );

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2), // header
                Constraint::Length(1), // spacer
                Constraint::Length(1), // buttons
                Constraint::Length(1), // spacer
                Constraint::Min(1),    // footer
            ])
            .split(inner);

        let header = Paragraph::new(vec![
            Line::from(Span::styled(
                "The layout on disk has been modified by another process.",
                Style::default().fg(theme.text),
            )),
            Line::from(Span::styled(
                format!("File: {}", self.changed_file),
                Style::default().fg(theme.text_secondary),
            )),
        ])
        .wrap(Wrap { trim: true });
        f.render_widget(header, chunks[0]);

        let button_labels = [
            (Selection::Reload, "[ Reload from disk ]"),
            (Selection::KeepMine, "[ Keep mine ]"),
            (Selection::SaveThenReload, "[ Save then reload ]"),
            (Selection::Cancel, "[ Cancel ]"),
        ];
        let mut spans: Vec<Span> = Vec::new();
        for (i, (sel, label)) in button_labels.iter().enumerate() {
            if i > 0 {
                spans.push(Span::raw("  "));
            }
            let style = if *sel == self.selection {
                Style::default()
                    .fg(theme.background)
                    .bg(theme.warning)
            } else {
                Style::default().fg(theme.text_secondary)
            };
            spans.push(Span::styled(*label, style));
        }
        f.render_widget(Paragraph::new(Line::from(spans)), chunks[2]);

        let footer = Paragraph::new(Span::styled(
            "←/→ to choose · Enter to confirm · Esc to dismiss",
            Style::default().fg(theme.text_secondary),
        ));
        f.render_widget(footer, chunks[4]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::KeyEvent;

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    #[test]
    fn test_default_selection_is_reload() {
        let prompt = ExternalChangePrompt::new("foo.json");
        assert_eq!(prompt.selection, Selection::Reload);
    }

    #[test]
    fn test_right_arrow_advances() {
        let mut p = ExternalChangePrompt::new("foo.json");
        let _ = p.handle_input(key(KeyCode::Right));
        assert_eq!(p.selection, Selection::KeepMine);
    }

    #[test]
    fn test_left_wraps_around() {
        let mut p = ExternalChangePrompt::new("foo.json");
        let _ = p.handle_input(key(KeyCode::Left));
        assert_eq!(p.selection, Selection::Cancel);
    }

    #[test]
    fn test_enter_emits_selected_event() {
        let mut p = ExternalChangePrompt::new("foo.json");
        let _ = p.handle_input(key(KeyCode::Right));
        let _ = p.handle_input(key(KeyCode::Right));
        let ev = p.handle_input(key(KeyCode::Enter));
        assert_eq!(ev, Some(ExternalChangeEvent::SaveThenReload));
    }

    #[test]
    fn test_esc_emits_cancel() {
        let mut p = ExternalChangePrompt::new("foo.json");
        let ev = p.handle_input(key(KeyCode::Esc));
        assert_eq!(ev, Some(ExternalChangeEvent::Cancel));
    }

    #[test]
    fn test_shortcut_r_emits_reload() {
        let mut p = ExternalChangePrompt::new("foo.json");
        let _ = p.handle_input(key(KeyCode::Right));
        let ev = p.handle_input(key(KeyCode::Char('r')));
        assert_eq!(ev, Some(ExternalChangeEvent::Reload));
    }

    #[test]
    fn test_shortcut_k_emits_keep_mine() {
        let mut p = ExternalChangePrompt::new("foo.json");
        let ev = p.handle_input(key(KeyCode::Char('k')));
        assert_eq!(ev, Some(ExternalChangeEvent::KeepMine));
    }
}
