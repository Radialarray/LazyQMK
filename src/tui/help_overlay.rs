//! Help overlay widget showing all keyboard shortcuts organized by category.
//!
//! This module provides a scrollable help overlay accessible via '?' key
//! that documents all keyboard shortcuts and features. Content is loaded
//! from the centralized help registry.
//!
//! ## Scroll Mechanics (Regression Fix)
//!
//! The scroll offset represents the first visible line of content. To prevent
//! scrolling past the content, we must clamp `scroll_offset` to:
//!
//! `max_scroll = total_lines.saturating_sub(visible_height)`
//!
//! This ensures the last line of content aligns with the bottom of the viewport
//! when scrolled to the end. Previous implementation incorrectly used
//! `total_lines - 1` as the max, causing blank space at the bottom.
//!
//! Since `visible_height` is only known at render time (depends on terminal
//! size), we store the raw `scroll_offset` and clamp it during render.

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

/// Minimum modal dimensions to ensure content is always visible
const MIN_MODAL_WIDTH: u16 = 40;
const MIN_MODAL_HEIGHT: u16 = 10;

/// Events emitted by the HelpOverlay component
#[derive(Debug, Clone)]
pub enum HelpOverlayEvent {
    /// User closed the help overlay
    Closed,
}

/// State for the help overlay.
///
/// The scroll offset is stored as a raw value and clamped at render time,
/// since we don't know the viewport size until then.
#[derive(Debug, Clone)]
pub struct HelpOverlayState {
    /// Current scroll offset (first visible line number).
    /// This may temporarily exceed the valid range; it gets clamped during render.
    pub scroll_offset: usize,
}

impl HelpOverlayState {
    /// Creates a new help overlay state.
    #[must_use]
    pub fn new() -> Self {
        Self { scroll_offset: 0 }
    }

    /// Computes the maximum valid scroll offset given content and viewport sizes.
    ///
    /// The maximum scroll position is the point where the last line of content
    /// aligns with the bottom of the viewport:
    ///     max_scroll = total_lines - visible_height
    ///
    /// Returns 0 if content fits within viewport (no scrolling needed).
    #[inline]
    fn compute_max_scroll(total_lines: usize, visible_height: usize) -> usize {
        total_lines.saturating_sub(visible_height)
    }

    /// Returns the clamped scroll offset for the given content and viewport sizes.
    ///
    /// This should be called during render to get the actual offset to use.
    fn clamped_offset(&self, total_lines: usize, visible_height: usize) -> usize {
        let max_scroll = Self::compute_max_scroll(total_lines, visible_height);
        self.scroll_offset.min(max_scroll)
    }

    /// Scroll up by one line.
    pub fn scroll_up(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_sub(1);
    }

    /// Scroll down by one line.
    ///
    /// The offset will be clamped to valid bounds during render.
    pub fn scroll_down(&mut self) {
        // Use saturating_add to prevent overflow; value will be clamped at render
        self.scroll_offset = self.scroll_offset.saturating_add(1);
    }

    /// Scroll to the top.
    pub fn scroll_to_top(&mut self) {
        self.scroll_offset = 0;
    }

    /// Scroll to the bottom.
    ///
    /// Sets to maximum value; will be clamped to actual max during render.
    pub fn scroll_to_bottom(&mut self) {
        self.scroll_offset = usize::MAX;
    }

    /// Scroll down by a page.
    ///
    /// The offset will be clamped to valid bounds during render.
    pub fn page_down(&mut self, visible_height: usize) {
        self.scroll_offset = self.scroll_offset.saturating_add(visible_height);
    }

