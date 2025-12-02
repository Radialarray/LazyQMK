//! Help overlay widget showing all keyboard shortcuts organized by category.
//!
//! This module provides a scrollable help overlay accessible via '?' key
//! that documents all keyboard shortcuts and features.

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap},
    Frame,
};

use super::Theme;

/// State for the help overlay.
#[derive(Debug, Clone)]
pub struct HelpOverlayState {
    /// Current scroll offset (line number)
    pub scroll_offset: usize,
    /// Total number of content lines
    total_lines: usize,
}

impl HelpOverlayState {
    /// Creates a new help overlay state.
    #[must_use]
    pub fn new() -> Self {
        // Calculate total lines using default dark theme for initialization
        // (actual rendering will use the current theme)
        let content = Self::get_help_content(&Theme::default());
        let total_lines = content.len();
        Self {
            scroll_offset: 0,
            total_lines,
        }
    }

    /// Scroll up by one line.
    pub const fn scroll_up(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_sub(1);
    }

    /// Scroll down by one line.
    pub const fn scroll_down(&mut self) {
        if self.scroll_offset + 1 < self.total_lines {
            self.scroll_offset += 1;
        }
    }

    /// Scroll to the top.
    pub const fn scroll_to_top(&mut self) {
        self.scroll_offset = 0;
    }

    /// Scroll to the bottom.
    pub const fn scroll_to_bottom(&mut self) {
        self.scroll_offset = self.total_lines.saturating_sub(1);
    }

    /// Scroll down by a page (approximation based on visible height).
    pub fn page_down(&mut self, visible_height: usize) {
        self.scroll_offset =
            (self.scroll_offset + visible_height).min(self.total_lines.saturating_sub(1));
    }

    /// Scroll up by a page (approximation based on visible height).
    pub const fn page_up(&mut self, visible_height: usize) {
        self.scroll_offset = self.scroll_offset.saturating_sub(visible_height);
    }

