//! Layout versions panel — list and manage per-layout revisions.
//!
//! Opened from the editor with Ctrl-V. Shows revision list (left) and
//! revision details (right). Supports restore (with confirmation), delete
//! (refuses current), and creating new snapshots.
//!
//! Note: this module is wired into the TUI through the popup routing layer
//! (planned in LazyQMK-zj1d.6). Until then, its public API is consumed by
//! tests only — hence the `#[allow(dead_code)]` annotations below.
#![allow(dead_code)] // Component API used by TUI popup routing (tracked in LazyQMK-zj1d.6).

use anyhow::Result;
use chrono::Utc;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Direction, Layout as RatatuiLayout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};

use crate::models::LayoutRevision;
use crate::services::layout_versions::LayoutVersionService;
use crate::tui::Theme;
use crate::tui::component::Component;

/// Events emitted by the panel.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LayoutVersionsEvent {
    /// User wants to restore the given revision (caller should confirm).
    RestoreRequested(u32),
    /// User wants to delete the given revision.
    DeleteRequested(u32),
    /// User wants to create a new snapshot with the given label.
    SnapshotRequested(Option<String>),
    /// User wants to diff the given revision with the current layout.
    DiffRequested(u32),
    /// User closed the panel.
    Closed,
}

/// State for the versions panel.
pub struct LayoutVersionsPanel {
    layout_name: String,
    service: LayoutVersionService,
    revisions: Vec<crate::models::RevisionSummary>,
    selected: usize,
    /// Confirmation prompt state, if any.
    confirm: Option<ConfirmAction>,
    /// Snapshot label entry state, if active.
    entering_label: bool,
    label_input: String,
    /// Latest error message to display.
    error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ConfirmAction {
    Restore(u32),
    Delete(u32),
}

impl LayoutVersionsPanel {
    /// Build a panel for the given layout. Loads the revision list eagerly.
    pub fn new(service: LayoutVersionService, layout_name: String) -> Result<Self> {
        let revisions = service.list(&layout_name)?;
        Ok(Self {
            layout_name,
            service,
            revisions,
            selected: 0,
            confirm: None,
            entering_label: false,
            label_input: String::new(),
            error: None,
        })
    }

    /// Reload the revision list from disk.
    pub fn refresh(&mut self) -> Result<()> {
        self.revisions = self.service.list(&self.layout_name)?;
        if self.selected >= self.revisions.len() && !self.revisions.is_empty() {
            self.selected = self.revisions.len() - 1;
        }
        Ok(())
    }

    /// Create a snapshot from the supplied layout, then refresh the list.
    pub fn snapshot(&mut self, layout: &crate::models::Layout) -> Result<()> {
        let label = if self.label_input.is_empty() {
            None
        } else {
            Some(self.label_input.clone())
        };
        match self
            .service
            .create_snapshot(&self.layout_name, layout, label.as_deref(), None, None)
        {
            Ok(_) => {
                self.label_input.clear();
                self.entering_label = false;
                self.refresh()?;
                self.error = None;
                Ok(())
        }
            Err(e) => {
                self.error = Some(e.to_string());
                Ok(())
            }
        }
    }

    /// Get the currently selected revision id, if any.
    #[must_use]
    pub fn selected_revision(&self) -> Option<u32> {
        self.revisions.get(self.selected).map(|r| r.revision)
    }

    /// Get the currently selected full revision, if any.
    #[must_use = "selected revision is meaningful to callers; ignoring the result loses the snapshot"]
    pub fn selected_full(&self) -> Result<Option<LayoutRevision>> {
        match self.selected_revision() {
            Some(rev) => Ok(Some(self.service.get(&self.layout_name, rev)?)),
            None => Ok(None),
        }
    }

    fn move_selection(&mut self, delta: isize) {
        if self.revisions.is_empty() {
            return;
        }
        let len = self.revisions.len() as isize;
        let new = (self.selected as isize + delta).rem_euclid(len);
        self.selected = new as usize;
    }
}

impl Component for LayoutVersionsPanel {
    type Event = LayoutVersionsEvent;

    fn handle_input(&mut self, key: KeyEvent) -> Option<Self::Event> {
        // Confirmation prompts consume everything.
        if let Some(action) = self.confirm.clone() {
            match key.code {
                KeyCode::Char('y') | KeyCode::Char('Y') => {
                    self.confirm = None;
                    return Some(match action {
                        ConfirmAction::Restore(rev) => LayoutVersionsEvent::RestoreRequested(rev),
                        ConfirmAction::Delete(rev) => LayoutVersionsEvent::DeleteRequested(rev),
                    });
                }
                KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                    self.confirm = None;
                    return None;
                }
                _ => return None,
            }
        }

