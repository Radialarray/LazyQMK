//! Color picker dialog for selecting RGB colors.
//!
//! Supports two modes:
//! - Palette mode: Select from curated colors with shades
//! - Custom RGB mode: Fine-tune with RGB sliders

// Input handlers use Result<bool> for consistency even when they never fail
#![allow(clippy::unnecessary_wraps)]
// Allow intentional type casts for color math
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_abs_to_unsigned)]
#![allow(clippy::cast_lossless)]
#![allow(clippy::as_conversions)]

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Gauge, Paragraph},
    Frame,
};

use crate::models::{ColorPalette, RgbColor};

/// RGB channel being edited
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RgbChannel {
    /// Red color channel
    Red,
    /// Green color channel
    Green,
    /// Blue color channel
    Blue,
}

/// Mode of the color picker
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ColorPickerMode {
    /// Selecting from the color palette
    #[default]
    Palette,
    /// Fine-tuning with RGB sliders
    CustomRgb,
}

/// Focus within palette mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PaletteFocus {
    /// Selecting base color
    #[default]
    Colors,
    /// Selecting shade
    Shades,
}

/// State for the color picker dialog
#[derive(Debug, Clone)]
pub struct ColorPickerState {
    /// Current mode (Palette or Custom RGB)
    pub mode: ColorPickerMode,
    /// Red channel value (0-255)
    pub r: u8,
    /// Green channel value (0-255)
    pub g: u8,
    /// Blue channel value (0-255)
    pub b: u8,
    /// Currently active RGB channel for editing
    pub active_channel: RgbChannel,
    /// Selected base color index (0-11)
    pub selected_color: usize,
    /// Selected shade index (0-8)
    pub selected_shade: usize,
    /// Focus within palette mode
    pub palette_focus: PaletteFocus,
    /// The color palette data
    pub palette: ColorPalette,
}

impl ColorPickerState {
    /// Create a new color picker with default white color
    #[must_use]
    pub fn new() -> Self {
        let palette = ColorPalette::default();
        Self {
            mode: ColorPickerMode::Palette,
            r: 255,
            g: 255,
            b: 255,
            active_channel: RgbChannel::Red,
            selected_color: 0,
            selected_shade: 4, // Default to middle shade (500)
            palette_focus: PaletteFocus::Colors,
            palette,
        }
    }

    /// Create a color picker initialized with a specific color
    #[must_use]
    pub fn with_color(color: RgbColor) -> Self {
        let palette = ColorPalette::default();
        let mut state = Self {
            mode: ColorPickerMode::Palette,
            r: color.r,
            g: color.g,
            b: color.b,
            active_channel: RgbChannel::Red,
            selected_color: 0,
            selected_shade: 4,
            palette_focus: PaletteFocus::Colors,
            palette,
        };
        // Try to find matching color in palette
        state.find_closest_palette_color(color);
        state
    }

    /// Find the closest palette color to the given RGB color
    fn find_closest_palette_color(&mut self, target: RgbColor) {
        let mut best_distance = u32::MAX;
        let mut best_color = 0;
        let mut best_shade = 4;

        for (ci, palette_color) in self.palette.colors.iter().enumerate() {
            for (si, shade) in palette_color.shades.iter().enumerate() {
                let dr = (i32::from(shade.r) - i32::from(target.r)).abs() as u32;
                let dg = (i32::from(shade.g) - i32::from(target.g)).abs() as u32;
                let db = (i32::from(shade.b) - i32::from(target.b)).abs() as u32;
                let distance = dr * dr + dg * dg + db * db;

                if distance < best_distance {
                    best_distance = distance;
                    best_color = ci;
                    best_shade = si;
                }
            }
        }

        self.selected_color = best_color;
        self.selected_shade = best_shade;
    }

    /// Get the current color
    #[must_use]
    pub const fn get_color(&self) -> RgbColor {
        RgbColor::new(self.r, self.g, self.b)
    }

