//! Keyboard widget for rendering the visual keyboard layout

use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use super::AppState;

/// Keyboard widget renders the visual keyboard layout
pub struct KeyboardWidget;

impl KeyboardWidget {
    /// Render the keyboard widget
    pub fn render(f: &mut Frame, area: Rect, state: &AppState) {
        let theme = &state.theme;
        
        // Get current layer
        let layer = if let Some(layer) = state.layout.layers.get(state.current_layer) {
            layer
        } else {
            // If layer doesn't exist, show error
            let error = Paragraph::new("Layer not found")
                .block(Block::default().title(" Keyboard ").borders(Borders::ALL));
            f.render_widget(error, area);
            return;
        };

        // Render outer container
        let outer_block = Block::default()
            .title(format!(" Layer {}: {} ", state.current_layer, layer.name))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.primary));
        f.render_widget(outer_block, area);

        // Calculate inner area for keys (inside the outer border)
        let inner_area = Rect {
            x: area.x + 1,
            y: area.y + 1,
            width: area.width.saturating_sub(2),
            height: area.height.saturating_sub(2),
        };

        // Each key needs: 7 chars width + 2 for borders = 9 total width
        // Each key needs: 3 lines height (1 content + 2 borders)
        let key_width = 9;
        let key_height = 3;

        // Render each key as an individual block
        for key in &layer.keys {
            let row = key.position.row as usize;
            let col = key.position.col as usize;

            // Calculate key position
            let key_x = inner_area.x + (col * key_width) as u16;
            let key_y = inner_area.y + (row * key_height) as u16;

            // Skip if key is outside visible area
            if key_x >= inner_area.x + inner_area.width
                || key_y >= inner_area.y + inner_area.height
            {
                continue;
            }

            let key_area = Rect {
                x: key_x,
                y: key_y,
                width: key_width.min((inner_area.x + inner_area.width).saturating_sub(key_x) as usize) as u16,
                height: key_height.min((inner_area.y + inner_area.height).saturating_sub(key_y) as usize) as u16,
            };

            // Skip if key area is too small
            if key_area.width < 7 || key_area.height < 3 {
                continue;
            }

            let is_selected = row == state.selected_position.row as usize
                && col == state.selected_position.col as usize;

            // Resolve key color for display (respects colors_enabled and inactive_key_behavior)
            let (key_color, color_indicator) = if let Some(current_layer) = state.layout.layers.get(state.current_layer) {
                if !current_layer.layer_colors_enabled {
                    // Layer colors disabled - show neutral gray with "-" indicator
                    (Color::DarkGray, "-")
                } else {
                    // Use resolve_display_color which considers inactive_key_behavior
                    let (rgb, is_key_specific) = state.layout.resolve_display_color(state.current_layer, key);
                    let color = Color::Rgb(rgb.r, rgb.g, rgb.b);
                    
                    let indicator = if is_key_specific {
                        if key.color_override.is_some() {
                            "i" // Individual override
                        } else {
                            "k" // Key category
                        }
                    } else if layer.category_id.is_some() {
                        "L" // Layer category
                    } else {
                        "d" // Layer default
                    };
                    (color, indicator)
                }
            } else {
                (Color::DarkGray, "-")
            };

            // Format keycode for display
            let display = Self::format_keycode(&key.keycode);

            // Create key content: keycode on left, indicator on right
            let content = Line::from(vec![
                Span::styled(
                    format!("{:<3}", display),
                    Style::default().fg(theme.text),
                ),
                Span::raw(" "),
                Span::styled(
                    color_indicator,
                    Style::default().fg(key_color).add_modifier(Modifier::BOLD),
                ),
            ]);

            // Create the key block with colored border
            let key_block = if is_selected {
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.accent).add_modifier(Modifier::BOLD))
                    .style(Style::default().bg(theme.accent).fg(theme.background))
            } else {
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(key_color))
            };

            let key_paragraph = Paragraph::new(content).block(key_block);

            f.render_widget(key_paragraph, key_area);
        }
    }

    /// Format keycode for compact display (first 3-4 chars)
    fn format_keycode(keycode: &str) -> String {
        // Remove KC_ prefix if present
        let display = keycode.strip_prefix("KC_").unwrap_or(keycode);

        // Take first 3 characters
        display.chars().take(3).collect()
    }
}
