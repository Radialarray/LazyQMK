//! Build log viewer widget for displaying compilation progress and logs.

use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};

use crate::firmware::BuildState;

/// Events emitted by the BuildLog component
#[derive(Debug, Clone)]
pub enum BuildLogEvent {
    /// User closed the build log
    Closed,
}

/// Build log viewer state with scrolling support.
#[derive(Debug, Clone)]
pub struct BuildLogState {
    /// Scroll offset (number of lines from top)
    pub scroll_offset: usize,
    /// Whether the log viewer is visible
    #[allow(dead_code)] // Part of state struct, may be used in future visibility toggle
    pub visible: bool,
}

impl BuildLogState {
    /// Creates a new build log state.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            scroll_offset: 0,
            visible: false,
        }
    }

    /// Scrolls the log view up by one line.
    pub const fn scroll_up(&mut self) {
        if self.scroll_offset > 0 {
            self.scroll_offset -= 1;
        }
    }

    /// Scrolls the log view down by one line.
    pub const fn scroll_down(&mut self, max_lines: usize, visible_lines: usize) {
        if max_lines > visible_lines && self.scroll_offset < max_lines - visible_lines {
            self.scroll_offset += 1;
        }
    }

    /// Jumps to the top of the log.
    pub const fn scroll_to_top(&mut self) {
        self.scroll_offset = 0;
    }

    /// Jumps to the bottom of the log.
    pub const fn scroll_to_bottom(&mut self, max_lines: usize, visible_lines: usize) {
        if max_lines > visible_lines {
            self.scroll_offset = max_lines - visible_lines;
        } else {
            self.scroll_offset = 0;
        }
    }
}

impl Default for BuildLogState {
    fn default() -> Self {
        Self::new()
    }
}

use super::Theme;

/// BuildLog component that implements the ContextualComponent trait
#[derive(Debug, Clone)]
pub struct BuildLog {
    /// Internal state of the build log
    state: BuildLogState,
}

impl BuildLog {
    /// Create a new BuildLog
    #[must_use]
    pub const fn new() -> Self {
        Self {
            state: BuildLogState::new(),
        }
    }


}

impl Default for BuildLog {
    fn default() -> Self {
        Self::new()
    }
}

impl crate::tui::component::ContextualComponent for BuildLog {
    type Context = BuildState;
    type Event = BuildLogEvent;

    fn handle_input(
        &mut self,
        key: crossterm::event::KeyEvent,
        context: &Self::Context,
    ) -> Option<Self::Event> {
        use crossterm::event::KeyCode;

        match key.code {
            KeyCode::Esc | KeyCode::Char('B') => Some(BuildLogEvent::Closed),
            KeyCode::Up | KeyCode::Char('k') => {
                self.state.scroll_up();
                None
            }
            KeyCode::Down | KeyCode::Char('j') => {
                let max_lines = context.log_lines.len();
                self.state.scroll_down(max_lines, 20); // Approximate visible lines
                None
            }
            KeyCode::Home | KeyCode::Char('g') => {
                self.state.scroll_to_top();
                None
            }
            KeyCode::End | KeyCode::Char('G') => {
                let max_lines = context.log_lines.len();
                self.state.scroll_to_bottom(max_lines, 20); // Approximate visible lines
                None
            }
            _ => None,
        }
    }

    fn render(&self, f: &mut Frame, _area: Rect, theme: &Theme, context: &Self::Context) {
        render_build_log_component(f, self, theme, context);
    }
}

/// Renders the build log viewer overlay (for Component)
fn render_build_log_component(
    f: &mut Frame,
    log: &BuildLog,
    theme: &Theme,
    build_state: &BuildState,
) {
    let log_state = &log.state;

    // Calculate centered area (80% width, 60% height)
    let area = centered_rect(80, 60, f.area());

    // Clear the background area first
    f.render_widget(Clear, area);

    // Render opaque background
    let background = Block::default().style(Style::default().bg(theme.background));
    f.render_widget(background, area);

    // Get log lines
    let log_lines = &build_state.log_lines;
    let total_lines = log_lines.len();

    // Calculate visible area height (subtract borders and title)
    let visible_lines = (area.height.saturating_sub(2)) as usize;

    // Apply scroll offset
    let start_idx = log_state.scroll_offset.min(total_lines.saturating_sub(1));
    let end_idx = (start_idx + visible_lines).min(total_lines);

    // Create list items with colored text based on log level
    let items: Vec<ListItem> = log_lines[start_idx..end_idx]
        .iter()
        .map(|(level, message)| {
            let color = match level {
                crate::firmware::builder::LogLevel::Info => theme.text,
                crate::firmware::builder::LogLevel::Ok => theme.success,
                crate::firmware::builder::LogLevel::Error => theme.error,
            };

            ListItem::new(Line::from(Span::styled(
                message.clone(),
                Style::default().fg(color),
            )))
        })
        .collect();

    // Build status in title
    let title = format!(
        " Build Log - {} ({}/{} lines) ",
        build_state.status,
        start_idx + 1,
        total_lines
    );

    let list = List::new(items).block(
        Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.primary)),
    );

    f.render_widget(list, area);

    // Render help text at bottom
    let help_text = "↑↓: Scroll | Home/End: Jump | Ctrl+C: Copy | Esc/Shift+B: Close";
    let help_area = Rect {
        x: area.x + 2,
        y: area.y + area.height - 1,
        width: area.width.saturating_sub(4),
        height: 1,
    };

    let help = Paragraph::new(help_text).style(
        Style::default()
            .fg(theme.text_muted)
            .add_modifier(Modifier::DIM),
    );

    f.render_widget(help, help_area);
}

/// Helper to create a centered rectangle.
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    use ratatui::layout::{Constraint, Direction, Layout};

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
