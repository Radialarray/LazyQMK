//! Help overlay widget showing all keyboard shortcuts organized by category.
//!
//! This module provides a scrollable help overlay accessible via '?' key
//! that documents all keyboard shortcuts and features. Content is loaded
//! from the centralized help registry.

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, Borders, Clear, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap,
    },
    Frame,
};

use super::help_registry::{contexts, HelpRegistry};
use super::Theme;

/// Events emitted by the HelpOverlay component
#[derive(Debug, Clone)]
pub enum HelpOverlayEvent {
    /// User closed the help overlay
    Closed,
}

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

    /// Format a binding's keys for display
    fn format_keys(keys: &[String], alt_keys: &[String]) -> String {
        let primary = keys.join("/");
        if alt_keys.is_empty() {
            primary
        } else {
            format!("{} ({})", primary, alt_keys.join("/"))
        }
    }

    /// Add a section header
    fn add_section_header(lines: &mut Vec<Line<'static>>, title: &str, theme: &Theme) {
        lines.push(Line::from(""));
        lines.push(Line::from(vec![Span::styled(
            format!("═══ {title} ═══"),
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        )]));
        lines.push(Line::from(""));
    }

    /// Add a subsection header (for "In X:" labels)
    fn add_subsection_header(lines: &mut Vec<Line<'static>>, title: &str, theme: &Theme) {
        lines.push(Line::from(vec![Span::styled(
            format!("  {title}:"),
            Style::default().fg(theme.text_muted),
        )]));
    }

    /// Add a subsection header using context name from registry
    fn add_context_subsection(
        lines: &mut Vec<Line<'static>>,
        registry: &HelpRegistry,
        context_name: &str,
        fallback_title: &str,
        theme: &Theme,
    ) {
        // Use context.name field if available, otherwise use fallback
        let title = registry
            .get_context(context_name)
            .map_or(fallback_title, |ctx| ctx.name.as_str());
        lines.push(Line::from(vec![Span::styled(
            format!("  In {title}:"),
            Style::default().fg(theme.text_muted),
        )]));
    }

    /// Add bindings from a context
    fn add_context_bindings(
        lines: &mut Vec<Line<'static>>,
        registry: &HelpRegistry,
        context_name: &str,
        theme: &Theme,
        key_style: Style,
    ) {
        let bindings = registry.get_bindings(context_name);
        for binding in bindings {
            // Use format_binding_for_help from registry for consistent formatting
            let (keys, action) = HelpRegistry::format_binding_for_help(binding);
            let padded_keys = format!("{keys:<18}");
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(padded_keys, key_style),
                Span::styled(action, Style::default().fg(theme.text)),
            ]));
        }
    }

    /// Get the comprehensive help content organized by feature.
    #[allow(clippy::too_many_lines, clippy::cognitive_complexity)]
    fn get_help_content(theme: &Theme) -> Vec<Line<'static>> {
        let registry = HelpRegistry::default();
        let mut lines = Vec::new();

        let key_style = Style::default().fg(theme.success);
        let info_style = Style::default().fg(theme.warning);

        // Use app_name from registry metadata for dynamic header
        let app_name = registry.app_name();
        let header_text = format!("{app_name} - Help");
        let padded_header = format!("{header_text:^65}");

        // Header
        lines.push(Line::from(vec![Span::styled(
            "═══════════════════════════════════════════════════════════════",
            Style::default().fg(theme.primary),
        )]));
        lines.push(Line::from(vec![Span::styled(
            padded_header,
            Style::default()
                .fg(theme.primary)
                .add_modifier(Modifier::BOLD),
        )]));
        lines.push(Line::from(vec![Span::styled(
            "═══════════════════════════════════════════════════════════════",
            Style::default().fg(theme.primary),
        )]));

        // =====================================================================
        // NAVIGATION - subset of main bindings
        // =====================================================================
        Self::add_section_header(&mut lines, "NAVIGATION", theme);

        if let Some(ctx) = registry.get_context(contexts::MAIN) {
            for binding in &ctx.bindings {
                // Only show navigation-related bindings
                if binding.action.contains("Navigate") || binding.action.contains("Jump") {
                    let keys = Self::format_keys(&binding.keys, &binding.alt_keys);
                    let padded_keys = format!("{keys:<18}");
                    lines.push(Line::from(vec![
                        Span::raw("  "),
                        Span::styled(padded_keys, key_style),
                        Span::styled(binding.action.clone(), Style::default().fg(theme.text)),
                    ]));
                }
            }
        }

        // =====================================================================
        // LAYERS
        // =====================================================================
        Self::add_section_header(&mut lines, "LAYERS", theme);

        // Main layer shortcuts
        if let Some(ctx) = registry.get_context(contexts::MAIN) {
            for binding in &ctx.bindings {
                if binding.action.contains("layer") || binding.action.contains("Layer") {
                    let keys = Self::format_keys(&binding.keys, &binding.alt_keys);
                    let padded_keys = format!("{keys:<18}");
                    lines.push(Line::from(vec![
                        Span::raw("  "),
                        Span::styled(padded_keys, key_style),
                        Span::styled(binding.action.clone(), Style::default().fg(theme.text)),
                    ]));
                }
            }
        }

        lines.push(Line::from(""));
        Self::add_context_subsection(
            &mut lines,
            &registry,
            contexts::LAYER_MANAGER,
            "layer manager",
            theme,
        );
        Self::add_context_bindings(
            &mut lines,
            &registry,
            contexts::LAYER_MANAGER,
            theme,
            key_style,
        );

        // =====================================================================
        // KEY EDITOR
        // =====================================================================
        Self::add_section_header(&mut lines, "KEY EDITOR", theme);

        // Main key editing shortcuts
        if let Some(ctx) = registry.get_context(contexts::MAIN) {
            for binding in &ctx.bindings {
                if binding.action.contains("keycode") || binding.action.contains("Clear key") {
                    let keys = Self::format_keys(&binding.keys, &binding.alt_keys);
                    let padded_keys = format!("{keys:<18}");
                    lines.push(Line::from(vec![
                        Span::raw("  "),
                        Span::styled(padded_keys, key_style),
                        Span::styled(binding.action.clone(), Style::default().fg(theme.text)),
                    ]));
                }
            }
        }

        lines.push(Line::from(""));
        Self::add_context_subsection(
            &mut lines,
            &registry,
            contexts::KEYCODE_PICKER,
            "keycode picker",
            theme,
        );
        Self::add_context_bindings(
            &mut lines,
            &registry,
            contexts::KEYCODE_PICKER,
            theme,
            key_style,
        );

        lines.push(Line::from(""));
        Self::add_context_subsection(
            &mut lines,
            &registry,
            contexts::PARAMETERIZED_KEYCODES,
            "parameterized keycodes",
            theme,
        );
        Self::add_context_bindings(
            &mut lines,
            &registry,
            contexts::PARAMETERIZED_KEYCODES,
            theme,
            info_style,
        );

        lines.push(Line::from(""));
        Self::add_context_subsection(
            &mut lines,
            &registry,
            contexts::MODIFIER_PICKER,
            "modifier picker",
            theme,
        );
        Self::add_context_bindings(
            &mut lines,
            &registry,
            contexts::MODIFIER_PICKER,
            theme,
            key_style,
        );

        // =====================================================================
        // CLIPBOARD
        // =====================================================================
        Self::add_section_header(&mut lines, "CLIPBOARD", theme);

        lines.push(Line::from(vec![Span::styled(
            "  Single key operations:",
            Style::default().fg(theme.text_muted),
        )]));
        Self::add_context_bindings(&mut lines, &registry, contexts::CLIPBOARD, theme, key_style);

        lines.push(Line::from(""));
        Self::add_subsection_header(&mut lines, "Multi-key selection", theme);
        Self::add_context_bindings(&mut lines, &registry, contexts::SELECTION, theme, key_style);

        lines.push(Line::from(""));
        lines.push(Line::from(vec![Span::styled(
            "  Copy/cut includes keycode, color, category",
            Style::default().fg(theme.text_muted),
        )]));
        lines.push(Line::from(vec![Span::styled(
            "  Multi-paste maintains relative positions",
            Style::default().fg(theme.text_muted),
        )]));

        // =====================================================================
        // COLOR SYSTEM
        // =====================================================================
        Self::add_section_header(&mut lines, "COLOR SYSTEM", theme);

        // Main color shortcuts
        if let Some(ctx) = registry.get_context(contexts::MAIN) {
            for binding in &ctx.bindings {
                if binding.action.contains("color") || binding.action.contains("Color") {
                    let keys = Self::format_keys(&binding.keys, &binding.alt_keys);
                    let padded_keys = format!("{keys:<18}");
                    lines.push(Line::from(vec![
                        Span::raw("  "),
                        Span::styled(padded_keys, key_style),
                        Span::styled(binding.action.clone(), Style::default().fg(theme.text)),
                    ]));
                }
            }
        }

        lines.push(Line::from(""));
        Self::add_subsection_header(&mut lines, "In color picker (Palette mode)", theme);
        Self::add_context_bindings(
            &mut lines,
            &registry,
            contexts::COLOR_PICKER_PALETTE,
            theme,
            key_style,
        );

        lines.push(Line::from(""));
        Self::add_subsection_header(&mut lines, "In color picker (Custom RGB mode)", theme);
        Self::add_context_bindings(
            &mut lines,
            &registry,
            contexts::COLOR_PICKER_RGB,
            theme,
            key_style,
        );

        lines.push(Line::from(""));
        Self::add_subsection_header(&mut lines, "Color priority (highest to lowest)", theme);
        Self::add_context_bindings(
            &mut lines,
            &registry,
            contexts::COLOR_PRIORITY,
            theme,
            info_style,
        );

        // =====================================================================
        // CATEGORY SYSTEM
        // =====================================================================
        Self::add_section_header(&mut lines, "CATEGORY SYSTEM", theme);

        // Main category shortcuts
        if let Some(ctx) = registry.get_context(contexts::MAIN) {
            for binding in &ctx.bindings {
                if binding.action.contains("ategory") {
                    let keys = Self::format_keys(&binding.keys, &binding.alt_keys);
                    let padded_keys = format!("{keys:<18}");
                    lines.push(Line::from(vec![
                        Span::raw("  "),
                        Span::styled(padded_keys, key_style),
                        Span::styled(binding.action.clone(), Style::default().fg(theme.text)),
                    ]));
                }
            }
        }

        lines.push(Line::from(""));
        Self::add_subsection_header(&mut lines, "In category manager", theme);
        Self::add_context_bindings(
            &mut lines,
            &registry,
            contexts::CATEGORY_MANAGER,
            theme,
            key_style,
        );

        // =====================================================================
        // SETTINGS
        // =====================================================================
        Self::add_section_header(&mut lines, "SETTINGS", theme);

        // Main settings shortcut
        if let Some(ctx) = registry.get_context(contexts::MAIN) {
            for binding in &ctx.bindings {
                if binding.action == "Settings" {
                    let keys = Self::format_keys(&binding.keys, &binding.alt_keys);
                    let padded_keys = format!("{keys:<18}");
                    lines.push(Line::from(vec![
                        Span::raw("  "),
                        Span::styled(padded_keys, key_style),
                        Span::styled(
                            "Open settings manager".to_string(),
                            Style::default().fg(theme.text),
                        ),
                    ]));
                }
            }
        }

        lines.push(Line::from(""));
        Self::add_subsection_header(&mut lines, "In settings manager", theme);
        Self::add_context_bindings(
            &mut lines,
            &registry,
            contexts::SETTINGS_MANAGER,
            theme,
            key_style,
        );

        lines.push(Line::from(""));
        Self::add_subsection_header(&mut lines, "Tap-Hold settings (for home-row mods)", theme);
        Self::add_context_bindings(
            &mut lines,
            &registry,
            contexts::TAP_HOLD_INFO,
            theme,
            info_style,
        );

        // =====================================================================
        // TEMPLATES
        // =====================================================================
        Self::add_section_header(&mut lines, "TEMPLATES", theme);

        // Main template shortcuts
        if let Some(ctx) = registry.get_context(contexts::MAIN) {
            for binding in &ctx.bindings {
                if binding.action.contains("emplate") {
                    let keys = Self::format_keys(&binding.keys, &binding.alt_keys);
                    let padded_keys = format!("{keys:<18}");
                    lines.push(Line::from(vec![
                        Span::raw("  "),
                        Span::styled(padded_keys, key_style),
                        Span::styled(binding.action.clone(), Style::default().fg(theme.text)),
                    ]));
                }
            }
        }

        // =====================================================================
        // FILE OPERATIONS
        // =====================================================================
        Self::add_section_header(&mut lines, "FILE OPERATIONS", theme);

        if let Some(ctx) = registry.get_context(contexts::MAIN) {
            for binding in &ctx.bindings {
                if binding.action.contains("Save")
                    || binding.action.contains("metadata")
                    || binding.action == "Quit"
                {
                    let keys = Self::format_keys(&binding.keys, &binding.alt_keys);
                    let padded_keys = format!("{keys:<18}");
                    lines.push(Line::from(vec![
                        Span::raw("  "),
                        Span::styled(padded_keys, key_style),
                        Span::styled(binding.action.clone(), Style::default().fg(theme.text)),
                    ]));
                }
            }
        }

        // =====================================================================
        // BUILD SYSTEM
        // =====================================================================
        Self::add_section_header(&mut lines, "BUILD SYSTEM", theme);

        if let Some(ctx) = registry.get_context(contexts::MAIN) {
            for binding in &ctx.bindings {
                if binding.action.contains("irmware")
                    || binding.action.contains("build")
                    || binding.action.contains("Generate")
                {
                    let keys = Self::format_keys(&binding.keys, &binding.alt_keys);
                    let padded_keys = format!("{keys:<18}");
                    lines.push(Line::from(vec![
                        Span::raw("  "),
                        Span::styled(padded_keys, key_style),
                        Span::styled(binding.action.clone(), Style::default().fg(theme.text)),
                    ]));
                }
            }
        }

        lines.push(Line::from(""));
        Self::add_subsection_header(&mut lines, "In build log", theme);
        Self::add_context_bindings(&mut lines, &registry, contexts::BUILD_LOG, theme, key_style);

        // =====================================================================
        // CONFIGURATION
        // =====================================================================
        Self::add_section_header(&mut lines, "CONFIGURATION", theme);

        if let Some(ctx) = registry.get_context(contexts::MAIN) {
            for binding in &ctx.bindings {
                if binding.action.contains("wizard") || binding.action.contains("variant") {
                    let keys = Self::format_keys(&binding.keys, &binding.alt_keys);
                    let padded_keys = format!("{keys:<18}");
                    lines.push(Line::from(vec![
                        Span::raw("  "),
                        Span::styled(padded_keys, key_style),
                        Span::styled(binding.action.clone(), Style::default().fg(theme.text)),
                    ]));
                }
            }
        }

        // =====================================================================
        // GENERAL
        // =====================================================================
        Self::add_section_header(&mut lines, "GENERAL", theme);

        if let Some(ctx) = registry.get_context(contexts::MAIN) {
            for binding in &ctx.bindings {
                if binding.action.contains("help")
                    || binding.action.contains("Help")
                    || binding.action.contains("Cancel")
                {
                    let keys = Self::format_keys(&binding.keys, &binding.alt_keys);
                    let padded_keys = format!("{keys:<18}");
                    lines.push(Line::from(vec![
                        Span::raw("  "),
                        Span::styled(padded_keys, key_style),
                        Span::styled(binding.action.clone(), Style::default().fg(theme.text)),
                    ]));
                }
            }
        }

        // =====================================================================
        // TIPS
        // =====================================================================
        Self::add_section_header(&mut lines, "TIPS", theme);

        if let Some(ctx) = registry.get_context(contexts::TIPS) {
            for binding in &ctx.bindings {
                lines.push(Line::from(vec![Span::styled(
                    format!("  • {}", binding.action),
                    Style::default().fg(theme.text),
                )]));
            }
        }

        // Footer - show version and context count from registry
        let version = registry.version();
        let context_count = registry.get_context_info().len();

        lines.push(Line::from(""));
        lines.push(Line::from(vec![Span::styled(
            "═══════════════════════════════════════════════════════════════",
            Style::default().fg(theme.primary),
        )]));
        lines.push(Line::from(vec![Span::styled(
            format!("  Help format v{version} • {context_count} contexts"),
            Style::default().fg(theme.text_muted),
        )]));
        lines.push(Line::from(vec![Span::styled(
            "              Press '?' to close • ↑↓ to scroll               ",
            Style::default().fg(theme.text_muted),
        )]));
        lines.push(Line::from(vec![Span::styled(
            "═══════════════════════════════════════════════════════════════",
            Style::default().fg(theme.primary),
        )]));

        lines
    }

    /// Render the help overlay as a centered modal (legacy - for backward compatibility during migration).
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
        let background = Block::default().style(Style::default().bg(theme.background));
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