    /// Sync RGB values from current palette selection
    pub fn sync_from_palette(&mut self) {
        if let Some(color) = self.palette.color_at(self.selected_color) {
            if let Some(shade) = color.shade_at(self.selected_shade) {
                self.r = shade.r;
                self.g = shade.g;
                self.b = shade.b;
            }
        }
    }

    /// Get the currently selected palette shade
    #[must_use]
    pub fn get_selected_shade(&self) -> Option<&crate::models::Shade> {
        self.palette
            .color_at(self.selected_color)
            .and_then(|c| c.shade_at(self.selected_shade))
    }

    /// Switch to next RGB channel (Red -> Green -> Blue -> Red)
    pub const fn next_channel(&mut self) {
        self.active_channel = match self.active_channel {
            RgbChannel::Red => RgbChannel::Green,
            RgbChannel::Green => RgbChannel::Blue,
            RgbChannel::Blue => RgbChannel::Red,
        };
    }

    /// Switch to previous RGB channel (Red -> Blue -> Green -> Red)
    pub const fn previous_channel(&mut self) {
        self.active_channel = match self.active_channel {
            RgbChannel::Red => RgbChannel::Blue,
            RgbChannel::Green => RgbChannel::Red,
            RgbChannel::Blue => RgbChannel::Green,
        };
    }

    /// Increase the active channel value
    pub const fn increase_value(&mut self, amount: u8) {
        match self.active_channel {
            RgbChannel::Red => self.r = self.r.saturating_add(amount),
            RgbChannel::Green => self.g = self.g.saturating_add(amount),
            RgbChannel::Blue => self.b = self.b.saturating_add(amount),
        }
    }

    /// Decrease the active channel value
    pub const fn decrease_value(&mut self, amount: u8) {
        match self.active_channel {
            RgbChannel::Red => self.r = self.r.saturating_sub(amount),
            RgbChannel::Green => self.g = self.g.saturating_sub(amount),
            RgbChannel::Blue => self.b = self.b.saturating_sub(amount),
        }
    }

    /// Navigate in palette mode
    pub fn navigate_palette(&mut self, dx: i32, dy: i32) {
        let columns = self.palette.columns();
        let color_count = self.palette.color_count();

        match self.palette_focus {
            PaletteFocus::Colors => {
                // Navigate the color grid
                let current_row = self.selected_color / columns;
                let current_col = self.selected_color % columns;

                let new_col = (current_col as i32 + dx).clamp(0, columns as i32 - 1) as usize;
                let new_row =
                    (current_row as i32 + dy).clamp(0, (self.palette.rows() - 1) as i32) as usize;

                let new_idx = new_row * columns + new_col;
                if new_idx < color_count {
                    self.selected_color = new_idx;
                    self.sync_from_palette();
                }
            }
            PaletteFocus::Shades => {
                // Navigate the shade bar
                if let Some(color) = self.palette.color_at(self.selected_color) {
                    let shade_count = color.shade_count();
                    self.selected_shade =
                        (self.selected_shade as i32 + dx).clamp(0, shade_count as i32 - 1) as usize;
                    self.sync_from_palette();
                }
            }
        }
    }

    /// Move focus between colors and shades
    pub const fn toggle_palette_focus(&mut self) {
        self.palette_focus = match self.palette_focus {
            PaletteFocus::Colors => PaletteFocus::Shades,
            PaletteFocus::Shades => PaletteFocus::Colors,
        };
    }
}

impl Default for ColorPickerState {
    fn default() -> Self {
        Self::new()
    }
}

/// Render the color picker dialog
pub fn render_color_picker(f: &mut Frame, state: &super::AppState) {
    let theme = &state.theme;
    let picker_state = &state.color_picker_state;

    match picker_state.mode {
        ColorPickerMode::Palette => render_palette_mode(f, state),
        ColorPickerMode::CustomRgb => render_rgb_mode(f, state),
    }

    // Border around everything
    let area = centered_rect(70, 70, f.size());
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.primary));
    f.render_widget(block, area);
}

