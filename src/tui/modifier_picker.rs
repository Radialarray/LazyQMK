//! Modifier picker dialog for selecting QMK modifiers
//!
//! Used for `MT()` and `LM()` keycodes that require modifier selection.
//!
//! This module implements the Component trait: `ModifierPicker` for self-contained UI components

// Navigation uses separate match arms for left/right columns for clarity
#![allow(clippy::match_same_arms)]
// Allow small types passed by reference for API consistency
#![allow(clippy::trivially_copy_pass_by_ref)]

use crossterm::event::KeyEvent;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier as StyleModifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use super::component::Component;
use super::theme::Theme;

/// QMK modifier bit flags
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum QmkModifier {
    /// Left Control
    LCtrl = 0b0000_0001,
    /// Left Shift
    LShift = 0b0000_0010,
    /// Left Alt/Option
    LAlt = 0b0000_0100,
    /// Left GUI/Command/Windows
    LGui = 0b0000_1000,
    /// Right Control
    RCtrl = 0b0001_0000,
    /// Right Shift
    RShift = 0b0010_0000,
    /// Right Alt/Option
    RAlt = 0b0100_0000,
    /// Right GUI/Command/Windows
    RGui = 0b1000_0000,
}

impl QmkModifier {
    /// All modifiers in display order (left side first, then right side)
    pub const ALL: [Self; 8] = [
        Self::LCtrl,
        Self::LShift,
        Self::LAlt,
        Self::LGui,
        Self::RCtrl,
        Self::RShift,
        Self::RAlt,
        Self::RGui,
    ];

    /// Get the QMK macro name for this modifier
    #[must_use]
    pub const fn qmk_name(&self) -> &'static str {
        match self {
            Self::LCtrl => "MOD_LCTL",
            Self::LShift => "MOD_LSFT",
            Self::LAlt => "MOD_LALT",
            Self::LGui => "MOD_LGUI",
            Self::RCtrl => "MOD_RCTL",
            Self::RShift => "MOD_RSFT",
            Self::RAlt => "MOD_RALT",
            Self::RGui => "MOD_RGUI",
        }
    }

    /// Get display name
    #[must_use]
    pub const fn display_name(&self) -> &'static str {
        match self {
            Self::LCtrl | Self::RCtrl => "Ctrl",
            Self::LShift | Self::RShift => "Shift",
            Self::LAlt | Self::RAlt => "Alt",
            Self::LGui | Self::RGui => "GUI",
        }
    }

    /// Check if this is a left-side modifier.
    /// Note: Kept for API completeness - useful for modifier display logic.
    #[allow(dead_code)]
    #[must_use]
    pub const fn is_left(&self) -> bool {
        matches!(self, Self::LCtrl | Self::LShift | Self::LAlt | Self::LGui)
    }
}

/// Preset modifier combinations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModifierPreset {
    /// Ctrl+Shift+Alt (Meh)
    Meh,
    /// Ctrl+Shift+Alt+GUI (Hyper)
    Hyper,
}

impl ModifierPreset {
    /// Get the modifier bits for this preset
    #[must_use]
    pub const fn bits(&self) -> u8 {
        match self {
            Self::Meh => {
                QmkModifier::LCtrl as u8 | QmkModifier::LShift as u8 | QmkModifier::LAlt as u8
            }
            Self::Hyper => {
                QmkModifier::LCtrl as u8
                    | QmkModifier::LShift as u8
                    | QmkModifier::LAlt as u8
                    | QmkModifier::LGui as u8
            }
        }
    }

    /// Get display name
    #[must_use]
    pub const fn display_name(&self) -> &'static str {
        match self {
            Self::Meh => "Meh (C+S+A)",
            Self::Hyper => "Hyper (C+S+A+G)",
        }
    }
}

/// State for the modifier picker
#[derive(Debug, Clone)]
pub struct ModifierPickerState {
    /// Bitfield of selected modifiers
    pub selected_mods: u8,
    /// Current focus position (0-7 for individual mods, 8-9 for presets)
    pub focus: usize,
}

