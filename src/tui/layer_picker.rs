//! Layer picker dialog for selecting a layer for layer-switching keycodes.
//!
//! When a user selects a layer keycode (MO, TG, TO, etc.), this dialog
//! allows them to pick which layer the keycode should reference.
//! The selected layer is stored as a UUID reference (@layer_id) rather
//! than a numeric index, making it stable across layer reordering.

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
    Frame,
};

use crate::models::Layer;

use super::Theme;

/// Layer picker state
#[derive(Debug, Clone)]
pub struct LayerPickerState {
    /// Currently selected layer index
    pub selected: usize,
    /// The keycode prefix (e.g., "MO", "TG", "TO")
    pub keycode_prefix: String,
    /// Extra parameter for compound keycodes (e.g., "KC_SPC" for LT)
    pub extra_param: Option<String>,
}

impl Default for LayerPickerState {
    fn default() -> Self {
        Self::new()
    }
}

impl LayerPickerState {
    /// Creates a new layer picker state
    #[must_use]
    pub const fn new() -> Self {
        Self {
            selected: 0,
            keycode_prefix: String::new(),
            extra_param: None,
        }
    }

    /// Initialize with a keycode prefix
    pub fn with_prefix(prefix: impl Into<String>) -> Self {
        Self {
            selected: 0,
            keycode_prefix: prefix.into(),
            extra_param: None,
        }
    }

    /// Initialize with a keycode prefix and extra parameter (for LT, LM)
    pub fn with_prefix_and_extra(prefix: impl Into<String>, extra: impl Into<String>) -> Self {
        Self {
            selected: 0,
            keycode_prefix: prefix.into(),
            extra_param: Some(extra.into()),
        }
    }

    /// Reset to initial state
    pub fn reset(&mut self) {
        self.selected = 0;
        self.keycode_prefix.clear();
        self.extra_param = None;
    }

    /// Move selection up
    pub fn select_previous(&mut self, layer_count: usize) {
        if layer_count > 0 {
            if self.selected > 0 {
                self.selected -= 1;
            } else {
                self.selected = layer_count - 1;
            }
        }
    }

    /// Move selection down
    pub fn select_next(&mut self, layer_count: usize) {
        if layer_count > 0 {
            self.selected = (self.selected + 1) % layer_count;
        }
    }

    /// Build the keycode string for the selected layer
    #[must_use]
    pub fn build_keycode(&self, layer: &Layer) -> String {
        let layer_ref = format!("@{}", layer.id);
        match &self.extra_param {
            Some(extra) => format!("{}({}, {})", self.keycode_prefix, layer_ref, extra),
            None => format!("{}({})", self.keycode_prefix, layer_ref),
        }
    }
}

/// Render the layer picker popup
pub fn render_layer_picker(f: &mut Frame, state: &LayerPickerState, layers: &[Layer], theme: &Theme) {
    let area = centered_rect(50, 60, f.size());

    // Clear the background area first
    f.render_widget(Clear, area);

    // Render opaque background with theme color
    let background = Block::default()
        .style(Style::default().bg(theme.background));
    f.render_widget(background, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(5),    // Layer list
            Constraint::Length(3), // Help text
        ])
        .split(area);

    // Title with keycode preview
    let preview = if let Some(layer) = layers.get(state.selected) {
        state.build_keycode(layer)
    } else {
        format!("{}(?)", state.keycode_prefix)
    };
    
    let title = format!(" Select Layer for {} ", state.keycode_prefix);
    let title_block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .style(Style::default().bg(theme.background));
    
    let preview_text = Paragraph::new(format!("Preview: {}", preview))
        .block(title_block)
        .style(Style::default().fg(theme.text));
    f.render_widget(preview_text, chunks[0]);

    // Build list items
    let list_items: Vec<ListItem> = layers
        .iter()
        .enumerate()
        .map(|(idx, layer)| {
            let content = Line::from(vec![
                Span::styled(
                    format!("Layer {}: ", idx),
                    Style::default().fg(theme.text_muted),
                ),
                Span::styled(
                    &layer.name,
                    Style::default().fg(theme.text),
                ),
            ]);
            ListItem::new(content)
        })
        .collect();

    // Create list widget with stateful selection
    let list = List::new(list_items)
        .block(
            Block::default()
                .title(format!(" Layers ({}) ", layers.len()))
                .borders(Borders::ALL)
                .style(Style::default().bg(theme.background)),
        )
        .highlight_style(
            Style::default()
                .bg(theme.surface)
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    // Create list state for highlighting
    let mut list_state = ListState::default();
    list_state.select(Some(state.selected.min(layers.len().saturating_sub(1))));

    f.render_stateful_widget(list, chunks[1], &mut list_state);

    // Help text
    let help = Paragraph::new("^|v: Navigate | Enter: Select | Esc: Cancel")
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().bg(theme.background)),
        )
        .style(Style::default().fg(theme.text_muted));
    f.render_widget(help, chunks[2]);
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