/// Render palette selection mode
fn render_palette_mode(f: &mut Frame, state: &super::AppState) {
    let theme = &state.theme;
    let area = centered_rect(70, 70, f.size());

    // Clear the background area first
    f.render_widget(Clear, area);

    // Render opaque background with theme color
    let background = Block::default().style(Style::default().bg(theme.background));
    f.render_widget(background, area);

    let picker_state = &state.color_picker_state;

    // Split into sections with spacing
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(2), // 0: Title
            Constraint::Length(1), // 1: Step 1 label
            Constraint::Length(9), // 2: Color grid (3 rows × 3 lines each for borders)
            Constraint::Length(1), // 3: Spacer
            Constraint::Length(1), // 4: Step 2 label
            Constraint::Length(3), // 5: Shade bar
            Constraint::Length(1), // 6: Spacer
            Constraint::Length(4), // 7: Preview
            Constraint::Min(0),    // 8: Flexible spacer (pushes instructions to bottom)
            Constraint::Length(2), // 9: Instructions
        ])
        .split(area);

    // Title - context-aware
    let title_text = match state.color_picker_context {
        Some(super::ColorPickerContext::IndividualKey) => "Individual Key Color Picker",
        Some(super::ColorPickerContext::LayerDefault) => "Layer Color Picker",
        Some(super::ColorPickerContext::Category) => "Category Color Picker",
        None => "Color Picker",
    };
    let title = Paragraph::new(title_text).style(
        Style::default()
            .fg(theme.primary)
            .add_modifier(Modifier::BOLD),
    );
    f.render_widget(title, chunks[0]);

    // Step 1 label
    let step1_style = if picker_state.palette_focus == PaletteFocus::Colors {
        Style::default()
            .fg(theme.accent)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme.text_muted)
    };
    let step1 = Paragraph::new("Step 1: Choose Base Color").style(step1_style);
    f.render_widget(step1, chunks[1]);

    // Color grid (4 columns x 3 rows)
    render_color_grid(f, chunks[2], picker_state, theme);

    // Step 2 label
    let step2_style = if picker_state.palette_focus == PaletteFocus::Shades {
        Style::default()
            .fg(theme.accent)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme.text_muted)
    };
    let step2 = Paragraph::new("Step 2: Choose Shade").style(step2_style);
    f.render_widget(step2, chunks[4]);

    // Shade bar
    render_shade_bar(f, chunks[5], picker_state, theme);

    // Preview
    render_preview(f, chunks[7], picker_state, theme);

    // Instructions (at bottom)
    let instructions = vec![Line::from(vec![
        Span::styled("←→↑↓", Style::default().fg(theme.accent)),
        Span::raw(" Navigate  "),
        Span::styled("Tab", Style::default().fg(theme.accent)),
        Span::raw(" Switch Step  "),
        Span::styled("c", Style::default().fg(theme.accent)),
        Span::raw(" Custom RGB  "),
        Span::styled("x", Style::default().fg(theme.accent)),
        Span::raw(" Clear  "),
        Span::styled("Enter", Style::default().fg(theme.accent)),
        Span::raw(" Apply  "),
        Span::styled("Esc", Style::default().fg(theme.accent)),
        Span::raw(" Cancel"),
    ])];
    let instructions_widget = Paragraph::new(instructions);
    f.render_widget(instructions_widget, chunks[9]);
}