impl Default for ModifierPickerState {
    fn default() -> Self {
        Self::new()
    }
}

impl ModifierPickerState {
    /// Create a new modifier picker state
    #[must_use]
    pub const fn new() -> Self {
        Self {
            selected_mods: 0,
            focus: 0,
        }
    }

    /// Toggle a modifier by its bit value
    pub const fn toggle_mod(&mut self, mod_bit: u8) {
        self.selected_mods ^= mod_bit;
    }

    /// Check if a modifier is selected
    #[must_use]
    pub const fn is_selected(&self, mod_bit: u8) -> bool {
        (self.selected_mods & mod_bit) != 0
    }

    /// Toggle the currently focused item
    pub fn toggle_focused(&mut self) {
        if self.focus < 8 {
            // Individual modifier
            let modifier = QmkModifier::ALL[self.focus];
            self.toggle_mod(modifier as u8);
        } else if self.focus == 8 {
            // Meh preset
            if self.selected_mods == ModifierPreset::Meh.bits() {
                self.selected_mods = 0;
            } else {
                self.selected_mods = ModifierPreset::Meh.bits();
            }
        } else if self.focus == 9 {
            // Hyper preset
            if self.selected_mods == ModifierPreset::Hyper.bits() {
                self.selected_mods = 0;
            } else {
                self.selected_mods = ModifierPreset::Hyper.bits();
            }
        }
    }

    /// Move focus up
    pub const fn focus_up(&mut self) {
        // Layout:
        // 0-3: left column (LCtrl, LShift, LAlt, LGui)
        // 4-7: right column (RCtrl, RShift, RAlt, RGui)
        // 8-9: presets row (Meh, Hyper)
        match self.focus {
            0 => self.focus = 8, // LCtrl -> Meh
            1..=3 => self.focus -= 1,
            4 => self.focus = 9, // RCtrl -> Hyper
            5..=7 => self.focus -= 1,
            8 => self.focus = 3, // Meh -> LGui
            9 => self.focus = 7, // Hyper -> RGui
            _ => {}
        }
    }

    /// Move focus down
    pub const fn focus_down(&mut self) {
        match self.focus {
            0..=2 => self.focus += 1,
            3 => self.focus = 8, // LGui -> Meh
            4..=6 => self.focus += 1,
            7 => self.focus = 9, // RGui -> Hyper
            8 => self.focus = 0, // Meh -> LCtrl
            9 => self.focus = 4, // Hyper -> RCtrl
            _ => {}
        }
    }

    /// Move focus left
    pub const fn focus_left(&mut self) {
        match self.focus {
            4..=7 => self.focus -= 4, // Right column -> Left column
            9 => self.focus = 8,      // Hyper -> Meh
            _ => {}
        }
    }

    /// Move focus right
    pub const fn focus_right(&mut self) {
        match self.focus {
            0..=3 => self.focus += 4, // Left column -> Right column
            8 => self.focus = 9,      // Meh -> Hyper
            _ => {}
        }
    }

    /// Convert selected modifiers to QMK modifier string
    /// e.g., "`MOD_LCTL` | `MOD_LSFT`"
    #[must_use]
    pub fn to_mod_string(&self) -> String {
        if self.selected_mods == 0 {
            return String::new();
        }

        let mut parts = Vec::new();
        for modifier in &QmkModifier::ALL {
            if self.is_selected(*modifier as u8) {
                parts.push(modifier.qmk_name());
            }
        }

        parts.join(" | ")
    }

    /// Check if any modifier is selected
    #[must_use]
    pub const fn has_selection(&self) -> bool {
        self.selected_mods != 0
    }
}

/// Events emitted by the ModifierPicker component
#[derive(Debug, Clone)]
pub enum ModifierPickerEvent {
    /// User selected modifiers and wants to apply them
    ModifiersSelected(Vec<String>),
    /// User cancelled without making changes
    Cancelled,
}

