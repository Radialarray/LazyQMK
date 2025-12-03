//! Key editor dialog for viewing and editing individual key properties.
//!
//! This dialog opens when pressing Enter on an already-assigned key (not KC_NO/KC_TRNS).
//! It provides:
//! - Visual display of current keycode with tap-hold breakdown
//! - Description field for documentation
//! - Quick access to reassign, color, and category actions

use crate::keycode_db::KeycodeDb;
use crate::models::{KeyDefinition, Position};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use super::{AppState, PopupType, ColorPickerContext};

/// Mode of the key editor dialog
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum KeyEditorMode {
    /// Viewing key properties (default)
    #[default]
    View,
    /// Editing the description field
    EditDescription,
}

/// State for the key editor dialog
#[derive(Debug, Clone)]
pub struct KeyEditorState {
    /// Position of the key being edited
    pub position: Position,
    /// Layer index of the key being edited
    pub layer_idx: usize,
    /// Current mode (View or EditDescription)
    pub mode: KeyEditorMode,
    /// Buffer for editing description
    pub description_buffer: String,
    /// Cursor position in description buffer
    pub cursor_position: usize,
    /// Original description (for cancel/restore)
    pub original_description: Option<String>,
}

impl Default for KeyEditorState {
    fn default() -> Self {
        Self::new()
    }
}

impl KeyEditorState {
    /// Create a new key editor state
    #[must_use]
    pub const fn new() -> Self {
        Self {
            position: Position { row: 0, col: 0 },
            layer_idx: 0,
            mode: KeyEditorMode::View,
            description_buffer: String::new(),
            cursor_position: 0,
            original_description: None,
        }
    }

    /// Initialize the editor with a specific key
    pub fn init_for_key(&mut self, key: &KeyDefinition, layer_idx: usize) {
        self.position = key.position;
        self.layer_idx = layer_idx;
        self.mode = KeyEditorMode::View;
        self.description_buffer = key.description.clone().unwrap_or_default();
        self.cursor_position = self.description_buffer.len();
        self.original_description = key.description.clone();
    }

    /// Start editing the description
    pub fn start_edit_description(&mut self) {
        self.mode = KeyEditorMode::EditDescription;
        self.cursor_position = self.description_buffer.len();
    }

    /// Cancel description editing and restore original
    pub fn cancel_edit_description(&mut self) {
        self.mode = KeyEditorMode::View;
        self.description_buffer = self.original_description.clone().unwrap_or_default();
        self.cursor_position = self.description_buffer.len();
    }

    /// Confirm description editing
    pub fn confirm_edit_description(&mut self) {
        self.mode = KeyEditorMode::View;
        self.original_description = if self.description_buffer.is_empty() {
            None
        } else {
            Some(self.description_buffer.clone())
        };
    }

    /// Get the edited description (None if empty)
    #[must_use]
    pub fn get_description(&self) -> Option<String> {
        if self.description_buffer.is_empty() {
            None
        } else {
            Some(self.description_buffer.clone())
        }
    }

    /// Handle character input in description edit mode
    pub fn handle_char(&mut self, c: char) {
        if self.mode == KeyEditorMode::EditDescription {
            self.description_buffer.insert(self.cursor_position, c);
            self.cursor_position += 1;
        }
    }

    /// Handle backspace in description edit mode
    pub fn handle_backspace(&mut self) {
        if self.mode == KeyEditorMode::EditDescription && self.cursor_position > 0 {
            self.cursor_position -= 1;
            self.description_buffer.remove(self.cursor_position);
        }
    }

    /// Handle delete in description edit mode
    pub fn handle_delete(&mut self) {
        if self.mode == KeyEditorMode::EditDescription
            && self.cursor_position < self.description_buffer.len()
        {
            self.description_buffer.remove(self.cursor_position);
        }
    }

    /// Move cursor left
    pub fn cursor_left(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
        }
    }

    /// Move cursor right
    pub fn cursor_right(&mut self) {
        if self.cursor_position < self.description_buffer.len() {
            self.cursor_position += 1;
        }
    }

    /// Move cursor to start
    pub fn cursor_home(&mut self) {
        self.cursor_position = 0;
    }

    /// Move cursor to end
    pub fn cursor_end(&mut self) {
        self.cursor_position = self.description_buffer.len();
    }

    /// Check if we're in edit mode
    #[must_use]
    pub const fn is_editing(&self) -> bool {
        matches!(self.mode, KeyEditorMode::EditDescription)
    }

    /// Reset the editor state
    pub fn reset(&mut self) {
        *self = Self::new();
    }
}

