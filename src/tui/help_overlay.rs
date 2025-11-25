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
                Span::styled("  ", Style::default().fg(theme.text)),
                Span::styled("Arrow Keys", Style::default().fg(theme.success)),
                Span::styled("          Move cursor (up/down/left/right)", Style::default().fg(theme.text)),
            ]),
            Line::from(vec![
                Span::styled("  ", Style::default().fg(theme.text)),
                Span::styled("h/j/k/l", Style::default().fg(theme.success)),
                Span::styled("             VIM-style navigation (left/down/up/right)", Style::default().fg(theme.text)),
            ]),
            Line::from(vec![
                Span::styled("  ", Style::default().fg(theme.text)),
                Span::styled("Tab", Style::default().fg(theme.success)),
                Span::styled("                  Switch to next layer", Style::default().fg(theme.text)),
            ]),
            Line::from(vec![
                Span::styled("  ", Style::default().fg(theme.text)),
                Span::styled("Shift+Tab", Style::default().fg(theme.success)),
                Span::styled("            Switch to previous layer", Style::default().fg(theme.text)),
            ]),
            Line::from(vec![
                Span::styled("  ", Style::default().fg(theme.text)),
                Span::styled("Home", Style::default().fg(theme.success)),
                Span::styled("                 Move to first key", Style::default().fg(theme.text)),
            ]),
            Line::from(vec![
                Span::styled("  ", Style::default().fg(theme.text)),
                Span::styled("End", Style::default().fg(theme.success)),
                Span::styled("                  Move to last key", Style::default().fg(theme.text)),
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
                Span::styled("  ", Style::default().fg(theme.text)),
                Span::styled("Enter", Style::default().fg(theme.success)),
                Span::styled("                Open keycode picker for selected key", Style::default().fg(theme.text)),
            ]),
            Line::from(vec![
                Span::styled("  ", Style::default().fg(theme.text)),
                Span::styled("x / Delete", Style::default().fg(theme.success)),
                Span::styled("           Clear selected key (set to KC_TRNS)", Style::default().fg(theme.text)),
            ]),
            Line::from(vec![
                Span::styled("  ", Style::default().fg(theme.text)),
                Span::styled("Shift+C", Style::default().fg(theme.success)),
                Span::styled("              Set individual key color override", Style::default().fg(theme.text)),
            ]),
            Line::from(vec![
                Span::styled("  ", Style::default().fg(theme.text)),
                Span::styled("c", Style::default().fg(theme.success)),
                Span::styled("                    Set layer default color", Style::default().fg(theme.text)),
            ]),
            Line::from(vec![
                Span::styled("  ", Style::default().fg(theme.text)),
                Span::styled("Shift+K", Style::default().fg(theme.success)),
                Span::styled("              Assign category to selected key", Style::default().fg(theme.text)),
            ]),
            Line::from(vec![
                Span::styled("  ", Style::default().fg(theme.text)),
                Span::styled("Ctrl+T", Style::default().fg(theme.success)),
                Span::styled("               Open category manager (create/edit/delete)", Style::default().fg(theme.text)),
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
                Span::styled("               Save current layout to file", Style::default().fg(theme.text)),
            ]),
            Line::from(vec![
                Span::styled("  ", Style::default().fg(theme.text)),
                Span::styled("Ctrl+Q", Style::default().fg(theme.success)),
                Span::styled("               Quit (prompts if unsaved changes)", Style::default().fg(theme.text)),
            ]),
            Line::from(vec![
                Span::styled("  ", Style::default().fg(theme.text)),
                Span::styled("Ctrl+E", Style::default().fg(theme.success)),
                Span::styled("               Edit layout metadata (name, description, tags)", Style::default().fg(theme.text)),
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
                Span::styled("  ", Style::default().fg(theme.text)),
                Span::styled("Ctrl+G", Style::default().fg(theme.success)),
                Span::styled("               Generate firmware files (keymap.c, vial.json)", Style::default().fg(theme.text)),
            ]),
            Line::from(vec![
                Span::styled("  ", Style::default().fg(theme.text)),
                Span::styled("Ctrl+B", Style::default().fg(theme.success)),
                Span::styled("               Build firmware in background", Style::default().fg(theme.text)),
            ]),
            Line::from(vec![
                Span::styled("  ", Style::default().fg(theme.text)),
                Span::styled("Ctrl+L", Style::default().fg(theme.success)),
                Span::styled("               View build log (scrollable)", Style::default().fg(theme.text)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("  ", Style::default().fg(theme.text)),
                Span::styled("Note:", Style::default().fg(theme.primary)),
                Span::styled(" Build runs in background - watch status bar for progress", Style::default().fg(theme.text)),
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
                Span::styled("Ctrl+P", Style::default().fg(theme.success)),
                Span::styled("               Configure QMK firmware path", Style::default().fg(theme.text)),
            ]),
            Line::from(vec![
                Span::styled("  ", Style::default().fg(theme.text)),
                Span::styled("Ctrl+K", Style::default().fg(theme.success)),
                Span::styled("               Select keyboard from QMK repository", Style::default().fg(theme.text)),
            ]),
            Line::from(vec![
                Span::styled("  ", Style::default().fg(theme.text)),
                Span::styled("Ctrl+Y", Style::default().fg(theme.success)),
                Span::styled("               Switch keyboard layout variant", Style::default().fg(theme.text)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("  ", Style::default().fg(theme.text)),
                Span::styled("Info:", Style::default().fg(theme.primary)),
                Span::styled(" Configuration saved to ~/.config/KeyboardConfigurator/config.toml", Style::default().fg(theme.text)),
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
                Span::styled("                    Browse and load templates", Style::default().fg(theme.text)),
            ]),
            Line::from(vec![
                Span::styled("  ", Style::default().fg(theme.text)),
                Span::styled("Shift+T", Style::default().fg(theme.success)),
                Span::styled("              Save current layout as template", Style::default().fg(theme.text)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("  ", Style::default().fg(theme.text)),
                Span::styled("Info:", Style::default().fg(theme.primary)),
                Span::styled(" Templates stored in ~/.config/KeyboardConfigurator/templates/", Style::default().fg(theme.text)),
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
                Span::styled("  ", Style::default().fg(theme.text)),
                Span::styled("?", Style::default().fg(theme.success)),
                Span::styled("                    Toggle this help overlay", Style::default().fg(theme.text)),
            ]),
            Line::from(vec![
                Span::styled("  ", Style::default().fg(theme.text)),
                Span::styled("F12", Style::default().fg(theme.success)),
                Span::styled("                  Toggle dark/light theme", Style::default().fg(theme.text)),
            ]),
            Line::from(vec![
                Span::styled("  ", Style::default().fg(theme.text)),
                Span::styled("Escape", Style::default().fg(theme.success)),
                Span::styled("               Close current dialog/popup", Style::default().fg(theme.text)),
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
                Span::styled("  Keys display color source indicators in top-right corner:", Style::default().fg(theme.text)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("  ", Style::default().fg(theme.text)),
                Span::styled("i", Style::default().fg(theme.warning)),
                Span::styled("  Individual color override (highest priority)", Style::default().fg(theme.text)),
            ]),
            Line::from(vec![
                Span::styled("  ", Style::default().fg(theme.text)),
                Span::styled("k", Style::default().fg(theme.primary)),
                Span::styled("  Key category color", Style::default().fg(theme.text)),
            ]),
            Line::from(vec![
                Span::styled("  ", Style::default().fg(theme.text)),
                Span::styled("L", Style::default().fg(theme.primary)),
                Span::styled("  Category layer default", Style::default().fg(theme.text)),
            ]),
            Line::from(vec![
                Span::styled("  ", Style::default().fg(theme.text)),
                Span::styled("d", Style::default().fg(theme.inactive)),
                Span::styled("  Layer default color (lowest priority)", Style::default().fg(theme.text)),
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
                Span::styled("  ", Style::default().fg(theme.text)),
                Span::styled("Type", Style::default().fg(theme.success)),
                Span::styled("                  Fuzzy search for keycodes", Style::default().fg(theme.text)),
            ]),
            Line::from(vec![
                Span::styled("  ", Style::default().fg(theme.text)),
                Span::styled("1-6", Style::default().fg(theme.success)),
                Span::styled("                  Filter by category (Basic/Nav/Symbols/Function/Media/Modifiers)", Style::default().fg(theme.text)),
            ]),
            Line::from(vec![
                Span::styled("  ", Style::default().fg(theme.text)),
                Span::styled("Arrow Keys", Style::default().fg(theme.success)),
                Span::styled("          Navigate through filtered list", Style::default().fg(theme.text)),
            ]),
            Line::from(vec![
                Span::styled("  ", Style::default().fg(theme.text)),
                Span::styled("Enter", Style::default().fg(theme.success)),
                Span::styled("                Select keycode and apply to key", Style::default().fg(theme.text)),
            ]),
            Line::from(vec![
                Span::styled("  ", Style::default().fg(theme.text)),
                Span::styled("Escape", Style::default().fg(theme.success)),
                Span::styled("               Cancel without changes", Style::default().fg(theme.text)),
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
                Span::styled("  ", Style::default().fg(theme.text)),
                Span::styled("Tab", Style::default().fg(theme.success)),
                Span::styled("                  Switch between R/G/B channels", Style::default().fg(theme.text)),
            ]),
            Line::from(vec![
                Span::styled("  ", Style::default().fg(theme.text)),
                Span::styled("Left/Right", Style::default().fg(theme.success)),
                Span::styled("          Adjust active channel value", Style::default().fg(theme.text)),
            ]),
            Line::from(vec![
                Span::styled("  ", Style::default().fg(theme.text)),
                Span::styled("Shift+Left/Right", Style::default().fg(theme.success)),
                Span::styled("    Adjust by larger increments (±10)", Style::default().fg(theme.text)),
            ]),
            Line::from(vec![
                Span::styled("  ", Style::default().fg(theme.text)),
                Span::styled("Enter", Style::default().fg(theme.success)),
                Span::styled("                Apply color", Style::default().fg(theme.text)),
            ]),
            Line::from(vec![
                Span::styled("  ", Style::default().fg(theme.text)),
                Span::styled("Escape", Style::default().fg(theme.success)),
                Span::styled("               Cancel without changes", Style::default().fg(theme.text)),
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
                Span::styled("  ", Style::default().fg(theme.text)),
                Span::styled("n", Style::default().fg(theme.success)),
                Span::styled("                    Create new category", Style::default().fg(theme.text)),
            ]),
            Line::from(vec![
                Span::styled("  ", Style::default().fg(theme.text)),
                Span::styled("r", Style::default().fg(theme.success)),
                Span::styled("                    Rename selected category", Style::default().fg(theme.text)),
            ]),
            Line::from(vec![
                Span::styled("  ", Style::default().fg(theme.text)),
                Span::styled("c", Style::default().fg(theme.success)),
                Span::styled("                    Change category color", Style::default().fg(theme.text)),
            ]),
            Line::from(vec![
                Span::styled("  ", Style::default().fg(theme.text)),
                Span::styled("d", Style::default().fg(theme.success)),
                Span::styled("                    Delete selected category (with confirmation)", Style::default().fg(theme.text)),
            ]),
            Line::from(vec![
                Span::styled("  ", Style::default().fg(theme.text)),
                Span::styled("Shift+L", Style::default().fg(theme.success)),
                Span::styled("              Assign category to current layer", Style::default().fg(theme.text)),
            ]),
            Line::from(vec![
                Span::styled("  ", Style::default().fg(theme.text)),
                Span::styled("Escape", Style::default().fg(theme.success)),
                Span::styled("               Close category manager", Style::default().fg(theme.text)),
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
                Span::styled("  ", Style::default().fg(theme.text)),
                Span::styled("Arrow Keys", Style::default().fg(theme.success)),
                Span::styled("          Scroll through build log", Style::default().fg(theme.text)),
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
                Span::styled("               Copy entire log to clipboard", Style::default().fg(theme.text)),
            ]),
            Line::from(vec![
                Span::styled("  ", Style::default().fg(theme.text)),
                Span::styled("Escape", Style::default().fg(theme.success)),
                Span::styled("               Close build log", Style::default().fg(theme.text)),
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
                Span::styled("  • Status bar shows context-sensitive help and current mode", Style::default().fg(theme.text)),
            ]),
            Line::from(vec![
                Span::styled("  • Asterisk (*) in title bar indicates unsaved changes", Style::default().fg(theme.text)),
            ]),
            Line::from(vec![
                Span::styled("  • Four-level color priority: Individual > Key Category > Layer Category > Default", Style::default().fg(theme.text)),
            ]),
            Line::from(vec![
                Span::styled("  • Use KC_TRNS (transparent) to pass through to lower layers", Style::default().fg(theme.text)),
            ]),
            Line::from(vec![
                Span::styled("  • Build runs in background - you can continue editing", Style::default().fg(theme.text)),
            ]),
            Line::from(vec![
                Span::styled("  • Templates preserve all colors, categories, and metadata", Style::default().fg(theme.text)),
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