/// ModifierPicker component that implements the Component trait
#[derive(Debug, Clone)]
pub struct ModifierPicker {
    /// Internal state of the modifier picker
    state: ModifierPickerState,
}

impl ModifierPicker {
    /// Create a new ModifierPicker with empty selection
    #[must_use]
    pub fn new() -> Self {
        Self {
            state: ModifierPickerState::new(),
        }
    }

    /// Create a new ModifierPicker initialized with specific modifiers
    #[must_use]
    #[allow(dead_code)]
    pub fn with_modifiers(mod_bits: u8) -> Self {
        Self {
            state: ModifierPickerState {
                selected_mods: mod_bits,
                focus: 0,
            },
        }
    }

    /// Get the internal state (for legacy rendering)
    #[allow(dead_code)]
    pub fn state(&self) -> &ModifierPickerState {
        &self.state
    }

    /// Get mutable access to state (for legacy rendering)
    #[allow(dead_code)]
    pub fn state_mut(&mut self) -> &mut ModifierPickerState {
        &mut self.state
    }

    /// Get the selected modifiers as a QMK modifier string
    #[must_use]
    #[allow(dead_code)]
    pub fn get_mod_string(&self) -> String {
        self.state.to_mod_string()
    }

    /// Get the selected modifiers as individual strings
    fn get_modifiers_list(&self) -> Vec<String> {
        if self.state.selected_mods == 0 {
            return Vec::new();
        }

        let mut parts = Vec::new();
        for modifier in &QmkModifier::ALL {
            if self.state.is_selected(*modifier as u8) {
                parts.push(modifier.qmk_name().to_string());
            }
        }

        parts
    }
}

impl Default for ModifierPicker {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for ModifierPicker {
    type Event = ModifierPickerEvent;

    fn handle_input(&mut self, key: KeyEvent) -> Option<Self::Event> {
        use crossterm::event::KeyCode;

        match key.code {
            KeyCode::Esc => Some(ModifierPickerEvent::Cancelled),
            KeyCode::Enter => Some(ModifierPickerEvent::ModifiersSelected(self.get_modifiers_list())),
            KeyCode::Char(' ') => {
                self.state.toggle_focused();
                None
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.state.focus_up();
                None
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.state.focus_down();
                None
            }
            KeyCode::Left | KeyCode::Char('h') => {
                self.state.focus_left();
                None
            }
            KeyCode::Right | KeyCode::Char('l') => {
                self.state.focus_right();
                None
            }
            _ => None,
        }
    }