/// Render the color grid (4x3)
fn render_color_grid(
    f: &mut Frame,
    area: Rect,
    picker_state: &ColorPickerState,
    theme: &super::Theme,
) {
    let columns = picker_state.palette.columns();
    let rows = picker_state.palette.rows();

    // Create row layout (3 lines per row for border support)
    let row_constraints: Vec<Constraint> = (0..rows).map(|_| Constraint::Length(3)).collect();

    let row_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(row_constraints)
        .split(area);

    for row in 0..rows {
        // Create column layout for this row
        let col_constraints: Vec<Constraint> = (0..columns)
            .map(|_| Constraint::Ratio(1, columns as u32))
            .collect();

        let col_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(col_constraints)
            .split(row_chunks[row]);

        for col in 0..columns {
            let idx = row * columns + col;
            if let Some(color) = picker_state.palette.color_at(idx) {
                let is_selected = idx == picker_state.selected_color
                    && picker_state.palette_focus == PaletteFocus::Colors;

                // Get the primary shade color for the dot
                let shade = color
                    .primary_shade()
                    .map_or(Color::White, |s| Color::Rgb(s.r, s.g, s.b));

                // Build the display: colored dot + name
                let dot = "●";
                let name = &color.name;

                let text_style = Style::default().fg(theme.text);

                let content = Line::from(vec![
                    Span::styled(dot, Style::default().fg(shade)),
                    Span::raw(" "),
                    Span::styled(name.as_str(), text_style),
                ]);

                if is_selected {
                    // Render with border for selected color
                    let block = Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(theme.accent));
                    let para = Paragraph::new(content).block(block);
                    f.render_widget(para, col_chunks[col]);
                } else {
                    // Render without border, but add top margin to align with bordered items
                    let inner_area = Rect {
                        x: col_chunks[col].x + 1,
                        y: col_chunks[col].y + 1,
                        width: col_chunks[col].width.saturating_sub(2),
                        height: 1,
                    };
                    let para = Paragraph::new(content);
                    f.render_widget(para, inner_area);
                }
            }
        }
    }
}