    /// Get the comprehensive help content organized by feature.
    ///
    /// Each section starts with how to open/toggle the feature,
    /// followed by all related shortcuts grouped together.
    fn get_help_content(theme: &Theme) -> Vec<Line<'static>> {
        vec![
            // Header
            Line::from(vec![
                Span::styled(
                    "═══════════════════════════════════════════════════════════════",
                    Style::default().fg(theme.primary),
                ),
            ]),
            Line::from(vec![
                Span::styled(
                    "                  Keyboard Layout Editor - Help                  ",
                    Style::default().fg(theme.primary).add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled(
                    "═══════════════════════════════════════════════════════════════",
                    Style::default().fg(theme.primary),
                ),
            ]),
            Line::from(""),
            // Navigation Section
            Line::from(vec![
                Span::styled(
                    "═══ NAVIGATION ═══",
                    Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("  ", Style::default().fg(theme.text)),
                Span::styled("Arrow Keys", Style::default().fg(theme.success)),
                Span::styled("          Move cursor between keys", Style::default().fg(theme.text)),
            ]),
            Line::from(vec![
                Span::styled("  ", Style::default().fg(theme.text)),
                Span::styled("h/j/k/l", Style::default().fg(theme.success)),
                Span::styled("             VIM-style navigation", Style::default().fg(theme.text)),
            ]),
            Line::from(vec![
                Span::styled("  ", Style::default().fg(theme.text)),
                Span::styled("Home / End", Style::default().fg(theme.success)),
                Span::styled("          Jump to first / last key", Style::default().fg(theme.text)),
            ]),
            Line::from(""),
            // Layers Section
            Line::from(vec![
                Span::styled(
                    "═══ LAYERS ═══",
                    Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("  ", Style::default().fg(theme.text)),
                Span::styled("Tab", Style::default().fg(theme.success)),
                Span::styled("                 Next layer", Style::default().fg(theme.text)),
            ]),
            Line::from(vec![
                Span::styled("  ", Style::default().fg(theme.text)),
                Span::styled("Shift+Tab", Style::default().fg(theme.success)),
                Span::styled("           Previous layer", Style::default().fg(theme.text)),
            ]),
            Line::from(""),
            // Key Editor Section
            Line::from(vec![
                Span::styled(
                    "═══ KEY EDITOR ═══",
                    Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("  ", Style::default().fg(theme.text)),
                Span::styled("Enter", Style::default().fg(theme.success)),
                Span::styled("               Open keycode picker", Style::default().fg(theme.text)),
            ]),
            Line::from(vec![
                Span::styled("  ", Style::default().fg(theme.text)),
                Span::styled("x / Delete", Style::default().fg(theme.success)),
                Span::styled("          Clear key (set to KC_TRNS)", Style::default().fg(theme.text)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("  In keycode picker:", Style::default().fg(theme.text_muted)),
            ]),
            Line::from(vec![
                Span::styled("  ", Style::default().fg(theme.text)),
                Span::styled("Type", Style::default().fg(theme.success)),
                Span::styled("                Search keycodes", Style::default().fg(theme.text)),
            ]),
            Line::from(vec![
                Span::styled("  ", Style::default().fg(theme.text)),
                Span::styled("1-6", Style::default().fg(theme.success)),
                Span::styled("                 Filter by category", Style::default().fg(theme.text)),
            ]),
            Line::from(vec![
                Span::styled("  ", Style::default().fg(theme.text)),
                Span::styled("Up/Down", Style::default().fg(theme.success)),
                Span::styled("             Navigate list", Style::default().fg(theme.text)),
            ]),
            Line::from(vec![
                Span::styled("  ", Style::default().fg(theme.text)),
                Span::styled("Enter", Style::default().fg(theme.success)),
                Span::styled("               Apply selected keycode", Style::default().fg(theme.text)),
            ]),
            Line::from(vec![
                Span::styled("  ", Style::default().fg(theme.text)),
                Span::styled("Escape", Style::default().fg(theme.success)),
                Span::styled("              Cancel", Style::default().fg(theme.text)),
            ]),
            Line::from(""),
            // Color System Section
            Line::from(vec![
                Span::styled(
                    "═══ COLOR SYSTEM ═══",
                    Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("  ", Style::default().fg(theme.text)),
                Span::styled("c", Style::default().fg(theme.success)),
                Span::styled("                   Set layer default color", Style::default().fg(theme.text)),
            ]),
            Line::from(vec![
                Span::styled("  ", Style::default().fg(theme.text)),
                Span::styled("Shift+C", Style::default().fg(theme.success)),
                Span::styled("             Set individual key color", Style::default().fg(theme.text)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("  In color picker:", Style::default().fg(theme.text_muted)),
            ]),
            Line::from(vec![
                Span::styled("  ", Style::default().fg(theme.text)),
                Span::styled("Tab", Style::default().fg(theme.success)),
                Span::styled("                 Switch R/G/B channel", Style::default().fg(theme.text)),
            ]),
            Line::from(vec![
                Span::styled("  ", Style::default().fg(theme.text)),
                Span::styled("Left/Right", Style::default().fg(theme.success)),
                Span::styled("          Adjust value (±1)", Style::default().fg(theme.text)),
            ]),
            Line::from(vec![
                Span::styled("  ", Style::default().fg(theme.text)),
                Span::styled("Shift+Left/Right", Style::default().fg(theme.success)),
                Span::styled("    Adjust value (±10)", Style::default().fg(theme.text)),
            ]),
            Line::from(vec![
                Span::styled("  ", Style::default().fg(theme.text)),
                Span::styled("Enter", Style::default().fg(theme.success)),
                Span::styled("               Apply color", Style::default().fg(theme.text)),
            ]),
            Line::from(vec![
                Span::styled("  ", Style::default().fg(theme.text)),
                Span::styled("Escape", Style::default().fg(theme.success)),
                Span::styled("              Cancel", Style::default().fg(theme.text)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("  Color priority (highest to lowest):", Style::default().fg(theme.text_muted)),
            ]),
            Line::from(vec![
                Span::styled("  ", Style::default().fg(theme.text)),
                Span::styled("i", Style::default().fg(theme.warning)),
                Span::styled(" Individual key override", Style::default().fg(theme.text)),
            ]),
            Line::from(vec![
                Span::styled("  ", Style::default().fg(theme.text)),
                Span::styled("k", Style::default().fg(theme.primary)),
                Span::styled(" Key category color", Style::default().fg(theme.text)),
            ]),
            Line::from(vec![
                Span::styled("  ", Style::default().fg(theme.text)),
                Span::styled("L", Style::default().fg(theme.primary)),
                Span::styled(" Layer category color", Style::default().fg(theme.text)),
            ]),
            Line::from(vec![
                Span::styled("  ", Style::default().fg(theme.text)),
                Span::styled("d", Style::default().fg(theme.inactive)),
                Span::styled(" Layer default color", Style::default().fg(theme.text)),
            ]),
            Line::from(""),
            // Category System Section
            Line::from(vec![
                Span::styled(
                    "═══ CATEGORY SYSTEM ═══",
                    Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("  ", Style::default().fg(theme.text)),
                Span::styled("Ctrl+T", Style::default().fg(theme.success)),
                Span::styled("              Open category manager", Style::default().fg(theme.text)),
            ]),
            Line::from(vec![
                Span::styled("  ", Style::default().fg(theme.text)),
                Span::styled("Shift+K", Style::default().fg(theme.success)),
                Span::styled("             Assign category to key", Style::default().fg(theme.text)),
            ]),
            Line::from(vec![
                Span::styled("  ", Style::default().fg(theme.text)),
                Span::styled("Shift+L", Style::default().fg(theme.success)),
                Span::styled("             Assign category to layer", Style::default().fg(theme.text)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("  In category manager:", Style::default().fg(theme.text_muted)),
            ]),
            Line::from(vec![
                Span::styled("  ", Style::default().fg(theme.text)),
                Span::styled("n", Style::default().fg(theme.success)),
                Span::styled("                   Create new category", Style::default().fg(theme.text)),
            ]),
            Line::from(vec![
                Span::styled("  ", Style::default().fg(theme.text)),
                Span::styled("r", Style::default().fg(theme.success)),
                Span::styled("                   Rename category", Style::default().fg(theme.text)),
            ]),
            Line::from(vec![
                Span::styled("  ", Style::default().fg(theme.text)),
                Span::styled("c", Style::default().fg(theme.success)),
                Span::styled("                   Change category color", Style::default().fg(theme.text)),
            ]),
            Line::from(vec![
                Span::styled("  ", Style::default().fg(theme.text)),
                Span::styled("d", Style::default().fg(theme.success)),
                Span::styled("                   Delete category", Style::default().fg(theme.text)),
            ]),
            Line::from(vec![
                Span::styled("  ", Style::default().fg(theme.text)),
                Span::styled("Escape", Style::default().fg(theme.success)),
                Span::styled("              Close manager", Style::default().fg(theme.text)),
            ]),
            Line::from(""),
            // Templates Section
            Line::from(vec![
                Span::styled(
                    "═══ TEMPLATES ═══",
                    Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("  ", Style::default().fg(theme.text)),
                Span::styled("t", Style::default().fg(theme.success)),
                Span::styled("                   Browse and load templates", Style::default().fg(theme.text)),
            ]),
            Line::from(vec![
                Span::styled("  ", Style::default().fg(theme.text)),
                Span::styled("Shift+T", Style::default().fg(theme.success)),
                Span::styled("             Save as template", Style::default().fg(theme.text)),
            ]),
            Line::from(""),
            // File Operations Section
            Line::from(vec![
                Span::styled(
                    "═══ FILE OPERATIONS ═══",
                    Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("  ", Style::default().fg(theme.text)),
                Span::styled("Ctrl+S", Style::default().fg(theme.success)),
                Span::styled("              Save layout", Style::default().fg(theme.text)),
            ]),
            Line::from(vec![
                Span::styled("  ", Style::default().fg(theme.text)),
                Span::styled("Ctrl+E", Style::default().fg(theme.success)),
                Span::styled("              Edit metadata (name, description, tags)", Style::default().fg(theme.text)),
            ]),
            Line::from(vec![
                Span::styled("  ", Style::default().fg(theme.text)),
                Span::styled("Ctrl+Q", Style::default().fg(theme.success)),
                Span::styled("              Quit (press twice if unsaved)", Style::default().fg(theme.text)),
            ]),
            Line::from(""),
            // Build System Section
            Line::from(vec![
                Span::styled(
                    "═══ BUILD SYSTEM ═══",
                    Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("  ", Style::default().fg(theme.text)),
                Span::styled("Ctrl+G", Style::default().fg(theme.success)),
                Span::styled("              Generate firmware (keymap.c, vial.json)", Style::default().fg(theme.text)),
            ]),
            Line::from(vec![
                Span::styled("  ", Style::default().fg(theme.text)),
                Span::styled("Ctrl+B", Style::default().fg(theme.success)),
                Span::styled("              Build firmware (runs in background)", Style::default().fg(theme.text)),
            ]),
            Line::from(vec![
                Span::styled("  ", Style::default().fg(theme.text)),
                Span::styled("Ctrl+L", Style::default().fg(theme.success)),
                Span::styled("              View build log", Style::default().fg(theme.text)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("  In build log:", Style::default().fg(theme.text_muted)),
            ]),
            Line::from(vec![
                Span::styled("  ", Style::default().fg(theme.text)),
                Span::styled("Up/Down", Style::default().fg(theme.success)),
                Span::styled("             Scroll log", Style::default().fg(theme.text)),
            ]),
            Line::from(vec![
                Span::styled("  ", Style::default().fg(theme.text)),
                Span::styled("Page Up/Down", Style::default().fg(theme.success)),
                Span::styled("        Scroll by page", Style::default().fg(theme.text)),
            ]),
            Line::from(vec![
                Span::styled("  ", Style::default().fg(theme.text)),
                Span::styled("Home/End", Style::default().fg(theme.success)),
                Span::styled("            Jump to top/bottom", Style::default().fg(theme.text)),
            ]),
            Line::from(vec![
                Span::styled("  ", Style::default().fg(theme.text)),
                Span::styled("Ctrl+C", Style::default().fg(theme.success)),
                Span::styled("              Copy log to clipboard", Style::default().fg(theme.text)),
            ]),
            Line::from(vec![
                Span::styled("  ", Style::default().fg(theme.text)),
                Span::styled("Escape", Style::default().fg(theme.success)),
                Span::styled("              Close log", Style::default().fg(theme.text)),
            ]),
            Line::from(""),
            // Configuration Section
            Line::from(vec![
                Span::styled(
                    "═══ CONFIGURATION ═══",
                    Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("  ", Style::default().fg(theme.text)),
                Span::styled("Ctrl+W", Style::default().fg(theme.success)),
                Span::styled("              Setup wizard", Style::default().fg(theme.text)),
            ]),
            Line::from(vec![
                Span::styled("  ", Style::default().fg(theme.text)),
                Span::styled("Ctrl+P", Style::default().fg(theme.success)),
                Span::styled("              Set QMK firmware path", Style::default().fg(theme.text)),
            ]),
            Line::from(vec![
                Span::styled("  ", Style::default().fg(theme.text)),
                Span::styled("Ctrl+K", Style::default().fg(theme.success)),
                Span::styled("              Select keyboard", Style::default().fg(theme.text)),
            ]),
            Line::from(vec![
                Span::styled("  ", Style::default().fg(theme.text)),
                Span::styled("Ctrl+Y", Style::default().fg(theme.success)),
                Span::styled("              Switch layout variant", Style::default().fg(theme.text)),
            ]),
            Line::from(""),
            // System Section
            Line::from(vec![
                Span::styled(
                    "═══ GENERAL ═══",
                    Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("  ", Style::default().fg(theme.text)),
                Span::styled("?", Style::default().fg(theme.success)),
                Span::styled("                   Toggle this help", Style::default().fg(theme.text)),
            ]),
            Line::from(vec![
                Span::styled("  ", Style::default().fg(theme.text)),
                Span::styled("Escape", Style::default().fg(theme.success)),
                Span::styled("              Close any dialog/popup", Style::default().fg(theme.text)),
            ]),
            Line::from(""),
            // Tips Section
            Line::from(vec![
                Span::styled(
                    "═══ TIPS ═══",
                    Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("  • Asterisk (*) in title = unsaved changes", Style::default().fg(theme.text)),
            ]),
            Line::from(vec![
                Span::styled("  • KC_TRNS passes keypress to lower layer", Style::default().fg(theme.text)),
            ]),
            Line::from(vec![
                Span::styled("  • Build runs in background - keep editing!", Style::default().fg(theme.text)),
            ]),
            Line::from(vec![
                Span::styled("  • Templates preserve colors and categories", Style::default().fg(theme.text)),
            ]),
            Line::from(""),
            // Footer
            Line::from(vec![
                Span::styled(
                    "═══════════════════════════════════════════════════════════════",
                    Style::default().fg(theme.primary),
                ),
            ]),
            Line::from(vec![
                Span::styled(
                    "              Press '?' to close • ↑↓ to scroll               ",
                    Style::default().fg(theme.text_muted),
                ),
            ]),
            Line::from(vec![
                Span::styled(
                    "═══════════════════════════════════════════════════════════════",
                    Style::default().fg(theme.primary),
                ),
            ]),
        ]
    }

    /// Render the help overlay as a centered modal.
    pub fn render(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        // Calculate centered modal size (60% width, 80% height)
        let width = (area.width * 60) / 100;
        let height = (area.height * 80) / 100;
        let x = (area.width.saturating_sub(width)) / 2;
        let y = (area.height.saturating_sub(height)) / 2;

        let modal_area = Rect {
            x: x + area.x,
            y: y + area.y,
            width,
            height,
        };

        // Clear the background area first
        frame.render_widget(Clear, modal_area);

        // Render opaque background
        let background = Block::default()
            .style(Style::default().bg(theme.background));
        frame.render_widget(background, modal_area);

        // Create layout for content area and scrollbar
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(1), Constraint::Length(1)])
            .split(modal_area);

        let content_area = chunks[0];
        let scrollbar_area = chunks[1];

        // Get help content
        let content = Self::get_help_content(theme);

        // Create paragraph with scrolling
        let visible_height = content_area.height.saturating_sub(2) as usize; // Account for borders
        let paragraph = Paragraph::new(content)
            .block(
                Block::default()
                    .title(" Help - Keyboard Shortcuts ")
                    .title_alignment(Alignment::Center)
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.primary))
                    .style(Style::default().bg(theme.background)),
            )
            .style(Style::default().fg(theme.text))
            .wrap(Wrap { trim: false })
            .scroll((self.scroll_offset as u16, 0));

        frame.render_widget(paragraph, content_area);

        // Render scrollbar
        let scrollbar = Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓"))
            .track_symbol(Some("│"))
            .thumb_symbol("█")
            .style(Style::default().fg(theme.primary));

        let mut scrollbar_state =
            ScrollbarState::new(self.total_lines.saturating_sub(visible_height))
                .position(self.scroll_offset);

        frame.render_stateful_widget(scrollbar, scrollbar_area, &mut scrollbar_state);
    }
}

impl Default for HelpOverlayState {
    fn default() -> Self {
        Self::new()
    }
}