    fn render(&self, f: &mut Frame, _area: Rect, theme: &Theme) {
        render_modifier_picker_component(f, self, theme);
    }
}

/// Render the modifier picker popup (Component version)
fn render_modifier_picker_component(f: &mut Frame, picker: &ModifierPicker, theme: &Theme) {
    let area = centered_rect(50, 60, f.size());

    // Clear background
    f.render_widget(Clear, area);
    f.render_widget(
        Block::default().style(Style::default().bg(theme.background)),
        area,
    );

    // Main layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Length(1), // Spacer
            Constraint::Length(6), // Modifier grid (4 rows)
            Constraint::Length(1), // Spacer
            Constraint::Length(3), // Presets row
            Constraint::Length(1), // Spacer
            Constraint::Length(2), // Selected display
            Constraint::Min(2),    // Help text
        ])
        .split(area);

    // Title
    let title = Paragraph::new(" Select Modifier(s) ")
        .style(
            Style::default()
                .fg(theme.primary)
                .add_modifier(StyleModifier::BOLD),
        )
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    // Modifier grid - split into left and right columns
    let grid_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[2]);

    // Left column header + modifiers
    render_modifier_column(f, grid_chunks[0], &picker.state, true, theme);

    // Right column header + modifiers
    render_modifier_column(f, grid_chunks[1], &picker.state, false, theme);

    // Presets row
    let preset_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[4]);

    render_preset(
        f,
        preset_chunks[0],
        ModifierPreset::Meh,
        picker.state.focus == 8,
        &picker.state,
        theme,
    );
    render_preset(
        f,
        preset_chunks[1],
        ModifierPreset::Hyper,
        picker.state.focus == 9,
        &picker.state,
        theme,
    );

    // Selected display
    let selected_text = if picker.state.has_selection() {
        format!(" Selected: {}", picker.state.to_mod_string())
    } else {
        " Selected: (none)".to_string()
    };
    let selected = Paragraph::new(selected_text).style(Style::default().fg(theme.accent));
    f.render_widget(selected, chunks[6]);

    // Help text
    let help_spans = vec![
        Span::styled(
            "↑↓←→",
            Style::default()
                .fg(theme.primary)
                .add_modifier(StyleModifier::BOLD),
        ),
        Span::raw(" Navigate  "),
        Span::styled(
            "Space",
            Style::default()
                .fg(theme.primary)
                .add_modifier(StyleModifier::BOLD),
        ),
        Span::raw(" Toggle  "),
        Span::styled(
            "Enter",
            Style::default()
                .fg(theme.success)
                .add_modifier(StyleModifier::BOLD),
        ),
        Span::raw(" Confirm  "),
        Span::styled(
            "Esc",
            Style::default()
                .fg(theme.error)
                .add_modifier(StyleModifier::BOLD),
        ),
        Span::raw(" Cancel"),
    ];
    let help = Paragraph::new(Line::from(help_spans)).style(Style::default().fg(theme.text_muted));
    f.render_widget(help, chunks[7]);
}

/// Render a column of modifiers (left or right)
fn render_modifier_column(
    f: &mut Frame,
    area: Rect,
    state: &ModifierPickerState,
    is_left: bool,
    theme: &Theme,
) {
    let header = if is_left { " Left Hand" } else { " Right Hand" };
    let header_widget = Paragraph::new(header).style(
        Style::default()
            .fg(theme.text_muted)
            .add_modifier(StyleModifier::BOLD),
    );

    // Split into header + 4 modifier rows
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Header
            Constraint::Length(1), // Ctrl
            Constraint::Length(1), // Shift
            Constraint::Length(1), // Alt
            Constraint::Length(1), // GUI
        ])
        .split(area);

    f.render_widget(header_widget, chunks[0]);

    // Render each modifier
    let start_idx = if is_left { 0 } else { 4 };
    for i in 0..4 {
        let mod_idx = start_idx + i;
        let modifier = QmkModifier::ALL[mod_idx];
        let is_focused = state.focus == mod_idx;
        let is_checked = state.is_selected(modifier as u8);

        let checkbox = if is_checked { "[x]" } else { "[ ]" };
        let text = format!(" {} {}", checkbox, modifier.display_name());

        let style = if is_focused {
            Style::default()
                .fg(theme.background)
                .bg(theme.primary)
                .add_modifier(StyleModifier::BOLD)
        } else if is_checked {
            Style::default().fg(theme.success)
        } else {
            Style::default().fg(theme.text)
        };

        let widget = Paragraph::new(text).style(style);
        f.render_widget(widget, chunks[i + 1]);
    }
}

/// Render a preset option
fn render_preset(
    f: &mut Frame,
    area: Rect,
    preset: ModifierPreset,
    is_focused: bool,
    state: &ModifierPickerState,
    theme: &Theme,
) {
    let is_active = state.selected_mods == preset.bits();
    let checkbox = if is_active { "[x]" } else { "[ ]" };
    let text = format!(" {} {}", checkbox, preset.display_name());

    let style = if is_focused {
        Style::default()
            .fg(theme.background)
            .bg(theme.primary)
            .add_modifier(StyleModifier::BOLD)
    } else if is_active {
        Style::default().fg(theme.success)
    } else {
        Style::default().fg(theme.text)
    };

    let widget = Paragraph::new(text).style(style);
    f.render_widget(widget, area);
}