/// Layer keycode types with their descriptions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayerKeycodeType {
    /// MO(layer) - Momentary layer activation (held)
    Momentary,
    /// TG(layer) - Toggle layer on/off
    Toggle,
    /// TO(layer) - Switch to layer
    SwitchTo,
    /// DF(layer) - Set default layer
    Default,
    /// OSL(layer) - One-shot layer
    OneShot,
    /// TT(layer) - Tap-toggle layer
    TapToggle,
    /// LT(layer, kc) - Layer-tap
    LayerTap,
    /// LM(layer, mod) - Layer-mod
    LayerMod,
}

impl LayerKeycodeType {
    /// Get the keycode prefix for this type
    #[must_use]
    pub const fn prefix(&self) -> &'static str {
        match self {
            Self::Momentary => "MO",
            Self::Toggle => "TG",
            Self::SwitchTo => "TO",
            Self::Default => "DF",
            Self::OneShot => "OSL",
            Self::TapToggle => "TT",
            Self::LayerTap => "LT",
            Self::LayerMod => "LM",
        }
    }

    /// Get a description of what this keycode does
    #[must_use]
    pub const fn description(&self) -> &'static str {
        match self {
            Self::Momentary => "Activate layer while held",
            Self::Toggle => "Toggle layer on/off",
            Self::SwitchTo => "Switch to layer",
            Self::Default => "Set as default layer",
            Self::OneShot => "Activate layer for next keypress",
            Self::TapToggle => "Tap to toggle, hold to activate",
            Self::LayerTap => "Tap for keycode, hold for layer",
            Self::LayerMod => "Activate layer with modifier",
        }
    }

    /// Check if this keycode type requires an extra parameter
    #[must_use]
    pub const fn requires_extra_param(&self) -> bool {
        matches!(self, Self::LayerTap | Self::LayerMod)
    }

    /// Parse a keycode string to determine if it's a layer keycode
    #[must_use]
    pub fn from_keycode(keycode: &str) -> Option<Self> {
        if keycode.starts_with("MO(") {
            Some(Self::Momentary)
        } else if keycode.starts_with("TG(") {
            Some(Self::Toggle)
        } else if keycode.starts_with("TO(") {
            Some(Self::SwitchTo)
        } else if keycode.starts_with("DF(") {
            Some(Self::Default)
        } else if keycode.starts_with("OSL(") {
            Some(Self::OneShot)
        } else if keycode.starts_with("TT(") {
            Some(Self::TapToggle)
        } else if keycode.starts_with("LT(") {
            Some(Self::LayerTap)
        } else if keycode.starts_with("LM(") {
            Some(Self::LayerMod)
        } else {
            None
        }
    }

    /// Get all simple layer keycode types (no extra parameter needed)
    #[must_use]
    pub const fn simple_types() -> &'static [Self] {
        &[
            Self::Momentary,
            Self::Toggle,
            Self::SwitchTo,
            Self::Default,
            Self::OneShot,
            Self::TapToggle,
        ]
    }

    /// Get all layer keycode types
    #[must_use]
    pub const fn all() -> &'static [Self] {
        &[
            Self::Momentary,
            Self::Toggle,
            Self::SwitchTo,
            Self::Default,
            Self::OneShot,
            Self::TapToggle,
            Self::LayerTap,
            Self::LayerMod,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::RgbColor;

    fn create_test_layers() -> Vec<Layer> {
        vec![
            Layer::new(0, "Base", RgbColor::new(255, 255, 255)).unwrap(),
            Layer::new(1, "Navigation", RgbColor::new(0, 255, 0)).unwrap(),
            Layer::new(2, "Symbols", RgbColor::new(255, 0, 0)).unwrap(),
        ]
    }

    #[test]
    fn test_layer_picker_state_new() {
        let state = LayerPickerState::new();
        assert_eq!(state.selected, 0);
        assert!(state.keycode_prefix.is_empty());
        assert!(state.extra_param.is_none());
    }

    #[test]
    fn test_layer_picker_with_prefix() {
        let state = LayerPickerState::with_prefix("MO");
        assert_eq!(state.keycode_prefix, "MO");
        assert!(state.extra_param.is_none());
    }

    #[test]
    fn test_layer_picker_with_prefix_and_extra() {
        let state = LayerPickerState::with_prefix_and_extra("LT", "KC_SPC");
        assert_eq!(state.keycode_prefix, "LT");
        assert_eq!(state.extra_param.as_deref(), Some("KC_SPC"));
    }

    #[test]
    fn test_build_keycode_simple() {
        let layers = create_test_layers();
        let mut state = LayerPickerState::with_prefix("MO");
        state.selected = 1;

        let keycode = state.build_keycode(&layers[1]);
        assert!(keycode.starts_with("MO(@"));
        assert!(keycode.ends_with(')'));
        assert!(keycode.contains(&layers[1].id));
    }

    #[test]
    fn test_build_keycode_with_extra() {
        let layers = create_test_layers();
        let mut state = LayerPickerState::with_prefix_and_extra("LT", "KC_SPC");
        state.selected = 2;

        let keycode = state.build_keycode(&layers[2]);
        assert!(keycode.starts_with("LT(@"));
        assert!(keycode.contains(", KC_SPC)"));
        assert!(keycode.contains(&layers[2].id));
    }

    #[test]
    fn test_navigation() {
        let mut state = LayerPickerState::new();
        let layer_count = 3;

        // Navigate down
        state.select_next(layer_count);
        assert_eq!(state.selected, 1);

        state.select_next(layer_count);
        assert_eq!(state.selected, 2);

        // Wrap around
        state.select_next(layer_count);
        assert_eq!(state.selected, 0);

        // Navigate up (wrap)
        state.select_previous(layer_count);
        assert_eq!(state.selected, 2);
    }

    #[test]
    fn test_layer_keycode_type_from_keycode() {
        assert_eq!(
            LayerKeycodeType::from_keycode("MO(1)"),
            Some(LayerKeycodeType::Momentary)
        );
        assert_eq!(
            LayerKeycodeType::from_keycode("TG(@abc-123)"),
            Some(LayerKeycodeType::Toggle)
        );
        assert_eq!(
            LayerKeycodeType::from_keycode("LT(@layer-id, KC_SPC)"),
            Some(LayerKeycodeType::LayerTap)
        );
        assert_eq!(LayerKeycodeType::from_keycode("KC_A"), None);
    }

    #[test]
    fn test_layer_keycode_type_requires_extra() {
        assert!(!LayerKeycodeType::Momentary.requires_extra_param());
        assert!(!LayerKeycodeType::Toggle.requires_extra_param());
        assert!(LayerKeycodeType::LayerTap.requires_extra_param());
        assert!(LayerKeycodeType::LayerMod.requires_extra_param());
    }
}