/// HelpOverlay component that implements the Component trait
#[derive(Debug, Clone)]
pub struct HelpOverlay {
    /// Internal state of the help overlay
    state: HelpOverlayState,
}

impl HelpOverlay {
    /// Create a new HelpOverlay
    #[must_use]
    pub fn new() -> Self {
        Self {
            state: HelpOverlayState::new(),
        }
    }

}

impl Default for HelpOverlay {
    fn default() -> Self {
        Self::new()
    }
}

impl crate::tui::component::Component for HelpOverlay {
    type Event = HelpOverlayEvent;

    fn handle_input(&mut self, key: crossterm::event::KeyEvent) -> Option<Self::Event> {
        use crossterm::event::KeyCode;

        match key.code {
            KeyCode::Char('?') | KeyCode::Esc | KeyCode::Char('q') => {
                Some(HelpOverlayEvent::Closed)
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.state.scroll_up();
                None
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.state.scroll_down();
                None
            }
            KeyCode::PageUp => {
                self.state.page_up(10);
                None
            }
            KeyCode::PageDown => {
                self.state.page_down(10);
                None
            }
            KeyCode::Home | KeyCode::Char('g') => {
                self.state.scroll_to_top();
                None
            }
            KeyCode::End | KeyCode::Char('G') => {
                self.state.scroll_to_bottom();
                None
            }
            _ => None,
        }
    }

    fn render(&self, f: &mut Frame, area: Rect, theme: &Theme) {
        self.state.render(f, area, theme);
    }
}
