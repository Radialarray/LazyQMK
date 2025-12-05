//! Keyboard widget for rendering the visual keyboard layout
//!
//! Key rendering with support for:
//! - Simple keycodes (`KC_A`, `KC_SPC`, etc.)
//! - Tap-hold keycodes (LT, MT, LM, `SH_T`) with dual-line display
//! - Color type indicators in border (i=individual, k=category, L=layer, d=default)
//! - RGB color borders based on the color priority system

// Allow intentional type casts for terminal rendering
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]

use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::keycode_db::TapHoldType;
use super::AppState;

/// Keyboard widget renders the visual keyboard layout
pub struct KeyboardWidget;

/// Parsed representation of a tap-hold keycode
#[derive(Debug, Clone)]
pub struct TapHoldKeycode {
    /// The hold action (e.g., "L1" for layer 1, "CTL" for Ctrl)
    pub hold: String,
    /// The tap action (e.g., "A" for `KC_A`)
    pub tap: String,
    /// The type of tap-hold (for display purposes)
    #[allow(dead_code)]
    pub kind: TapHoldKind,
}

/// Types of tap-hold keycodes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TapHoldKind {
    /// LT(layer, keycode) - Layer on hold, keycode on tap
    LayerTap,
    /// MT(mod, keycode) - Modifier on hold, keycode on tap
    ModTap,
    /// LM(layer, mod) - Layer + modifier on hold
    LayerMod,
    /// `SH_T(keycode)` - Swap hands on hold, keycode on tap
    SwapHandsTap,
}

impl KeyboardWidget {
    /// Render the keyboard widget
    #[allow(clippy::too_many_lines)]
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
        // Each key needs: 4 lines height (2 content + 2 borders) to support tap-hold display
        let key_width = 9;
        let key_height = 4;

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
            if key_area.width < 7 || key_area.height < 4 {
                continue;
            }

            let is_selected = row == state.selected_position.row as usize
                && col == state.selected_position.col as usize;

            // Check if this key is the cut source (for visual feedback)
            let is_cut_source = state.clipboard.is_cut_source(state.current_layer, key.position);

            // Check if this key is part of multi-selection
            let is_in_selection = state.selected_keys.contains(&key.position);

            // Check if this key should flash (paste feedback)
            let is_flashing = state.flash_highlight
                .is_some_and(|(layer, pos, _)| layer == state.current_layer && pos == key.position);