/// Render the shade bar for the selected color
fn render_shade_bar(
    f: &mut Frame,
    area: Rect,
    picker_state: &ColorPickerState,
    theme: &super::Theme,
) {
    if let Some(color) = picker_state.palette.color_at(picker_state.selected_color) {
        let shade_count = color.shade_count();

        // Create columns for each shade
        let col_constraints: Vec<Constraint> = (0..shade_count)
            .map(|_| Constraint::Ratio(1, shade_count as u32))
            .collect();

        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(col_constraints)
            .split(area);

        for (i, shade) in color.shades.iter().enumerate() {
            let is_selected = i == picker_state.selected_shade
                && picker_state.palette_focus == PaletteFocus::Shades;

            let shade_color = Color::Rgb(shade.r, shade.g, shade.b);

            // Shade block with level label
            let label = format!("{}", shade.level);

            let (block_style, text_style) = if is_selected {
                (
                    Style::default().bg(shade_color),
                    Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                (
                    Style::default().bg(shade_color),
                    Style::default().fg(theme.text_muted),
                )
            };

            // Draw the color block
            let block = Block::default().style(block_style);
            f.render_widget(block, chunks[i]);

            // Draw selection indicator and label below
            if chunks[i].height >= 2 {
                let indicator = if is_selected { "▲" } else { " " };
                let label_area = Rect {
                    x: chunks[i].x,
                    y: chunks[i].y + chunks[i].height - 1,
                    width: chunks[i].width,
                    height: 1,
                };
                let label_content = format!("{indicator}{label}");
                let label_para = Paragraph::new(label_content).style(text_style);
                f.render_widget(label_para, label_area);
            }
        }
    }
}

/// Render preview section
fn render_preview(
    f: &mut Frame,
    area: Rect,
    picker_state: &ColorPickerState,
    theme: &super::Theme,
) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    // Color preview block
    let preview_color = Color::Rgb(picker_state.r, picker_state.g, picker_state.b);
    let preview = Block::default()
        .title(" Preview ")
        .borders(Borders::ALL)
        .style(Style::default().bg(preview_color));
    f.render_widget(preview, chunks[0]);

    // Hex code and color name - use pre-computed hex from Shade when available
    let hex = picker_state
        .get_selected_shade()
        .map_or_else(|| picker_state.get_color().to_hex(), |s| s.hex.clone());
    let color_name = picker_state
        .palette
        .color_at(picker_state.selected_color)
        .map(|c| {
            format!(
                "{}-{}",
                c.name,
                picker_state
                    .get_selected_shade()
                    .map(|s| s.level.to_string())
                    .unwrap_or_default()
            )
        })
        .unwrap_or_default();

    let info = Paragraph::new(vec![
        Line::from(vec![
            Span::styled(" Hex: ", Style::default().fg(theme.text_muted)),
            Span::styled(
                hex,
                Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled(" Name: ", Style::default().fg(theme.text_muted)),
            Span::styled(color_name, Style::default().fg(theme.text)),
        ]),
    ])
    .block(Block::default().borders(Borders::ALL).title(" Info "));
    f.render_widget(info, chunks[1]);
}

/// Render custom RGB mode
fn render_rgb_mode(f: &mut Frame, state: &super::AppState) {
    let theme = &state.theme;
    let area = centered_rect(70, 70, f.size());

    // Clear the background area first
    f.render_widget(Clear, area);

    // Render opaque background with theme color
    let background = Block::default().style(Style::default().bg(theme.background));
    f.render_widget(background, area);

    // Split into sections
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(2), // Title
            Constraint::Length(3), // Red slider
            Constraint::Length(3), // Green slider
            Constraint::Length(3), // Blue slider
            Constraint::Length(4), // Color preview
            Constraint::Length(3), // Hex display
            Constraint::Length(2), // Instructions
        ])
        .split(area);

    let picker_state = &state.color_picker_state;

    // Title - context-aware
    let title_text = match state.color_picker_context {
        Some(super::ColorPickerContext::IndividualKey) => {
            "Individual Key Color Picker (Custom RGB)"
        }
        Some(super::ColorPickerContext::LayerDefault) => "Layer Color Picker (Custom RGB)",
        Some(super::ColorPickerContext::Category) => "Category Color Picker (Custom RGB)",
        None => "Custom RGB",
    };
    let title = Paragraph::new(title_text).style(
        Style::default()
            .fg(theme.primary)
            .add_modifier(Modifier::BOLD),
    );
    f.render_widget(title, chunks[0]);

    // Red channel slider
    render_channel_slider(
        f,
        chunks[1],
        "Red",
        picker_state.r,
        Color::Red,
        picker_state.active_channel == RgbChannel::Red,
        theme.text_muted,
    );

    // Green channel slider
    render_channel_slider(
        f,
        chunks[2],
        "Green",
        picker_state.g,
        Color::Green,
        picker_state.active_channel == RgbChannel::Green,
        theme.text_muted,
    );

    // Blue channel slider
    render_channel_slider(
        f,
        chunks[3],
        "Blue",
        picker_state.b,
        Color::Blue,
        picker_state.active_channel == RgbChannel::Blue,
        theme.text_muted,
    );

    // Color preview
    let preview_color = Color::Rgb(picker_state.r, picker_state.g, picker_state.b);
    let preview = Block::default()
        .title(" Preview ")
        .borders(Borders::ALL)
        .style(Style::default().bg(preview_color));
    f.render_widget(preview, chunks[4]);

    // Hex code display
    let hex = picker_state.get_color().to_hex();
    let hex_display = Paragraph::new(format!("  {hex}"))
        .style(Style::default().fg(theme.text).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::ALL).title(" Hex Code "));
    f.render_widget(hex_display, chunks[5]);

    // Instructions
    let instructions = vec![Line::from(vec![
        Span::styled("↑↓", Style::default().fg(theme.accent)),
        Span::raw(" ±10  "),
        Span::styled("←→", Style::default().fg(theme.accent)),
        Span::raw(" ±1  "),
        Span::styled("Tab", Style::default().fg(theme.accent)),
        Span::raw(" Channel  "),
        Span::styled("p", Style::default().fg(theme.accent)),
        Span::raw(" Palette  "),
        Span::styled("x", Style::default().fg(theme.accent)),
        Span::raw(" Clear  "),
        Span::styled("Enter", Style::default().fg(theme.accent)),
        Span::raw(" Apply  "),
        Span::styled("Esc", Style::default().fg(theme.accent)),
        Span::raw(" Cancel"),
    ])];
    let instructions_widget = Paragraph::new(instructions);
    f.render_widget(instructions_widget, chunks[6]);
}