/// Helper to create centered rectangle
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_modifier_picker_initial_state() {
        let state = ModifierPickerState::new();
        assert_eq!(state.selected_mods, 0);
        assert_eq!(state.focus, 0);
        assert!(!state.has_selection());
    }

    #[test]
    fn test_toggle_individual_modifier() {
        let mut state = ModifierPickerState::new();

        // Toggle LCtrl on
        state.toggle_mod(QmkModifier::LCtrl as u8);
        assert!(state.is_selected(QmkModifier::LCtrl as u8));
        assert!(state.has_selection());

        // Toggle LShift on
        state.toggle_mod(QmkModifier::LShift as u8);
        assert!(state.is_selected(QmkModifier::LCtrl as u8));
        assert!(state.is_selected(QmkModifier::LShift as u8));

        // Toggle LCtrl off
        state.toggle_mod(QmkModifier::LCtrl as u8);
        assert!(!state.is_selected(QmkModifier::LCtrl as u8));
        assert!(state.is_selected(QmkModifier::LShift as u8));
    }

    #[test]
    fn test_to_mod_string_single() {
        let mut state = ModifierPickerState::new();
        state.toggle_mod(QmkModifier::LCtrl as u8);

        assert_eq!(state.to_mod_string(), "MOD_LCTL");
    }

    #[test]
    fn test_to_mod_string_multiple() {
        let mut state = ModifierPickerState::new();
        state.toggle_mod(QmkModifier::LCtrl as u8);
        state.toggle_mod(QmkModifier::LShift as u8);

        assert_eq!(state.to_mod_string(), "MOD_LCTL | MOD_LSFT");
    }

    #[test]
    fn test_to_mod_string_meh() {
        let mut state = ModifierPickerState::new();
        state.selected_mods = ModifierPreset::Meh.bits();

        assert_eq!(state.to_mod_string(), "MOD_LCTL | MOD_LSFT | MOD_LALT");
    }

    #[test]
    fn test_to_mod_string_hyper() {
        let mut state = ModifierPickerState::new();
        state.selected_mods = ModifierPreset::Hyper.bits();

        assert_eq!(
            state.to_mod_string(),
            "MOD_LCTL | MOD_LSFT | MOD_LALT | MOD_LGUI"
        );
    }

    #[test]
    fn test_to_mod_string_empty() {
        let state = ModifierPickerState::new();
        assert_eq!(state.to_mod_string(), "");
    }

    #[test]
    fn test_toggle_focused_modifier() {
        let mut state = ModifierPickerState::new();

        // Focus is at 0 (LCtrl)
        state.toggle_focused();
        assert!(state.is_selected(QmkModifier::LCtrl as u8));

        // Move focus to 1 (LShift) and toggle
        state.focus = 1;
        state.toggle_focused();
        assert!(state.is_selected(QmkModifier::LShift as u8));

        // Move focus to 8 (Meh preset) and toggle
        state.focus = 8;
        state.toggle_focused();
        assert_eq!(state.selected_mods, ModifierPreset::Meh.bits());
    }

    #[test]
    fn test_focus_navigation() {
        let mut state = ModifierPickerState::new();

        // Start at 0 (LCtrl)
        assert_eq!(state.focus, 0);

        // Move right to 4 (RCtrl)
        state.focus_right();
        assert_eq!(state.focus, 4);

        // Move left back to 0 (LCtrl)
        state.focus_left();
        assert_eq!(state.focus, 0);

        // Move down to 1 (LShift)
        state.focus_down();
        assert_eq!(state.focus, 1);

        // Move up back to 0 (LCtrl)
        state.focus_up();
        assert_eq!(state.focus, 0);

        // Move up wraps to 8 (Meh)
        state.focus_up();
        assert_eq!(state.focus, 8);
    }

}