            // Resolve key color for display (respects colors_enabled and inactive_key_behavior)
            let (key_color, color_indicator) = if let Some(current_layer) = state.layout.layers.get(state.current_layer) {
                if !current_layer.layer_colors_enabled {
                    // Layer colors disabled - use theme text_muted for visible border
                    (theme.text_muted, "-")
                } else {
                    // Use resolve_display_color which considers inactive_key_behavior
                    let (rgb, is_key_specific) = state.layout.resolve_display_color(state.current_layer, key);
                    
                    // Check if the color is too dark to be visible (e.g., black from "Off" behavior)
                    // If brightness is below threshold, use theme.text_muted for visibility
                    let brightness = (u16::from(rgb.r) + u16::from(rgb.g) + u16::from(rgb.b)) / 3;
                    let color = if brightness < 30 {
                        // Color too dark for TUI visibility, use muted theme color
                        theme.text_muted
                    } else {
                        Color::Rgb(rgb.r, rgb.g, rgb.b)
                    };
                    
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
                // Fallback - use theme text_muted for visible border
                (theme.text_muted, "-")
            };

            // Parse keycode to determine if it's a tap-hold type
            let tap_hold = Self::parse_tap_hold_keycode(&key.keycode, state);
            
            // Build content lines based on keycode type
            let content: Vec<Line> = if let Some(th) = &tap_hold {
                // Tap-hold keycode: show hold on top, tap on bottom
                vec![
                    Line::from(vec![
                        Span::styled(
                            format!("▼{:<5}", Self::truncate(&th.hold, 5)),
                            Style::default().fg(theme.text_muted),
                        ),
                    ]),
                    Line::from(vec![
                        Span::styled(
                            format!(" {:<5}", Self::truncate(&th.tap, 5)),
                            Style::default().fg(theme.text),
                        ),
                    ]),
                ]
            } else {
                // Simple keycode: center vertically with two lines
                let display = Self::format_simple_keycode(&key.keycode);
                vec![
                    Line::from(""), // Empty first line for vertical centering
                    Line::from(vec![
                        Span::styled(
                            format!(" {:<5}", Self::truncate(&display, 5)),
                            Style::default().fg(theme.text),
                        ),
                    ]),
                ]
            };

            // Render the key with custom border that includes color indicator
            Self::render_key_with_indicator(
                f,
                key_area,
                &content,
                color_indicator,
                key_color,
                is_selected,
                is_cut_source,
                is_in_selection,
                is_flashing,
                theme,
            );
        }
    }

    /// Render a key with the color indicator embedded in the top border
    #[allow(clippy::too_many_lines)]
    fn render_key_with_indicator(
        f: &mut Frame,
        area: Rect,
        content: &[Line],
        indicator: &str,
        border_color: Color,
        is_selected: bool,
        is_cut_source: bool,
        is_in_selection: bool,
        is_flashing: bool,
        theme: &super::Theme,
    ) {
        // Determine colors based on selection, cut state, multi-selection, and flash
        let (border_style, content_bg, content_fg) = if is_flashing {
            // Flash highlight: bright accent background
            (
                Style::default().fg(theme.success).add_modifier(Modifier::BOLD),
                Some(theme.success),
                theme.background,
            )
        } else if is_selected {
            (
                Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
                Some(theme.accent),
                theme.background,
            )
        } else if is_in_selection {
            // Multi-selection: highlighted but not as prominent as primary selection
            (
                Style::default().fg(theme.warning).add_modifier(Modifier::BOLD),
                Some(theme.surface),
                theme.text,
            )
        } else if is_cut_source {
            // Cut source: dimmed appearance
            (
                Style::default().fg(border_color).add_modifier(Modifier::DIM),
                None,
                theme.text_muted,
            )
        } else {
            (
                Style::default().fg(border_color),
                None,
                theme.text,
            )
        };

        // Draw the custom border with indicator in top-right corner
        let buf = f.buffer_mut();
        
        // Top border with indicator in right corner: ┌──────i┐
        let top_y = area.y;
        let left_x = area.x;
        let right_x = area.x + area.width.saturating_sub(1);
        
        // Draw corners
        buf.get_mut(left_x, top_y).set_char('┌').set_style(border_style);
        buf.get_mut(right_x, top_y).set_char('┐').set_style(border_style);
        buf.get_mut(left_x, area.y + area.height.saturating_sub(1)).set_char('└').set_style(border_style);
        buf.get_mut(right_x, area.y + area.height.saturating_sub(1)).set_char('┘').set_style(border_style);
        
        // Top border with indicator in right corner (just before ┐)
        let top_width = area.width.saturating_sub(2) as usize;
        if top_width > 0 {
            // Indicator goes in the rightmost position of the top border
            let indicator_pos = top_width.saturating_sub(1);
            
            for i in 0..top_width {
                let x = left_x + 1 + i as u16;
                if i == indicator_pos {
                    // Draw the indicator character with the border color
                    let indicator_style = if is_selected {
                        Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(border_color).add_modifier(Modifier::BOLD)
                    };
                    buf.get_mut(x, top_y)
                        .set_char(indicator.chars().next().unwrap_or('─'))
                        .set_style(indicator_style);
                } else {
                    buf.get_mut(x, top_y).set_char('─').set_style(border_style);
                }
            }
        }
        
        // Bottom border
        for i in 1..area.width.saturating_sub(1) {
            buf.get_mut(left_x + i, area.y + area.height.saturating_sub(1))
                .set_char('─')
                .set_style(border_style);
        }
        
        // Left and right borders
        for row in 1..area.height.saturating_sub(1) {
            buf.get_mut(left_x, area.y + row).set_char('│').set_style(border_style);
            buf.get_mut(right_x, area.y + row).set_char('│').set_style(border_style);
        }
        
        // Fill content area background if selected
        if let Some(bg) = content_bg {
            for row in 1..area.height.saturating_sub(1) {
                for col in 1..area.width.saturating_sub(1) {
                    buf.get_mut(area.x + col, area.y + row)
                        .set_bg(bg);
                }
            }
        }
        
        // Render content lines
        let content_area = Rect {
            x: area.x + 1,
            y: area.y + 1,
            width: area.width.saturating_sub(2),
            height: area.height.saturating_sub(2),
        };
        
        for (i, line) in content.iter().enumerate() {
            if i >= content_area.height as usize {
                break;
            }
            let y = content_area.y + i as u16;
            
            // Render each span in the line
            let mut x = content_area.x;
            for span in &line.spans {
                for ch in span.content.chars() {
                    if x >= content_area.x + content_area.width {
                        break;
                    }
                    let mut style = span.style;
                    if is_selected {
                        style = style.fg(content_fg);
                        if let Some(bg) = content_bg {
                            style = style.bg(bg);
                        }
                    }
                    buf.get_mut(x, y).set_char(ch).set_style(style);
                    x += 1;
                }
            }
        }
    }

    /// Parse a keycode to extract tap-hold components if applicable.
    /// 
    /// Uses the keycode database to dynamically detect mod-tap prefixes
    /// rather than maintaining a hardcoded list.
    fn parse_tap_hold_keycode(keycode: &str, state: &AppState) -> Option<TapHoldKeycode> {
        // Use the keycode database to parse tap-hold keycodes
        let info = state.keycode_db.parse_tap_hold(keycode)?;
        
        match info.tap_hold_type {
            TapHoldType::LayerTap => {
                // LT(layer, keycode) - Layer Tap
                let layer_display = Self::resolve_layer_display(&info.arg1, state);
                let tap_display = Self::format_simple_keycode(info.arg2.as_deref().unwrap_or(""));
                Some(TapHoldKeycode {
                    hold: format!("L{layer_display}"),
                    tap: tap_display,
                    kind: TapHoldKind::LayerTap,
                })
            }
            TapHoldType::ModTap => {
                // MT(mod, keycode) - Custom Mod Tap
                let mod_display = Self::format_modifier(&info.arg1);
                let tap_display = Self::format_simple_keycode(info.arg2.as_deref().unwrap_or(""));
                Some(TapHoldKeycode {
                    hold: mod_display,
                    tap: tap_display,
                    kind: TapHoldKind::ModTap,
                })
            }
            TapHoldType::ModTapNamed => {
                // Named mod-tap like LCTL_T(keycode)
                // Use keycode_db to get the display name for the prefix
                let mod_display = state.keycode_db
                    .get_mod_tap_display(&info.prefix)
                    .unwrap_or("MOD")
                    .to_string();
                let tap_display = Self::format_simple_keycode(&info.arg1);
                Some(TapHoldKeycode {
                    hold: mod_display,
                    tap: tap_display,
                    kind: TapHoldKind::ModTap,
                })
            }
            TapHoldType::LayerMod => {
                // LM(layer, mod) - Layer Mod
                let layer_display = Self::resolve_layer_display(&info.arg1, state);
                let mod_display = Self::format_modifier(info.arg2.as_deref().unwrap_or(""));
                Some(TapHoldKeycode {
                    hold: format!("L{layer_display}+"),
                    tap: mod_display,
                    kind: TapHoldKind::LayerMod,
                })
            }
            TapHoldType::SwapHands => {
                // SH_T(keycode) - Swap Hands Tap
                let tap_display = Self::format_simple_keycode(&info.arg1);
                Some(TapHoldKeycode {
                    hold: "SWAP".to_string(),
                    tap: tap_display,
                    kind: TapHoldKind::SwapHandsTap,
                })
            }
        }
    }
    
    /// Resolve layer reference to display string
    fn resolve_layer_display(layer_ref: &str, state: &AppState) -> String {
        // If it's a @uuid reference, find the layer number
        if let Some(uuid) = layer_ref.strip_prefix('@') {
            for (i, layer) in state.layout.layers.iter().enumerate() {
                if layer.id == uuid {
                    return i.to_string();
                }
            }
        }
        // Otherwise assume it's already a number or return as-is
        layer_ref.to_string()
    }
    
    /// Format modifier for compact display
    fn format_modifier(mod_str: &str) -> String {
        // Handle combined modifiers like "MOD_LCTL | MOD_LSFT"
        let mut result = String::new();
        
        if mod_str.contains("LCTL") || mod_str.contains("RCTL") {
            result.push('C');
        }
        if mod_str.contains("LSFT") || mod_str.contains("RSFT") {
            result.push('S');
        }
        if mod_str.contains("LALT") || mod_str.contains("RALT") {
            result.push('A');
        }
        if mod_str.contains("LGUI") || mod_str.contains("RGUI") {
            result.push('G');
        }
        
        if result.is_empty() {
            // Fallback: take first 3 chars
            mod_str.chars().take(3).collect()
        } else {
            result
        }
    }

    /// Format a simple keycode for compact display (removes KC_ prefix)
    fn format_simple_keycode(keycode: &str) -> String {
        // Remove KC_ prefix if present
        let display = keycode.strip_prefix("KC_").unwrap_or(keycode);
        display.to_string()
    }
    
    /// Truncate a string to a maximum length
    fn truncate(s: &str, max_len: usize) -> String {
        if s.len() <= max_len {
            s.to_string()
        } else {
            s.chars().take(max_len).collect()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_simple_keycode() {
        assert_eq!(KeyboardWidget::format_simple_keycode("KC_A"), "A");
        assert_eq!(KeyboardWidget::format_simple_keycode("KC_SPC"), "SPC");
        assert_eq!(KeyboardWidget::format_simple_keycode("KC_ENTER"), "ENTER");
        assert_eq!(KeyboardWidget::format_simple_keycode("MO(1)"), "MO(1)");
    }

    #[test]
    fn test_format_modifier() {
        assert_eq!(KeyboardWidget::format_modifier("MOD_LCTL"), "C");
        assert_eq!(KeyboardWidget::format_modifier("MOD_LSFT"), "S");
        assert_eq!(KeyboardWidget::format_modifier("MOD_LALT"), "A");
        assert_eq!(KeyboardWidget::format_modifier("MOD_LGUI"), "G");
        assert_eq!(KeyboardWidget::format_modifier("MOD_LCTL | MOD_LSFT"), "CS");
        assert_eq!(KeyboardWidget::format_modifier("MOD_LCTL | MOD_LSFT | MOD_LALT"), "CSA");
        assert_eq!(KeyboardWidget::format_modifier("MOD_LCTL | MOD_LSFT | MOD_LALT | MOD_LGUI"), "CSAG");
    }

    #[test]
    fn test_truncate() {
        assert_eq!(KeyboardWidget::truncate("ABC", 5), "ABC");
        assert_eq!(KeyboardWidget::truncate("ABCDEF", 5), "ABCDE");
        assert_eq!(KeyboardWidget::truncate("", 5), "");
    }
}