/// Render a single channel slider
fn render_channel_slider(
    f: &mut Frame,
    area: Rect,
    label: &str,
    value: u8,
    color: Color,
    is_active: bool,
    inactive_color: Color,
) {
    let percentage = (f64::from(value) / 255.0 * 100.0) as u16;
    let label_text = format!("{label}: {value:3}");

    let style = if is_active {
        Style::default().fg(color).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(inactive_color)
    };

    let gauge = Gauge::default()
        .block(Block::default().borders(Borders::NONE))
        .gauge_style(style)
        .label(label_text)
        .percent(percentage);

    f.render_widget(gauge, area);
}

/// Handle input for color picker
pub fn handle_input(state: &mut super::AppState, key: KeyEvent) -> anyhow::Result<bool> {
    match state.color_picker_state.mode {
        ColorPickerMode::Palette => handle_palette_input(state, key),
        ColorPickerMode::CustomRgb => handle_rgb_input(state, key),
    }
}

/// Handle input in palette mode
fn handle_palette_input(state: &mut super::AppState, key: KeyEvent) -> anyhow::Result<bool> {
    match key.code {
        KeyCode::Esc => {
            state.active_popup = None;
            state.color_picker_context = None;
            state.set_status("Cancelled");
            Ok(false)
        }
        KeyCode::Enter => {
            apply_color(state);
            Ok(false)
        }
        KeyCode::Char('x') | KeyCode::Delete => {
            // Clear/reset the color
            clear_color(state);
            Ok(false)
        }
        KeyCode::Char('c' | 'C') => {
            // Switch to custom RGB mode
            state.color_picker_state.mode = ColorPickerMode::CustomRgb;
            Ok(false)
        }
        KeyCode::Tab => {
            // Toggle between colors and shades
            state.color_picker_state.toggle_palette_focus();
            Ok(false)
        }
        KeyCode::Up | KeyCode::Char('k') => {
            match state.color_picker_state.palette_focus {
                PaletteFocus::Colors => state.color_picker_state.navigate_palette(0, -1),
                PaletteFocus::Shades => {
                    // Move back to colors when pressing up in shades
                    state.color_picker_state.palette_focus = PaletteFocus::Colors;
                }
            }
            Ok(false)
        }
        KeyCode::Down | KeyCode::Char('j') => {
            match state.color_picker_state.palette_focus {
                PaletteFocus::Colors => {
                    // Check if we should move to shades or next row
                    let columns = state.color_picker_state.palette.columns();
                    let current_row = state.color_picker_state.selected_color / columns;
                    let rows = state.color_picker_state.palette.rows();

                    if current_row >= rows - 1 {
                        // At bottom row, move to shades
                        state.color_picker_state.palette_focus = PaletteFocus::Shades;
                    } else {
                        state.color_picker_state.navigate_palette(0, 1);
                    }
                }
                PaletteFocus::Shades => {
                    // Already at bottom, do nothing
                }
            }
            Ok(false)
        }
        KeyCode::Left | KeyCode::Char('h') => {
            state.color_picker_state.navigate_palette(-1, 0);
            Ok(false)
        }
        KeyCode::Right | KeyCode::Char('l') => {
            state.color_picker_state.navigate_palette(1, 0);
            Ok(false)
        }
        _ => Ok(false),
    }
}

/// Handle input in custom RGB mode
fn handle_rgb_input(state: &mut super::AppState, key: KeyEvent) -> anyhow::Result<bool> {
    match key.code {
        KeyCode::Esc => {
            state.active_popup = None;
            state.color_picker_context = None;
            state.set_status("Cancelled");
            Ok(false)
        }
        KeyCode::Enter => {
            apply_color(state);
            Ok(false)
        }
        KeyCode::Char('x') | KeyCode::Delete => {
            // Clear/reset the color
            clear_color(state);
            Ok(false)
        }
        KeyCode::Char('p' | 'P') => {
            // Switch to palette mode
            state.color_picker_state.mode = ColorPickerMode::Palette;
            Ok(false)
        }
        KeyCode::Up | KeyCode::Char('k') => {
            state.color_picker_state.increase_value(10);
            Ok(false)
        }
        KeyCode::Down | KeyCode::Char('j') => {
            state.color_picker_state.decrease_value(10);
            Ok(false)
        }
        KeyCode::Right | KeyCode::Char('l') => {
            state.color_picker_state.increase_value(1);
            Ok(false)
        }
        KeyCode::Left | KeyCode::Char('h') => {
            state.color_picker_state.decrease_value(1);
            Ok(false)
        }
        KeyCode::Tab => {
            if key.modifiers.contains(KeyModifiers::SHIFT) {
                state.color_picker_state.previous_channel();
            } else {
                state.color_picker_state.next_channel();
            }
            Ok(false)
        }
        _ => Ok(false),
    }
}

