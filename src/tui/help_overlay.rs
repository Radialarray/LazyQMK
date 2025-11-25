//! Help overlay widget showing all keyboard shortcuts organized by category.
//!
//! This module provides a scrollable help overlay accessible via '?' key
//! that documents all keyboard shortcuts and features.

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap},
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

    /// Get the comprehensive help content organized by category.
    ///
    /// This includes all shortcuts from US1-US9 features:
    /// - Navigation (US1)
    /// - Editing (US1, US2, US3)
    /// - File Operations (US4)
    /// - Build Operations (US6)
    /// - Configuration (US7)
    /// - Templates (US5)
    /// - System (general)
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
                Span::raw("  "),
                Span::styled("Arrow Keys", Style::default().fg(theme.success)),
                Span::raw("          Move cursor (up/down/left/right)"),
            ]),
            Line::from(vec![
                Span::raw("  "),
                Span::styled("h/j/k/l", Style::default().fg(theme.success)),
                Span::raw("             VIM-style navigation (left/down/up/right)"),
            ]),
            Line::from(vec![
                Span::raw("  "),
                Span::styled("Tab", Style::default().fg(theme.success)),
                Span::raw("                  Switch to next layer"),
            ]),
            Line::from(vec![
                Span::raw("  "),
                Span::styled("Shift+Tab", Style::default().fg(theme.success)),
                Span::raw("            Switch to previous layer"),
            ]),
            Line::from(vec![
                Span::raw("  "),
                Span::styled("Home", Style::default().fg(theme.success)),
                Span::raw("                 Move to first key"),
            ]),
            Line::from(vec![
                Span::raw("  "),
                Span::styled("End", Style::default().fg(theme.success)),
                Span::raw("                  Move to last key"),
            ]),
            Line::from(""),
            // Editing Section
            Line::from(vec![
                Span::styled(
                    "═══ EDITING ═══",
                    Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::raw("  "),
                Span::styled("Enter", Style::default().fg(theme.success)),
                Span::raw("                Open keycode picker for selected key"),
            ]),
            Line::from(vec![
                Span::raw("  "),
                Span::styled("x / Delete", Style::default().fg(theme.success)),
                Span::raw("           Clear selected key (set to KC_TRNS)"),
            ]),
            Line::from(vec![
                Span::raw("  "),
                Span::styled("Shift+C", Style::default().fg(theme.success)),
                Span::raw("              Set individual key color override"),
            ]),
            Line::from(vec![
                Span::raw("  "),
                Span::styled("c", Style::default().fg(theme.success)),
                Span::raw("                    Set layer default color"),
            ]),
            Line::from(vec![
                Span::raw("  "),
                Span::styled("Shift+K", Style::default().fg(theme.success)),
                Span::raw("              Assign category to selected key"),
            ]),
            Line::from(vec![
                Span::raw("  "),
                Span::styled("Ctrl+T", Style::default().fg(theme.success)),
                Span::raw("               Open category manager (create/edit/delete)"),
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
                Span::raw("  "),
                Span::styled("Ctrl+S", Style::default().fg(theme.success)),
                Span::raw("               Save current layout to file"),
            ]),
            Line::from(vec![
                Span::raw("  "),
                Span::styled("Ctrl+Q", Style::default().fg(theme.success)),
                Span::raw("               Quit (prompts if unsaved changes)"),
            ]),
            Line::from(vec![
                Span::raw("  "),
                Span::styled("Ctrl+E", Style::default().fg(theme.success)),
                Span::raw("               Edit layout metadata (name, description, tags)"),
            ]),
            Line::from(""),
            // Build Operations Section
            Line::from(vec![
                Span::styled(
                    "═══ BUILD OPERATIONS ═══",
                    Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::raw("  "),
                Span::styled("Ctrl+G", Style::default().fg(theme.success)),
                Span::raw("               Generate firmware files (keymap.c, vial.json)"),
            ]),
            Line::from(vec![
                Span::raw("  "),
                Span::styled("Ctrl+B", Style::default().fg(theme.success)),
                Span::raw("               Build firmware in background"),
            ]),
            Line::from(vec![
                Span::raw("  "),
                Span::styled("Ctrl+L", Style::default().fg(theme.success)),
                Span::raw("               View build log (scrollable)"),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::raw("  "),
                Span::styled("Note:", Style::default().fg(theme.primary)),
                Span::raw(" Build runs in background - watch status bar for progress"),
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
                Span::raw("  "),
                Span::styled("Ctrl+P", Style::default().fg(theme.success)),
                Span::raw("               Configure QMK firmware path"),
            ]),
            Line::from(vec![
                Span::raw("  "),
                Span::styled("Ctrl+K", Style::default().fg(theme.success)),
                Span::raw("               Select keyboard from QMK repository"),
            ]),
            Line::from(vec![
                Span::raw("  "),
                Span::styled("Ctrl+Y", Style::default().fg(theme.success)),
                Span::raw("               Switch keyboard layout variant"),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::raw("  "),
                Span::styled("Info:", Style::default().fg(theme.primary)),
                Span::raw(" Configuration saved to ~/.config/layout_tools/config.toml"),
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
                Span::raw("  "),
                Span::styled("t", Style::default().fg(theme.success)),
                Span::raw("                    Browse and load templates"),
            ]),
            Line::from(vec![
                Span::raw("  "),
                Span::styled("Shift+T", Style::default().fg(theme.success)),
                Span::raw("              Save current layout as template"),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::raw("  "),
                Span::styled("Info:", Style::default().fg(theme.primary)),
                Span::raw(" Templates stored in ~/.config/layout_tools/templates/"),
            ]),
            Line::from(""),
            // System Section
            Line::from(vec![
                Span::styled(
                    "═══ SYSTEM ═══",
                    Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::raw("  "),
                Span::styled("?", Style::default().fg(theme.success)),
                Span::raw("                    Toggle this help overlay"),
            ]),
            Line::from(vec![
                Span::raw("  "),
                Span::styled("F12", Style::default().fg(theme.success)),
                Span::raw("                  Toggle dark/light theme"),
            ]),
            Line::from(vec![
                Span::raw("  "),
                Span::styled("Escape", Style::default().fg(theme.success)),
                Span::raw("               Close current dialog/popup"),
            ]),
            Line::from(""),
            // Color Indicators Section
            Line::from(vec![
                Span::styled(
                    "═══ COLOR INDICATORS ═══",
                    Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::raw("  Keys display color source indicators in top-right corner:"),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::raw("  "),
                Span::styled("i", Style::default().fg(theme.warning)),
                Span::raw("  Individual color override (highest priority)"),
            ]),
            Line::from(vec![
                Span::raw("  "),
                Span::styled("k", Style::default().fg(theme.primary)),
                Span::raw("  Key category color"),
            ]),
            Line::from(vec![
                Span::raw("  "),
                Span::styled("L", Style::default().fg(theme.primary)),
                Span::raw("  Category layer default"),
            ]),
            Line::from(vec![
                Span::raw("  "),
                Span::styled("d", Style::default().fg(theme.inactive)),
                Span::raw("  Layer default color (lowest priority)"),
            ]),
            Line::from(""),
            // Keycode Picker Section
            Line::from(vec![
                Span::styled(
                    "═══ KEYCODE PICKER ═══",
                    Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::raw("  "),
                Span::styled("Type", Style::default().fg(theme.success)),
                Span::raw("                  Fuzzy search for keycodes"),
            ]),
            Line::from(vec![
                Span::raw("  "),
                Span::styled("1-6", Style::default().fg(theme.success)),
                Span::raw("                  Filter by category (Basic/Nav/Symbols/Function/Media/Modifiers)"),
            ]),
            Line::from(vec![
                Span::raw("  "),
                Span::styled("Arrow Keys", Style::default().fg(theme.success)),
                Span::raw("          Navigate through filtered list"),
            ]),
            Line::from(vec![
                Span::raw("  "),
                Span::styled("Enter", Style::default().fg(theme.success)),
                Span::raw("                Select keycode and apply to key"),
            ]),
            Line::from(vec![
                Span::raw("  "),
                Span::styled("Escape", Style::default().fg(theme.success)),
                Span::raw("               Cancel without changes"),
            ]),
            Line::from(""),
            // Color Picker Section
            Line::from(vec![
                Span::styled(
                    "═══ COLOR PICKER ═══",
                    Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::raw("  "),
                Span::styled("Tab", Style::default().fg(theme.success)),
                Span::raw("                  Switch between R/G/B channels"),
            ]),
            Line::from(vec![
                Span::raw("  "),
                Span::styled("Left/Right", Style::default().fg(theme.success)),
                Span::raw("          Adjust active channel value"),
            ]),
            Line::from(vec![
                Span::raw("  "),
                Span::styled("Shift+Left/Right", Style::default().fg(theme.success)),
                Span::raw("    Adjust by larger increments (±10)"),
            ]),
            Line::from(vec![
                Span::raw("  "),
                Span::styled("Enter", Style::default().fg(theme.success)),
                Span::raw("                Apply color"),
            ]),
            Line::from(vec![
                Span::raw("  "),
                Span::styled("Escape", Style::default().fg(theme.success)),
                Span::raw("               Cancel without changes"),
            ]),
            Line::from(""),
            // Category Manager Section
            Line::from(vec![
                Span::styled(
                    "═══ CATEGORY MANAGER ═══",
                    Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::raw("  "),
                Span::styled("n", Style::default().fg(theme.success)),
                Span::raw("                    Create new category"),
            ]),
            Line::from(vec![
                Span::raw("  "),
                Span::styled("r", Style::default().fg(theme.success)),
                Span::raw("                    Rename selected category"),
            ]),
            Line::from(vec![
                Span::raw("  "),
                Span::styled("c", Style::default().fg(theme.success)),
                Span::raw("                    Change category color"),
            ]),
            Line::from(vec![
                Span::raw("  "),
                Span::styled("d", Style::default().fg(theme.success)),
                Span::raw("                    Delete selected category (with confirmation)"),
            ]),
            Line::from(vec![
                Span::raw("  "),
                Span::styled("Shift+L", Style::default().fg(theme.success)),
                Span::raw("              Assign category to current layer"),
            ]),
            Line::from(vec![
                Span::raw("  "),
                Span::styled("Escape", Style::default().fg(theme.success)),
                Span::raw("               Close category manager"),
            ]),
            Line::from(""),
            // Build Log Section
            Line::from(vec![
                Span::styled(
                    "═══ BUILD LOG VIEWER ═══",
                    Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::raw("  "),
                Span::styled("Arrow Keys", Style::default().fg(theme.success)),
                Span::raw("          Scroll through build log"),
            ]),
            Line::from(vec![
                Span::raw("  "),
                Span::styled("Page Up/Down", Style::default().fg(theme.success)),
                Span::raw("        Scroll by page"),
            ]),
            Line::from(vec![
                Span::raw("  "),
                Span::styled("Home/End", Style::default().fg(theme.success)),
                Span::raw("            Jump to top/bottom"),
            ]),
            Line::from(vec![
                Span::raw("  "),
                Span::styled("Escape", Style::default().fg(theme.success)),
                Span::raw("               Close build log"),
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
                Span::raw("  • Status bar shows context-sensitive help and current mode"),
            ]),
            Line::from(vec![
                Span::raw("  • Asterisk (*) in title bar indicates unsaved changes"),
            ]),
            Line::from(vec![
                Span::raw("  • Four-level color priority: Individual > Key Category > Layer Category > Default"),
            ]),
            Line::from(vec![
                Span::raw("  • Use KC_TRNS (transparent) to pass through to lower layers"),
            ]),
            Line::from(vec![
                Span::raw("  • Build runs in background - you can continue editing"),
            ]),
            Line::from(vec![
                Span::raw("  • Templates preserve all colors, categories, and metadata"),
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
                    "              Press '?' to close help • Press ↑↓ to scroll               ",
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
                    .border_style(Style::default().fg(theme.primary)),
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