/// Parsed keycode information from the database
#[derive(Debug, Clone)]
pub struct ParsedKeycode {
    /// The category of the keycode (mod_tap, mod_combo, layers, etc.)
    pub category: String,
    /// Human-readable name from the database
    pub name: String,
    /// Optional description from the database
    pub description: Option<String>,
    /// Extracted parameters (if any)
    pub params: Vec<String>,
}

/// Parse a keycode using the keycode database.
/// Returns structured information about the keycode if it matches a database pattern.
#[must_use]
pub fn parse_keycode_with_db(db: &KeycodeDb, keycode: &str) -> Option<ParsedKeycode> {
    // First, try direct lookup (for simple keycodes)
    if let Some(def) = db.get(keycode) {
        return Some(ParsedKeycode {
            category: def.category.clone(),
            name: def.name.clone(),
            description: def.description.clone(),
            params: vec![],
        });
    }

    // For parameterized keycodes, extract prefix and look up template
    if let Some(paren_pos) = keycode.find('(') {
        let prefix = &keycode[..paren_pos];
        let template = format!("{}()", prefix);
        
        if let Some(def) = db.get(&template) {
            // Extract parameters from the keycode
            let inner = &keycode[paren_pos + 1..keycode.len() - 1];
            let params: Vec<String> = inner
                .split(',')
                .map(|s| s.trim().to_string())
                .collect();

            return Some(ParsedKeycode {
                category: def.category.clone(),
                name: def.name.clone(),
                description: def.description.clone(),
                params,
            });
        }
    }

    None
}

/// Parse a keycode and return display information for the key editor.
/// Returns (line1_label, line1_value, line2_label, line2_value) for display.
#[must_use]
pub fn get_keycode_breakdown(db: &KeycodeDb, keycode: &str) -> Option<(String, String, String, String)> {
    let parsed = parse_keycode_with_db(db, keycode)?;
    
    match parsed.category.as_str() {
        "mod_tap" => {
            // Mod-tap: Hold for modifier, tap for keycode
            let hold_desc = parsed.name.trim_end_matches("-Tap").to_string();
            let tap_key = parsed.params.first().cloned().unwrap_or_default();
            // For MT() which has 2 params (modifier, keycode)
            let tap_key = if parsed.params.len() == 2 {
                parsed.params.get(1).cloned().unwrap_or_default()
            } else {
                tap_key
            };
            Some(("Hold".to_string(), hold_desc, "Tap".to_string(), tap_key))
        }
        "mod_combo" => {
            // Modifier wrapper: Modifier + base key
            let mod_desc = parsed.name.clone();
            let base_key = parsed.params.first().cloned().unwrap_or_default();
            Some(("Modifier".to_string(), mod_desc, "Key".to_string(), base_key))
        }
        "layers" => {
            match keycode.split('(').next()? {
                "LT" => {
                    // Layer-Tap: Hold for layer, tap for keycode
                    let layer = parsed.params.first().cloned().unwrap_or_default();
                    let tap_key = parsed.params.get(1).cloned().unwrap_or_default();
                    Some(("Hold".to_string(), format!("Layer {}", layer), "Tap".to_string(), tap_key))
                }
                "LM" => {
                    // Layer-Mod: Layer + modifier
                    let layer = parsed.params.first().cloned().unwrap_or_default();
                    let mods = parsed.params.get(1).cloned().unwrap_or_default();
                    Some(("Layer".to_string(), layer, "Modifier".to_string(), mods))
                }
                _ => None, // MO, TG, TO etc. are simple layer switches, no breakdown needed
            }
        }
        _ => None,
    }
}

