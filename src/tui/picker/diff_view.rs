//! Layout diff view — show changes between two revisions.
//!
//! Opened from the versions panel with `v`. Shows summary counts at the top
//! and a scrollable list of layer/setting changes below. Supports restoring
//! the target revision from this view.
//!
//! Note: this module is wired into the TUI through the popup routing layer
//! (planned in LazyQMK-zj1d.6). Until then, its public API is consumed by
//! tests only — hence the `#[allow(dead_code)]` annotations below.
#![allow(dead_code)] // Component API used by TUI popup routing (tracked in LazyQMK-zj1d.6).

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Direction, Layout as RatatuiLayout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

use crate::models::{LayerDiff, LayoutDiff, SettingDiff};
use crate::services::layout_versions::LayoutVersionService;
use crate::tui::Theme;
use crate::tui::component::Component;

/// Events emitted by the diff view.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiffViewEvent {
    /// User wants to restore the "to" revision.
    RestoreRequested,
    /// User closed the diff view.
    Closed,
}

/// State for the diff view.
pub struct DiffView {
    layout_name: String,
    from_revision: u32,
    to_revision: u32,
    diff: LayoutDiff,
    scroll: usize,
}

impl DiffView {
    /// Build the diff view by computing the diff between two revisions.
    pub fn new(
        service: &LayoutVersionService,
        layout_name: String,
        from_revision: u32,
        to_revision: u32,
    ) -> Result<Self> {
        let diff = service.diff(&layout_name, from_revision, to_revision)?;
        Ok(Self {
            layout_name,
            from_revision,
            to_revision,
            diff,
            scroll: 0,
        })
    }

    /// Get the diff body (for tests / external rendering).
    #[must_use]
    pub fn diff(&self) -> &LayoutDiff {
        &self.diff
    }

    fn lines(&self) -> Vec<Line<'_>> {
        let mut lines: Vec<Line> = Vec::new();
        let s = &self.diff.summary;
        lines.push(Line::from(vec![
            Span::styled("  Layers: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(format!(
                "+{} -{} ~{} keys",
                s.layers_added, s.layers_removed, s.keys_changed
            )),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  Flags:  ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(format!(
                "rgb={} combos={} tap_dances={} meta={}",
                yes_no(s.rgb_changed),
                yes_no(s.combos_changed),
                yes_no(s.tap_dances_changed),
                yes_no(s.metadata_changed),
            )),
        ]));
        lines.push(Line::raw(""));

        for change in &self.diff.layer_changes {
            match change {
                LayerDiff::Added { index, layer } => {
                    lines.push(Line::from(Span::styled(
                        format!("+ Layer {} '{}' (added)", index, layer.name),
                        Style::default().add_modifier(Modifier::BOLD),
                    )));
                }
                LayerDiff::Removed { index, name } => {
                    lines.push(Line::from(Span::styled(
                        format!("- Layer {} '{}' (removed)", index, name),
                        Style::default().add_modifier(Modifier::BOLD),
                    )));
                }
                LayerDiff::Renamed { index, from, to } => {
                    lines.push(Line::from(Span::styled(
                        format!("~ Layer {} renamed: '{}' → '{}'", index, from, to),
                        Style::default().add_modifier(Modifier::BOLD),
                    )));
                }
                LayerDiff::KeysChanged {
                    index,
                    name,
                    changes,
                } => {
                    lines.push(Line::from(Span::styled(
                        format!("~ Layer {} '{}' ({} keys)", index, name, changes.len()),
                        Style::default().add_modifier(Modifier::BOLD),
                    )));
                    for c in changes {
                        lines.push(Line::from(format!(
                            "    ({},{}): {} → {}",
                            c.row, c.col, c.from, c.to
                        )));
                    }
                }
            }
        }

        if !self.diff.setting_changes.is_empty() {
            lines.push(Line::raw(""));
            lines.push(Line::from(Span::styled(
                "Settings:",
                Style::default().add_modifier(Modifier::BOLD),
            )));
            for s in &self.diff.setting_changes {
                lines.push(setting_line(s));
            }
        }
        lines
    }
}

fn yes_no(b: bool) -> &'static str {
    if b { "yes" } else { "no" }
}

fn setting_line(s: &SettingDiff) -> Line<'static> {
    Line::from(format!("  {}: {} → {}", s.path, s.from, s.to))
}

impl Component for DiffView {
    type Event = DiffViewEvent;

    fn handle_input(&mut self, key: KeyEvent) -> Option<Self::Event> {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => Some(DiffViewEvent::Closed),
            KeyCode::Char('r') => Some(DiffViewEvent::RestoreRequested),
            KeyCode::Down | KeyCode::Char('j') => {
                self.scroll = self.scroll.saturating_add(1);
                None
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.scroll = self.scroll.saturating_sub(1);
                None
            }
            KeyCode::PageDown => {
                self.scroll = self.scroll.saturating_add(10);
                None
            }
            KeyCode::PageUp => {
                self.scroll = self.scroll.saturating_sub(10);
                None
            }
            _ => None,
        }
    }

    fn render(&self, f: &mut Frame, area: Rect, theme: &Theme) {
        f.render_widget(Clear, area);

        let block = Block::default()
            .title(format!(
                " Diff · {} · #{} → #{} ",
                self.layout_name, self.from_revision, self.to_revision
            ))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.accent));
        f.render_widget(block, area);

        let inner = area.inner(ratatui::layout::Margin {
            horizontal: 1,
            vertical: 1,
        });

        let chunks = RatatuiLayout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(5), Constraint::Length(3)])
            .split(inner);

        let lines = self.lines();
        let visible_lines: Vec<Line> = lines
            .into_iter()
            .skip(self.scroll)
            .collect();

        let paragraph = Paragraph::new(visible_lines).wrap(Wrap { trim: false }).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.text_secondary)),
        );
        f.render_widget(paragraph, chunks[0]);

        let footer = Line::from(vec![
            Span::styled(" ↑↓", Style::default().fg(theme.text_secondary)),
            Span::styled(" scroll  ", Style::default().fg(theme.text)),
            Span::styled("r", Style::default().fg(theme.text_secondary)),
            Span::styled(" restore  ", Style::default().fg(theme.text)),
            Span::styled("Esc", Style::default().fg(theme.text_secondary)),
            Span::styled(" close", Style::default().fg(theme.text)),
        ]);
        let footer_widget = Paragraph::new(footer).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.text_secondary)),
        );
        f.render_widget(footer_widget, chunks[1]);
    }
}