        // Label entry mode.
        if self.entering_label {
            match key.code {
                KeyCode::Esc => {
                    self.entering_label = false;
                    self.label_input.clear();
                    return None;
                }
                KeyCode::Enter => {
                    self.entering_label = false;
                    return Some(LayoutVersionsEvent::SnapshotRequested(if self
                        .label_input
                        .is_empty()
                    {
                        None
                    } else {
                        Some(std::mem::take(&mut self.label_input))
                    }));
                }
                KeyCode::Backspace => {
                    self.label_input.pop();
                    return None;
                }
                KeyCode::Char(c) => {
                    if !c.is_control() {
                        self.label_input.push(c);
                    }
                    return None;
                }
                _ => return None,
            }
        }

        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => Some(LayoutVersionsEvent::Closed),
            KeyCode::Down | KeyCode::Char('j') => {
                self.move_selection(1);
                None
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.move_selection(-1);
                None
            }
            KeyCode::Char('r') => {
                if let Some(rev) = self.selected_revision() {
                    self.confirm = Some(ConfirmAction::Restore(rev));
                }
                None
            }
            KeyCode::Char('d') => {
                if let Some(rev) = self.selected_revision() {
                    self.confirm = Some(ConfirmAction::Delete(rev));
                }
                None
            }
            KeyCode::Char('n') => {
                self.entering_label = true;
                None
            }
            KeyCode::Char('v') => self.selected_revision().map(LayoutVersionsEvent::DiffRequested),
            _ => None,
        }
    }

    fn render(&self, f: &mut Frame, area: Rect, theme: &Theme) {
        f.render_widget(Clear, area);

        let block = Block::default()
            .title(format!(" Versions · {} ", self.layout_name))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.accent));
        f.render_widget(block, area);

        let inner = area.inner(ratatui::layout::Margin {
            horizontal: 1,
            vertical: 1,
        });

        let chunks = RatatuiLayout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
            .split(inner);

        // Left pane: revision list
        let items: Vec<ListItem> = self
            .revisions
            .iter()
            .enumerate()
            .map(|(idx, r)| {
                let active = if idx == self.selected { "▶ " } else { "  " };
                let label = r.label.as_deref().unwrap_or("-");
                let created = r.created.format("%Y-%m-%d %H:%M:%S");
                let text = format!("{active}#{:<3} {:<19} {}", r.revision, created, label);
                let style = if idx == self.selected {
                    Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(theme.text)
                };
                ListItem::new(text).style(style)
            })
            .collect();

        let list = List::new(items).block(
            Block::default()
                .title(" Revisions ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.text_secondary)),
        );
        f.render_widget(list, chunks[0]);

        // Right pane: details + footer
        let right_chunks = RatatuiLayout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(5),
                Constraint::Length(3),
                Constraint::Length(3),
            ])
            .split(chunks[1]);

        let detail_text = if let Some(r) = self.revisions.get(self.selected) {
            let label = r.label.as_deref().unwrap_or("(none)");
            let note = r.note.as_deref().unwrap_or("(none)");
            format!(
                "Revision #{}\nCreated: {}\nAuthor:  {}\nLabel:   {}\nNote:    {}\nFile:    {}",
                r.revision,
                r.created.format("%Y-%m-%dT%H:%M:%SZ"),
                r.author,
                label,
                note,
                r.filename,
            )
        } else {
            "No revisions yet. Press 'n' to create one.".to_string()
        };
        let detail = Paragraph::new(detail_text)
            .block(
                Block::default()
                    .title(" Details ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.text_secondary)),
            )
            .wrap(Wrap { trim: false });
        f.render_widget(detail, right_chunks[0]);

        // Status line: error, confirm, or label entry
        let status: Line = if let Some(err) = &self.error {
            Line::from(Span::styled(
                format!(" Error: {err}"),
                Style::default().fg(theme.error),
            ))
        } else if let Some(action) = &self.confirm {
            let msg = match action {
                ConfirmAction::Restore(rev) => {
                    format!("Restore revision #{rev}? Current will be auto-snapshotted. (y/N)")
                }
                ConfirmAction::Delete(rev) => {
                    format!("Delete revision #{rev}? This cannot be undone. (y/N)")
                }
            };
            Line::from(Span::styled(
                format!(" {msg}"),
                Style::default().fg(theme.warning),
            ))
        } else if self.entering_label {
            Line::from(Span::styled(
                format!(" Label: {}_", self.label_input),
                Style::default().fg(theme.text),
            ))
        } else {
            Line::from(Span::styled(
                format!(" {} revisions", self.revisions.len()),
                Style::default().fg(theme.text_secondary),
            ))
        };
        let status_widget = Paragraph::new(status).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.text_secondary)),
        );
        f.render_widget(status_widget, right_chunks[1]);

        // Footer keybinds
        let footer = Line::from(vec![
            Span::styled(" ↑↓", Style::default().fg(theme.text_secondary)),
            Span::styled(" select  ", Style::default().fg(theme.text)),
            Span::styled("n", Style::default().fg(theme.text_secondary)),
            Span::styled(" new  ", Style::default().fg(theme.text)),
            Span::styled("r", Style::default().fg(theme.text_secondary)),
            Span::styled(" restore  ", Style::default().fg(theme.text)),
            Span::styled("d", Style::default().fg(theme.text_secondary)),
            Span::styled(" delete  ", Style::default().fg(theme.text)),
            Span::styled("v", Style::default().fg(theme.text_secondary)),
            Span::styled(" diff  ", Style::default().fg(theme.text)),
            Span::styled("Esc", Style::default().fg(theme.text_secondary)),
            Span::styled(" close", Style::default().fg(theme.text)),
        ]);
        let footer_widget = Paragraph::new(footer).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.text_secondary)),
        );
        f.render_widget(footer_widget, right_chunks[2]);
        let _ = Color::Reset; // unused warning suppression if needed
    }
}

/// Auto-label helper for pre-compile snapshots.
#[must_use]
pub fn auto_label_now() -> String {
    crate::models::auto_label(Utc::now())
}
