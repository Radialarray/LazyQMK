//! Keyboard widget for rendering the visual keyboard layout

use ratatui::{
    layout::{Constraint, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Cell, Row, Table},
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

        // Find dimensions by examining all key positions
        let max_row = layer.keys.iter().map(|k| k.position.row).max().unwrap_or(0) + 1;
        let max_col = layer.keys.iter().map(|k| k.position.col).max().unwrap_or(0) + 1;

        // Calculate viewport bounds (accounting for borders and spacing)
        // Area height - 2 for borders, divide by row height
        let visible_rows = (area.height.saturating_sub(3)) as usize;
        // Area width - 2 for borders, divide by column width (7 chars + 1 spacing)
        let visible_cols = (area.width.saturating_sub(2) / 8) as usize;

        // Determine which rows/cols to render based on viewport
        let start_row = 0; // Could scroll in future
        let end_row = (start_row + visible_rows).min(max_row as usize);
        let start_col = 0; // Could scroll in future
        let end_col = (start_col + visible_cols).min(max_col as usize);

        // Build a grid (only for visible range)
        let visible_row_count = end_row - start_row;
        let visible_col_count = end_col - start_col;
        let mut grid = vec![vec![String::from("     "); visible_col_count]; visible_row_count];

        // Fill grid with keycodes and resolve colors (only visible keys)
        for key in &layer.keys {
            let row = key.position.row as usize;
            let col = key.position.col as usize;

            // Skip keys outside viewport
            if row < start_row || row >= end_row || col < start_col || col >= end_col {
                continue;
            }

            // Map to grid coordinates
            let grid_row = row - start_row;
            let grid_col = col - start_col;

            if grid_row < grid.len() && grid_col < grid[0].len() {
                // Abbreviate keycode for display
                let display = Self::format_keycode(&key.keycode);

                // Add color indicator based on source priority
                let color_indicator = if key.color_override.is_some() {
                    "i" // Individual override
                } else if key.category_id.is_some() {
                    "k" // Key category
                } else if layer.category_id.is_some() {
                    "L" // Layer category
                } else {
                    "d" // Layer default
                };

                // Format: "ABC i" (3 chars keycode + space + indicator)
                grid[grid_row][grid_col] = format!(" {display:<3}{color_indicator}");
            }
        }

        // Build table rows with color resolution
        let mut table_rows = vec![];
        for (grid_row_idx, row_data) in grid.iter().enumerate() {
            let cells: Vec<Cell> = row_data
                .iter()
                .enumerate()
                .map(|(grid_col_idx, text)| {
                    // Map back to actual position
                    let actual_row = grid_row_idx + start_row;
                    let actual_col = grid_col_idx + start_col;

                    let is_selected = actual_row == state.selected_position.row as usize
                        && actual_col == state.selected_position.col as usize;

                    // Find the key at this position to get its color
                    let key_color = layer
                        .keys
                        .iter()
                        .find(|k| {
                            k.position.row as usize == actual_row
                                && k.position.col as usize == actual_col
                        })
                        .map(|key| {
                            let rgb = state.layout.resolve_key_color(state.current_layer, key);
                            Color::Rgb(rgb.r, rgb.g, rgb.b)
                        });

                    let style = if is_selected {
                        Style::default().fg(theme.background).bg(theme.accent)
                    } else if let Some(color) = key_color {
                        Style::default().fg(color)
                    } else {
                        Style::default().fg(theme.text)
                    };

                    Cell::from(text.as_str()).style(style)
                })
                .collect();

            table_rows.push(Row::new(cells));
        }

        // Create column constraints (equal width) for visible columns only
        let constraints = vec![Constraint::Length(7); visible_col_count];

        // Build table widget
        let table = Table::new(table_rows, constraints)
            .block(
                Block::default()
                    .title(format!(" Layer {}: {} ", state.current_layer, layer.name))
                    .borders(Borders::ALL),
            )
            .column_spacing(1);

        f.render_widget(table, area);
    }

    /// Format keycode for compact display (first 3-4 chars)
    fn format_keycode(keycode: &str) -> String {
        // Remove KC_ prefix if present
        let display = keycode.strip_prefix("KC_").unwrap_or(keycode);

        // Take first 3 characters
        display.chars().take(3).collect()
    }
}

use ratatui::widgets::Paragraph;
