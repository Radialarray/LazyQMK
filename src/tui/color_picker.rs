//! Color picker dialog for selecting RGB colors

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph},
    Frame,
};

use crate::models::RgbColor;

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

/// State for the color picker dialog
#[derive(Debug, Clone)]
pub struct ColorPickerState {
    /// Red channel value (0-255)
    pub r: u8,
    /// Green channel value (0-255)
    pub g: u8,
    /// Blue channel value (0-255)
    pub b: u8,
    /// Currently active channel for editing
    pub active_channel: RgbChannel,
}

impl ColorPickerState {
    /// Create a new color picker with default white color
    #[must_use]
    pub const fn new() -> Self {
        Self {
            r: 255,
            g: 255,
            b: 255,
            active_channel: RgbChannel::Red,
        }
    }

    /// Create a color picker initialized with a specific color
    #[must_use]
    pub const fn with_color(color: RgbColor) -> Self {
        Self {
            r: color.r,
            g: color.g,
            b: color.b,
            active_channel: RgbChannel::Red,
        }
    }

    /// Get the current color
    #[must_use]
    pub const fn get_color(&self) -> RgbColor {
        RgbColor::new(self.r, self.g, self.b)
    }

    /// Switch to next channel (Red -> Green -> Blue -> Red)
    pub const fn next_channel(&mut self) {
        self.active_channel = match self.active_channel {
            RgbChannel::Red => RgbChannel::Green,
            RgbChannel::Green => RgbChannel::Blue,
            RgbChannel::Blue => RgbChannel::Red,
        };
    }

    /// Switch to previous channel (Red -> Blue -> Green -> Red)
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
}

impl Default for ColorPickerState {
    fn default() -> Self {
        Self::new()
    }
}

/// Render the color picker dialog
pub fn render_color_picker(f: &mut Frame, state: &super::AppState) {
    let theme = &state.theme;
    let area = centered_rect(60, 50, f.size());

    // Split into sections
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Length(3), // Red slider
            Constraint::Length(3), // Green slider
            Constraint::Length(3), // Blue slider
            Constraint::Length(4), // Color preview
            Constraint::Length(3), // Hex display
            Constraint::Length(3), // Instructions
        ])
        .split(area);

    let picker_state = &state.color_picker_state;

    // Title
    let title = Paragraph::new("RGB Color Picker").style(
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
        Span::raw("  "),
        Span::styled("↑↓", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" Adjust  "),
        Span::styled("Tab", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" Next Channel  "),
        Span::styled("Enter", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" Apply  "),
        Span::styled("Esc", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" Cancel"),
    ])];
    let instructions_widget = Paragraph::new(instructions);
    f.render_widget(instructions_widget, chunks[6]);

    // Border around everything
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.primary));
    f.render_widget(block, area);
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
    match key.code {
        KeyCode::Esc => {
            // Cancel
            state.active_popup = None;
            state.color_picker_context = None;
            state.set_status("Cancelled");
            Ok(false)
        }
        KeyCode::Enter => {
            // Apply color based on context
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
                    // Handle category color completion
                    use super::category_manager::ManagerMode;

                    match &state.category_manager_state.mode {
                        ManagerMode::CreatingColor { name } => {
                            // Create new category (T108)
                            let name = name.clone();
                            let id = name.to_lowercase().replace(' ', "-");

                            if let Ok(category) = crate::models::Category::new(&id, &name, color) {
                                state.layout.categories.push(category);
                                state.mark_dirty();
                                state.category_manager_state.cancel();
                                state.set_status(format!("Created category '{name}'"));
                            } else {
                                state.set_error("Failed to create category");
                            }

                            state.active_popup = Some(super::PopupType::CategoryManager);
                            state.color_picker_context = None;
                        }
                        ManagerMode::Browsing => {
                            // Changing color of existing category (T110)
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

            Ok(false)
        }
        KeyCode::Up => {
            // Increase value (large step)
            state.color_picker_state.increase_value(10);
            Ok(false)
        }
        KeyCode::Down => {
            // Decrease value (large step)
            state.color_picker_state.decrease_value(10);
            Ok(false)
        }
        KeyCode::Right => {
            // Increase value (small step)
            state.color_picker_state.increase_value(1);
            Ok(false)
        }
        KeyCode::Left => {
            // Decrease value (small step)
            state.color_picker_state.decrease_value(1);
            Ok(false)
        }
        KeyCode::Tab => {
            if key.modifiers.contains(KeyModifiers::SHIFT) {
                // Previous channel
                state.color_picker_state.previous_channel();
            } else {
                // Next channel
                state.color_picker_state.next_channel();
            }
            Ok(false)
        }
        _ => Ok(false),
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