/// Check if a keycode is "assigned" (not empty or transparent)
#[must_use]
pub fn is_key_assigned(keycode: &str) -> bool {
    !matches!(keycode, "KC_NO" | "KC_TRNS" | "XXXXXXX" | "_______" | "")
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

/// Render the key editor dialog
pub fn render_key_editor(f: &mut Frame, state: &AppState) {
    let area = centered_rect(60, 50, f.size());
    let theme = &state.theme;
    let editor_state = &state.key_editor_state;

    // Get the key being edited
    let key = state.layout.layers
        .get(editor_state.layer_idx)
        .and_then(|layer| layer.keys.iter().find(|k| k.position == editor_state.position));

    let Some(key) = key else {
        return;
    };

    // Clear the background
    f.render_widget(Clear, area);

    // Render opaque background
    let background = Block::default()
        .style(Style::default().bg(theme.background));
    f.render_widget(background, area);

    // Main container with title
    let title = format!(
        " Key Editor - Layer {} ({}, {}) ",
        editor_state.layer_idx,
        editor_state.position.row,
        editor_state.position.col
    );

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.primary));

    let inner = block.inner(area);
    f.render_widget(block, area);

    // Layout sections
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Keycode display
            Constraint::Length(4), // Tap-hold breakdown (if applicable)
            Constraint::Length(4), // Description
            Constraint::Min(2),    // Spacer
            Constraint::Length(2), // Actions
        ])
        .split(inner);

    // Keycode display - get color from the key
    let keycode_color = state.layout.resolve_key_color(editor_state.layer_idx, key);
    let keycode_style = Style::default()
        .fg(Color::Rgb(keycode_color.r, keycode_color.g, keycode_color.b))
        .add_modifier(Modifier::BOLD);

    let keycode_display = Paragraph::new(vec![
        Line::from(vec![
            Span::styled("Keycode: ", Style::default().fg(theme.text_muted)),
            Span::styled(&key.keycode, keycode_style),
        ]),
    ])
    .block(Block::default().borders(Borders::BOTTOM).border_style(Style::default().fg(theme.inactive)));
    f.render_widget(keycode_display, chunks[0]);

    // Keycode breakdown using the database
    let tap_hold_content = if let Some((label1, val1, label2, val2)) = get_keycode_breakdown(&state.keycode_db, &key.keycode) {
        vec![
            Line::from(vec![
                Span::styled(format!("{}: ", label1), Style::default().fg(theme.text_muted)),
                Span::styled(val1, Style::default().fg(theme.accent)),
            ]),
            Line::from(vec![
                Span::styled(format!("{}: ", label2), Style::default().fg(theme.text_muted)),
                Span::styled(val2, Style::default().fg(theme.success)),
            ]),
        ]
    } else {
        // Simple keycode, no breakdown needed
        vec![
            Line::from(Span::styled(
                "(Simple keycode - no hold/tap or modifier)",
                Style::default().fg(theme.text_muted),
            )),
        ]
    };

    let tap_hold_display = Paragraph::new(tap_hold_content)
        .block(Block::default().borders(Borders::BOTTOM).border_style(Style::default().fg(theme.inactive)));
    f.render_widget(tap_hold_display, chunks[1]);

    // Description field
    let desc_style = if editor_state.is_editing() {
        Style::default().fg(theme.accent)
    } else {
        Style::default().fg(theme.text)
    };

    let desc_content = if editor_state.is_editing() {
        // Show cursor in edit mode
        let before_cursor = &editor_state.description_buffer[..editor_state.cursor_position];
        let after_cursor = &editor_state.description_buffer[editor_state.cursor_position..];
        format!("{}█{}", before_cursor, after_cursor)
    } else if editor_state.description_buffer.is_empty() {
        "(No description - press D to add)".to_string()
    } else {
        editor_state.description_buffer.clone()
    };

    let desc_title = if editor_state.is_editing() {
        " Description (editing) "
    } else {
        " Description "
    };

    let description_display = Paragraph::new(desc_content)
        .style(desc_style)
        .block(
            Block::default()
                .title(desc_title)
                .borders(Borders::ALL)
                .border_style(if editor_state.is_editing() {
                    Style::default().fg(theme.accent)
                } else {
                    Style::default().fg(theme.inactive)
                }),
        );
    f.render_widget(description_display, chunks[2]);

    // Actions bar
    let actions = if editor_state.is_editing() {
        Line::from(vec![
            Span::styled("Enter", Style::default().fg(theme.success).add_modifier(Modifier::BOLD)),
            Span::styled(": Save  ", Style::default().fg(theme.text_muted)),
            Span::styled("Esc", Style::default().fg(theme.warning).add_modifier(Modifier::BOLD)),
            Span::styled(": Cancel", Style::default().fg(theme.text_muted)),
        ])
    } else {
        Line::from(vec![
            Span::styled("Enter", Style::default().fg(theme.success).add_modifier(Modifier::BOLD)),
            Span::styled(": Reassign  ", Style::default().fg(theme.text_muted)),
            Span::styled("D", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
            Span::styled(": Description  ", Style::default().fg(theme.text_muted)),
            Span::styled("C", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
            Span::styled(": Color  ", Style::default().fg(theme.text_muted)),
            Span::styled("Esc", Style::default().fg(theme.warning).add_modifier(Modifier::BOLD)),
            Span::styled(": Close", Style::default().fg(theme.text_muted)),
        ])
    };

    let actions_bar = Paragraph::new(actions);
    f.render_widget(actions_bar, chunks[4]);
}

/// Handle input for the key editor
pub fn handle_input(
    state: &mut AppState,
    key: crossterm::event::KeyEvent,
) -> anyhow::Result<bool> {
    use crossterm::event::KeyCode;

    // Check if we're in edit mode first (immutable borrow)
    let is_editing = state.key_editor_state.is_editing();

    if is_editing {
        // Description edit mode
        match key.code {
            KeyCode::Esc => {
                state.key_editor_state.cancel_edit_description();
                state.set_status("Description edit cancelled");
            }
            KeyCode::Enter => {
                state.key_editor_state.confirm_edit_description();
                // Apply the description to the actual key
                let description = state.key_editor_state.get_description();
                if let Some(key) = state.get_selected_key_mut() {
                    key.description = description;
                    state.mark_dirty();
                    state.set_status("Description saved");
                }
            }
            KeyCode::Backspace => {
                state.key_editor_state.handle_backspace();
            }
            KeyCode::Delete => {
                state.key_editor_state.handle_delete();
            }
            KeyCode::Left => {
                state.key_editor_state.cursor_left();
            }
            KeyCode::Right => {
                state.key_editor_state.cursor_right();
            }
            KeyCode::Home => {
                state.key_editor_state.cursor_home();
            }
            KeyCode::End => {
                state.key_editor_state.cursor_end();
            }
            KeyCode::Char(c) => {
                state.key_editor_state.handle_char(c);
            }
            _ => {}
        }
    } else {
        // View mode
        match key.code {
            KeyCode::Esc => {
                // Close the editor
                state.active_popup = None;
                state.key_editor_state.reset();
                state.set_status("Key editor closed");
            }
            KeyCode::Enter => {
                // Open keycode picker to reassign
                state.active_popup = Some(PopupType::KeycodePicker);
                state.set_status("Select new keycode");
            }
            KeyCode::Char('d' | 'D') => {
                // Start editing description
                state.key_editor_state.start_edit_description();
                state.set_status("Editing description - Enter to save, Esc to cancel");
            }
            KeyCode::Char('c' | 'C') => {
                // Open color picker
                state.color_picker_context = Some(ColorPickerContext::IndividualKey);
                state.active_popup = Some(PopupType::ColorPicker);
                state.set_status("Select color for key");
            }
            _ => {}
        }
    }

    Ok(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_key_assigned() {
        assert!(!is_key_assigned("KC_NO"));
        assert!(!is_key_assigned("KC_TRNS"));
        assert!(!is_key_assigned("XXXXXXX"));
        assert!(!is_key_assigned("_______"));
        assert!(!is_key_assigned(""));
        assert!(is_key_assigned("KC_A"));
        assert!(is_key_assigned("LT(1, KC_SPC)"));
    }

    #[test]
    fn test_parse_keycode_with_db_simple() {
        let db = KeycodeDb::load().expect("Failed to load keycode database");
        
        let result = parse_keycode_with_db(&db, "KC_A");
        assert!(result.is_some());
        let parsed = result.unwrap();
        assert_eq!(parsed.category, "basic");
        assert_eq!(parsed.name, "A");
    }

    #[test]
    fn test_parse_keycode_with_db_mod_combo() {
        let db = KeycodeDb::load().expect("Failed to load keycode database");
        
        let result = parse_keycode_with_db(&db, "LCG(KC_Q)");
        assert!(result.is_some());
        let parsed = result.unwrap();
        assert_eq!(parsed.category, "mod_combo");
        assert_eq!(parsed.name, "Ctrl+GUI+");
        assert_eq!(parsed.params, vec!["KC_Q"]);
    }

    #[test]
    fn test_parse_keycode_with_db_mod_tap() {
        let db = KeycodeDb::load().expect("Failed to load keycode database");
        
        let result = parse_keycode_with_db(&db, "LCTL_T(KC_A)");
        assert!(result.is_some());
        let parsed = result.unwrap();
        assert_eq!(parsed.category, "mod_tap");
        assert_eq!(parsed.name, "Ctrl-Tap");
        assert_eq!(parsed.params, vec!["KC_A"]);
    }

    #[test]
    fn test_parse_keycode_with_db_layer_tap() {
        let db = KeycodeDb::load().expect("Failed to load keycode database");
        
        let result = parse_keycode_with_db(&db, "LT(1, KC_SPC)");
        assert!(result.is_some());
        let parsed = result.unwrap();
        assert_eq!(parsed.category, "layers");
        assert_eq!(parsed.name, "Layer-Tap");
        assert_eq!(parsed.params, vec!["1", "KC_SPC"]);
    }

    #[test]
    fn test_get_keycode_breakdown_mod_tap() {
        let db = KeycodeDb::load().expect("Failed to load keycode database");
        
        let result = get_keycode_breakdown(&db, "LCTL_T(KC_A)");
        assert!(result.is_some());
        let (label1, val1, label2, val2) = result.unwrap();
        assert_eq!(label1, "Hold");
        assert_eq!(val1, "Ctrl");
        assert_eq!(label2, "Tap");
        assert_eq!(val2, "KC_A");
    }

    #[test]
    fn test_get_keycode_breakdown_mod_combo() {
        let db = KeycodeDb::load().expect("Failed to load keycode database");
        
        let result = get_keycode_breakdown(&db, "LCG(KC_Q)");
        assert!(result.is_some());
        let (label1, val1, label2, val2) = result.unwrap();
        assert_eq!(label1, "Modifier");
        assert_eq!(val1, "Ctrl+GUI+");
        assert_eq!(label2, "Key");
        assert_eq!(val2, "KC_Q");
    }

    #[test]
    fn test_get_keycode_breakdown_layer_tap() {
        let db = KeycodeDb::load().expect("Failed to load keycode database");
        
        let result = get_keycode_breakdown(&db, "LT(2, KC_SPC)");
        assert!(result.is_some());
        let (label1, val1, label2, val2) = result.unwrap();
        assert_eq!(label1, "Hold");
        assert_eq!(val1, "Layer 2");
        assert_eq!(label2, "Tap");
        assert_eq!(val2, "KC_SPC");
    }

    #[test]
    fn test_get_keycode_breakdown_simple_keycode() {
        let db = KeycodeDb::load().expect("Failed to load keycode database");
        
        // Simple keycodes should return None (no breakdown needed)
        let result = get_keycode_breakdown(&db, "KC_A");
        assert!(result.is_none());
    }

    #[test]
    fn test_get_keycode_breakdown_layer_mod() {
        let db = KeycodeDb::load().expect("Failed to load keycode database");
        
        let result = get_keycode_breakdown(&db, "LM(1, MOD_LSFT)");
        assert!(result.is_some());
        let (label1, val1, label2, val2) = result.unwrap();
        assert_eq!(label1, "Layer");
        assert_eq!(val1, "1");
        assert_eq!(label2, "Modifier");
        assert_eq!(val2, "MOD_LSFT");
    }

    #[test]
    fn test_get_keycode_breakdown_custom_mod_tap() {
        let db = KeycodeDb::load().expect("Failed to load keycode database");
        
        let result = get_keycode_breakdown(&db, "MT(MOD_LCTL, KC_A)");
        assert!(result.is_some());
        let (label1, val1, label2, val2) = result.unwrap();
        assert_eq!(label1, "Hold");
        // MT() is named "Mod-Tap (Custom)" in the DB, trim_end_matches("-Tap") doesn't apply
        assert_eq!(val1, "Mod-Tap (Custom)");
        assert_eq!(label2, "Tap");
        assert_eq!(val2, "KC_A");
    }
}