/// Apply the selected color based on context
fn apply_color(state: &mut super::AppState) {
    let color = state.color_picker_state.get_color();

    match state.color_picker_context {
        Some(super::ColorPickerContext::IndividualKey) => {
            if let Some(key) = state.get_selected_key_mut() {
                key.color_override = Some(color);
                state.mark_dirty();
                state.set_status(format!("Set key color to {}", color.to_hex()));
            }
            state.active_popup = None;
            state.color_picker_context = None;
        }
        Some(super::ColorPickerContext::LayerDefault) => {
            if let Some(layer) = state.layout.layers.get_mut(state.current_layer) {
                layer.default_color = color;
                state.mark_dirty();
                state.set_status(format!("Set layer default color to {}", color.to_hex()));
            }
            state.active_popup = None;
            state.color_picker_context = None;
        }
        Some(super::ColorPickerContext::Category) => {
            use super::category_manager::ManagerMode;

            match &state.category_manager_state.mode {
                ManagerMode::CreatingColor { name } => {
                    let name = name.clone();
                    let id = name.to_lowercase().replace(' ', "-");

                    if let Ok(category) = crate::models::Category::new(&id, &name, color) {
                        state.layout.categories.push(category);
                        state.mark_dirty();
                        state.set_status(format!("Created category '{name}'"));
                    } else {
                        state.set_error("Failed to create category");
                    }

                    // Always reset mode back to browsing after color selection
                    state.category_manager_state.cancel();
                    state.active_popup = Some(super::PopupType::CategoryManager);
                    state.color_picker_context = None;
                }
                ManagerMode::Browsing => {
                    let selected_idx = state.category_manager_state.selected;
                    if let Some(category) = state.layout.categories.get_mut(selected_idx) {
                        let name = category.name.clone();
                        category.set_color(color);
                        state.mark_dirty();
                        state.set_status(format!("Updated color for '{name}'"));
                    }

                    state.active_popup = Some(super::PopupType::CategoryManager);
                    state.color_picker_context = None;
                }
                _ => {
                    state.set_error("Invalid category manager state");
                    state.active_popup = Some(super::PopupType::CategoryManager);
                    state.color_picker_context = None;
                }
            }
        }
        None => {
            state.set_error("No color context set");
            state.active_popup = None;
            state.color_picker_context = None;
        }
    }
}

/// Clear/reset the color based on context
fn clear_color(state: &mut super::AppState) {
    match state.color_picker_context {
        Some(super::ColorPickerContext::IndividualKey) => {
            // Clear individual key color override - key will inherit layer default
            if let Some(key) = state.get_selected_key_mut() {
                key.color_override = None;
                state.mark_dirty();
                state.set_status("Cleared key color (using layer default)");
            }
            state.active_popup = None;
            state.color_picker_context = None;
        }
        Some(super::ColorPickerContext::LayerDefault) => {
            // Reset layer default to white
            let default_color = crate::models::RgbColor::new(255, 255, 255);
            if let Some(layer) = state.layout.layers.get_mut(state.current_layer) {
                layer.default_color = default_color;
                state.mark_dirty();
                state.set_status("Reset layer color to white");
            }
            state.active_popup = None;
            state.color_picker_context = None;
        }
        Some(super::ColorPickerContext::Category) => {
            // Categories must have a color - clearing is not allowed
            state.set_error("Categories must have a color");
        }
        None => {
            state.set_error("No color context set");
            state.active_popup = None;
            state.color_picker_context = None;
        }
    }
}

/// Helper to create a centered rectangle
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
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