    /// Scroll up by a page.
    pub fn page_up(&mut self, visible_height: usize) {
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
    pub(super) fn get_help_content(theme: &Theme) -> Vec<Line<'static>> {
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

        // Tap dance editor
        if let Some(ctx) = registry.get_context(contexts::MAIN) {
            for binding in &ctx.bindings {
                if binding.action.contains("Tap dance editor") {
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
        lines.push(Line::from(vec![Span::styled(
            "  Selection entry points:",
            Style::default().fg(theme.text_muted),
        )]));
        if let Some(ctx) = registry.get_context(contexts::MAIN) {
            for binding in &ctx.bindings {
                if binding.action == "Selection mode"
                    || binding.action == "Rectangle select"
                    || binding.action.contains("Swap two keys")
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
        lines.push(Line::from(vec![Span::styled(
            "  Note: When selection mode is active, color is applied to all selected keys.",
            Style::default().fg(theme.text_muted),
        )]));
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
        lines.push(Line::from(vec![Span::styled(
            "  Note: When selection mode is active, category is applied to all selected keys.",
            Style::default().fg(theme.text_muted),
        )]));
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
        // Calculate centered modal size (60% width, 80% height) with minimum dimensions
        // to ensure content is always visible even in tiny terminals
        let width = ((area.width * 60) / 100)
            .max(MIN_MODAL_WIDTH)
            .min(area.width);
        let height = ((area.height * 80) / 100)
            .max(MIN_MODAL_HEIGHT)
            .min(area.height);
        let x = (area.width.saturating_sub(width)) / 2;
        let y = (area.height.saturating_sub(height)) / 2;

        let modal_area = Rect {
            x: x + area.x,
            y: y + area.y,
            width,
            height,
        };

        // Clear the background area first
        f.render_widget(Clear, modal_area);

        // Render opaque background
        let background = Block::default().style(Style::default().bg(theme.background));
        f.render_widget(background, modal_area);

        // Create layout for content area and scrollbar
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(1), Constraint::Length(1)])
            .split(modal_area);

        let content_area = chunks[0];
        let scrollbar_area = chunks[1];

        // Get help content (fresh each render to ensure correct theme styling)
        let content = HelpOverlayState::get_help_content(theme);
        let total_lines = content.len();

        // Calculate visible height (content area minus borders)
        // Ensure at least 1 to avoid division issues
        let visible_height = (content_area.height.saturating_sub(2) as usize).max(1);

        // Clamp scroll offset to valid bounds:
        // max_scroll = total_lines - visible_height (or 0 if content fits)
        let clamped_offset = self.state.clamped_offset(total_lines, visible_height);

        // Create paragraph with clamped scroll position
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
            .scroll((clamped_offset as u16, 0));

        f.render_widget(paragraph, content_area);

        // Render scrollbar with correct max value (total scrollable range)
        let max_scroll = HelpOverlayState::compute_max_scroll(total_lines, visible_height);
        let scrollbar = Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓"))
            .track_symbol(Some("│"))
            .thumb_symbol("█")
            .style(Style::default().fg(theme.primary));

        // ScrollbarState takes the max value (not total) and current position
        let mut scrollbar_state = ScrollbarState::new(max_scroll).position(clamped_offset);

        f.render_stateful_widget(scrollbar, scrollbar_area, &mut scrollbar_state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    /// Helper to create a test terminal with given dimensions
    fn create_test_terminal(width: u16, height: u16) -> Terminal<TestBackend> {
        let backend = TestBackend::new(width, height);
        Terminal::new(backend).expect("Failed to create test terminal")
    }

    // =========================================================================
    // HelpOverlayState unit tests
    // =========================================================================

    #[test]
    fn test_help_overlay_state_initial() {
        let state = HelpOverlayState::new();
        assert_eq!(state.scroll_offset, 0, "Initial scroll should be at top");
    }

    #[test]
    fn test_scroll_up_from_zero_stays_zero() {
        let mut state = HelpOverlayState::new();
        state.scroll_up();
        assert_eq!(state.scroll_offset, 0, "Scroll up from 0 should stay at 0");
    }

    #[test]
    fn test_scroll_down_increments() {
        let mut state = HelpOverlayState::new();
        state.scroll_down();
        assert_eq!(
            state.scroll_offset, 1,
            "Scroll down should increment offset"
        );
        state.scroll_down();
        assert_eq!(state.scroll_offset, 2, "Scroll down again should increment");
    }

    #[test]
    fn test_scroll_to_top_resets() {
        let mut state = HelpOverlayState::new();
        state.scroll_offset = 50;
        state.scroll_to_top();
        assert_eq!(state.scroll_offset, 0, "Scroll to top should reset to 0");
    }

    #[test]
    fn test_scroll_to_bottom_sets_max() {
        let mut state = HelpOverlayState::new();
        state.scroll_to_bottom();
        assert_eq!(
            state.scroll_offset,
            usize::MAX,
            "Scroll to bottom should set to MAX"
        );
    }

    #[test]
    fn test_page_up_page_down() {
        let mut state = HelpOverlayState::new();
        state.scroll_offset = 50;

        state.page_down(10);
        assert_eq!(state.scroll_offset, 60, "Page down by 10 should add 10");

        state.page_up(15);
        assert_eq!(state.scroll_offset, 45, "Page up by 15 should subtract 15");

        state.page_up(100);
        assert_eq!(state.scroll_offset, 0, "Page up beyond 0 should clamp to 0");
    }

    #[test]
    fn test_compute_max_scroll() {
        // Content smaller than viewport: no scrolling possible
        assert_eq!(
            HelpOverlayState::compute_max_scroll(10, 20),
            0,
            "Content smaller than viewport should have max_scroll=0"
        );

        // Content equals viewport: no scrolling possible
        assert_eq!(
            HelpOverlayState::compute_max_scroll(20, 20),
            0,
            "Content equals viewport should have max_scroll=0"
        );

        // Content larger than viewport
        assert_eq!(
            HelpOverlayState::compute_max_scroll(100, 20),
            80,
            "100 lines with 20 visible should have max_scroll=80"
        );
    }

    #[test]
    fn test_clamped_offset_within_bounds() {
        let mut state = HelpOverlayState::new();
        state.scroll_offset = 30;

        // 100 total lines, 20 visible => max_scroll = 80
        let clamped = state.clamped_offset(100, 20);
        assert_eq!(
            clamped, 30,
            "Offset 30 is within bounds, should be unchanged"
        );
    }

    #[test]
    fn test_clamped_offset_exceeds_max() {
        let mut state = HelpOverlayState::new();
        state.scroll_offset = 95;

        // 100 total lines, 20 visible => max_scroll = 80
        let clamped = state.clamped_offset(100, 20);
        assert_eq!(clamped, 80, "Offset 95 should clamp to max_scroll=80");
    }

    #[test]
    fn test_clamped_offset_at_usize_max() {
        let mut state = HelpOverlayState::new();
        state.scroll_to_bottom(); // Sets to usize::MAX

        // 100 total lines, 20 visible => max_scroll = 80
        let clamped = state.clamped_offset(100, 20);
        assert_eq!(clamped, 80, "usize::MAX should clamp to max_scroll=80");
    }

    #[test]
    fn test_clamped_offset_content_fits_viewport() {
        let mut state = HelpOverlayState::new();
        state.scroll_offset = 10;

        // 15 total lines, 20 visible => max_scroll = 0 (content fits)
        let clamped = state.clamped_offset(15, 20);
        assert_eq!(clamped, 0, "When content fits viewport, offset should be 0");
    }

    // =========================================================================
    // HelpOverlay Component tests
    // =========================================================================

    #[test]
    fn test_help_overlay_new() {
        let overlay = HelpOverlay::new();
        assert_eq!(overlay.state.scroll_offset, 0);
    }

    #[test]
    fn test_help_overlay_default() {
        let overlay = HelpOverlay::default();
        assert_eq!(overlay.state.scroll_offset, 0);
    }

    // =========================================================================
    // Rendering tests using TestBackend
    // =========================================================================

    #[test]
    fn test_render_normal_terminal_shows_content() {
        // Test that rendering on a normal-sized terminal shows content
        let mut terminal = create_test_terminal(80, 40);
        let overlay = HelpOverlay::new();
        let theme = Theme::default();

        terminal
            .draw(|frame| {
                let area = frame.area();
                use crate::tui::component::Component;
                overlay.render(frame, area, &theme);
            })
            .expect("Failed to render");

        // Get the buffer and check that title is present
        let buffer = terminal.backend().buffer();
        let content = buffer_to_string(buffer);

        assert!(
            content.contains("Help"),
            "Rendered content should contain 'Help' title. Got:\n{}",
            content
        );

        // Should contain at least one help entry (NAVIGATION section exists)
        assert!(
            content.contains("NAVIGATION") || content.contains("Navigate"),
            "Rendered content should contain help entries"
        );
    }

    #[test]
    fn test_render_tiny_terminal_shows_something() {
        // Test that even a very small terminal doesn't panic and shows something
        let mut terminal = create_test_terminal(20, 10);
        let overlay = HelpOverlay::new();
        let theme = Theme::default();

        // This should not panic
        terminal
            .draw(|frame| {
                let area = frame.area();
                use crate::tui::component::Component;
                overlay.render(frame, area, &theme);
            })
            .expect("Failed to render on tiny terminal");

        // Get the buffer
        let buffer = terminal.backend().buffer();
        let content = buffer_to_string(buffer);

        // Should still show border or title fragment
        assert!(
            !content.trim().is_empty(),
            "Even tiny terminal should render something"
        );
    }

    #[test]
    fn test_render_excessive_scroll_clamped() {
        // Test that excessive scroll offset is clamped and doesn't produce blank output
        let mut terminal = create_test_terminal(80, 40);
        let mut overlay = HelpOverlay::new();
        let theme = Theme::default();

        // Set scroll to an excessive value
        overlay.state.scroll_offset = 10000;

        terminal
            .draw(|frame| {
                let area = frame.area();
                use crate::tui::component::Component;
                overlay.render(frame, area, &theme);
            })
            .expect("Failed to render with excessive scroll");

        let buffer = terminal.backend().buffer();
        let content = buffer_to_string(buffer);

        // Should still have visible content (the footer at minimum)
        assert!(
            content.contains("Help") || content.contains("═"),
            "Excessive scroll should be clamped and still show content. Got:\n{}",
            content
        );
    }

    #[test]
    fn test_render_at_scroll_bottom_shows_footer() {
        // Test that scrolling to bottom shows the footer
        let mut terminal = create_test_terminal(80, 40);
        let mut overlay = HelpOverlay::new();
        let theme = Theme::default();

        overlay.state.scroll_to_bottom();

        terminal
            .draw(|frame| {
                let area = frame.area();
                use crate::tui::component::Component;
                overlay.render(frame, area, &theme);
            })
            .expect("Failed to render at scroll bottom");

        let buffer = terminal.backend().buffer();
        let content = buffer_to_string(buffer);

        // Footer contains "Press '?' to close" text
        assert!(
            content.contains("close") || content.contains("scroll") || content.contains("═"),
            "Scroll to bottom should show footer content. Got:\n{}",
            content
        );
    }

    #[test]
    fn test_render_zero_height_terminal() {
        // Edge case: terminal with 0 height should not panic
        let mut terminal = create_test_terminal(80, 0);
        let overlay = HelpOverlay::new();
        let theme = Theme::default();

        // This should not panic
        let result = terminal.draw(|frame| {
            let area = frame.area();
            use crate::tui::component::Component;
            overlay.render(frame, area, &theme);
        });

        assert!(
            result.is_ok(),
            "Rendering on 0-height terminal should not panic"
        );
    }

    // =========================================================================
    // Input handling tests
    // =========================================================================

    #[test]
    fn test_handle_input_close_keys() {
        use crate::tui::component::Component;
        use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

        let mut overlay = HelpOverlay::new();

        // '?' should close
        let event = overlay.handle_input(KeyEvent::new(KeyCode::Char('?'), KeyModifiers::NONE));
        assert!(matches!(event, Some(HelpOverlayEvent::Closed)));

        // Esc should close
        let event = overlay.handle_input(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
        assert!(matches!(event, Some(HelpOverlayEvent::Closed)));

        // 'q' should close
        let event = overlay.handle_input(KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE));
        assert!(matches!(event, Some(HelpOverlayEvent::Closed)));
    }

    #[test]
    fn test_handle_input_scroll_keys() {
        use crate::tui::component::Component;
        use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

        let mut overlay = HelpOverlay::new();

        // Down arrow scrolls down
        let event = overlay.handle_input(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
        assert!(event.is_none());
        assert_eq!(overlay.state.scroll_offset, 1);

        // 'j' also scrolls down
        let event = overlay.handle_input(KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE));
        assert!(event.is_none());
        assert_eq!(overlay.state.scroll_offset, 2);

        // Up arrow scrolls up
        let event = overlay.handle_input(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE));
        assert!(event.is_none());
        assert_eq!(overlay.state.scroll_offset, 1);

        // 'k' also scrolls up
        let event = overlay.handle_input(KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE));
        assert!(event.is_none());
        assert_eq!(overlay.state.scroll_offset, 0);
    }

    #[test]
    fn test_handle_input_home_end() {
        use crate::tui::component::Component;
        use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

        let mut overlay = HelpOverlay::new();
        overlay.state.scroll_offset = 50;

        // Home goes to top
        let event = overlay.handle_input(KeyEvent::new(KeyCode::Home, KeyModifiers::NONE));
        assert!(event.is_none());
        assert_eq!(overlay.state.scroll_offset, 0);

        // End goes to bottom (sets to MAX, will be clamped at render)
        let event = overlay.handle_input(KeyEvent::new(KeyCode::End, KeyModifiers::NONE));
        assert!(event.is_none());
        assert_eq!(overlay.state.scroll_offset, usize::MAX);
    }

    // =========================================================================
    // Content generation tests
    // =========================================================================

    #[test]
    fn test_get_help_content_not_empty() {
        let theme = Theme::default();
        let content = HelpOverlayState::get_help_content(&theme);

        assert!(!content.is_empty(), "Help content should not be empty");
        assert!(
            content.len() > 10,
            "Help content should have multiple lines"
        );
    }

    #[test]
    fn test_get_help_content_has_sections() {
        let theme = Theme::default();
        let content = HelpOverlayState::get_help_content(&theme);

        // Convert to string for easier checking
        let text: String = content
            .iter()
            .map(|line| {
                line.spans
                    .iter()
                    .map(|span| span.content.as_ref())
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n");

        assert!(
            text.contains("NAVIGATION"),
            "Should have NAVIGATION section"
        );
        assert!(text.contains("LAYERS"), "Should have LAYERS section");
        assert!(
            text.contains("KEY EDITOR"),
            "Should have KEY EDITOR section"
        );
    }

    // =========================================================================
    // Helper functions
    // =========================================================================

    /// Convert a buffer to a single string for content checking
    fn buffer_to_string(buffer: &ratatui::buffer::Buffer) -> String {
        let mut result = String::new();
        for y in 0..buffer.area.height {
            for x in 0..buffer.area.width {
                let cell = &buffer[(x, y)];
                result.push_str(cell.symbol());
            }
            result.push('\n');
        }
        result
    }
}
